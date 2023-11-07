use std::{collections::HashMap, path};

use anyhow::{anyhow, Context, Error, Result};

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        HeaderMap, StatusCode, Uri,
    },
    response::{IntoResponse, Response},
    routing::get,
    Json, Router, Server,
};

use chrono::Utc;
use deadpool_sqlite::{Config, Pool, Runtime};

use quick_xml::{
    events::{
        attributes::Attributes,
        Event::{End, Eof, Start, Text},
    },
    Reader,
};

use rusqlite::Connection;
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::{from_str, Value};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{error, info};

use crate::{
    repository::Repository,
    shared::{Item, ItemFilter, ItemHeader, ItemSearchResult},
};

struct AppError(Error);

impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error = format!("{}", self.0);
        error!(error);
        (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
    }
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool,
    item_types: Vec<ItemType>,
}

impl AppState {
    async fn call_db<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        self.db_pool
            .get()
            .await?
            .interact(f)
            .await
            .map_err(|error| anyhow!("cannot call db: {error}"))?
    }

    fn get_xml_item_type(&self, path: &str, attributes: &Attributes) -> Option<&str> {
        for item_type in &self.item_types {
            if let Some(path_attributes) = item_type.xml_paths.get(path) {
                let mut matched_atributes = 0;

                for attribute in attributes.clone().flatten() {
                    let local_name = attribute.key.local_name();

                    if let Some(value) =
                        path_attributes.get(String::from_utf8_lossy(local_name.as_ref()).as_ref())
                    {
                        if attribute.value == value.as_bytes() {
                            matched_atributes += 1;
                        }
                    }
                }

                if matched_atributes == path_attributes.len() {
                    return Some(&item_type.key);
                }
            }
        }

        None
    }

    fn new(db: &path::Path) -> Result<Self> {
        let db_pool = Config::new(db).create_pool(Runtime::Tokio1)?;

        let item_types: Vec<ItemType> =
            from_str(include_str!("../item.types.json")).context("cannot parse item.types.json")?;

        Ok(Self {
            db_pool,
            item_types,
        })
    }
}

#[derive(RustEmbed)]
#[folder = "web/build"]
struct Assets;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemType {
    key: String,

    #[serde(default)]
    xml_paths: HashMap<String, HashMap<String, String>>,
}

type JsonResponse<T> = Result<Json<T>, AppError>;

async fn get_asset(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if let Some(content) = Assets::get(path).or_else(|| Assets::get("index.html")) {
        let mime = content.metadata.mimetype();

        if path.starts_with("_app/immutable") {
            (
                [
                    (CACHE_CONTROL, "public, max-age=31536000, immutable"),
                    (CONTENT_TYPE, mime),
                ],
                content.data,
            )
                .into_response()
        } else {
            ([(CONTENT_TYPE, mime)], content.data).into_response()
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn get_item(State(app_state): State<AppState>, Path(id): Path<i64>) -> JsonResponse<Item> {
    app_state
        .call_db(move |db| db.get_item(id))
        .await
        .map(Json)
        .map_err(Into::into)
}

async fn get_items(
    State(app_state): State<AppState>,
    filter: Query<ItemFilter>,
) -> JsonResponse<ItemSearchResult> {
    app_state
        .call_db(move |db| {
            let mut items = db.get_items(&filter)?;

            if filter.load_first_item.unwrap_or_default() && !items.items.is_empty() {
                items.first_item = Some(db.get_item(items.items[0].id)?);
            }

            Ok(items)
        })
        .await
        .map(Json)
        .map_err(Into::into)
}

#[tokio::main]
pub async fn start(host: &str, port: u16, db: &path::Path) -> Result<()> {
    info!(host, port, ?db, "starting server");

    let app_state = AppState::new(db)?;

    app_state
        .call_db(|db| {
            db.prepare_schema()?;
            db.init()
        })
        .await
        .with_context(|| format!("cannot prepare database schema in {}", db.display()))?;

    let app = Router::new()
        .route("/api/item/:id", get(get_item))
        .route("/api/items", get(get_items))
        .fallback(get(get_asset).post(submit_item))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http()),
        );

    Server::bind(&format!("{host}:{port}").parse()?)
        .serve(app.into_make_service())
        .await?;

    info!("bye!");

    Ok(())
}

async fn submit_item(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> JsonResponse<i64> {
    let headers: Vec<ItemHeader> = headers
        .iter()
        .map(|(name, value)| ItemHeader {
            name: name.to_string(),
            value: value.as_bytes().into(),
        })
        .collect();

    let mut system = headers
        .iter()
        .filter(|header| header.name == "mgs-system-id" || header.name == "mgssystem")
        .map(|header| String::from_utf8_lossy(&header.value).to_string())
        .next();

    let mut item_type = None;

    if let Ok(json) = serde_json::from_slice::<Value>(&body) {
        if json.get("entityEventId").is_some() {
            item_type = Some(if json.get("eventDesc").is_some() {
                "event_payload"
            } else {
                "event_notification"
            });
        }
    } else {
        let mut xml_reader = Reader::from_reader(body.as_ref());
        let mut buffer = Vec::new();
        let mut path = String::new();

        loop {
            if system.is_some() && item_type.is_some() {
                break;
            }

            buffer.clear();

            if let Ok(event) = xml_reader.read_event_into(&mut buffer) {
                match event {
                    Start(element) => {
                        path.push('/');
                        path.push_str(&String::from_utf8_lossy(element.local_name().as_ref()));

                        if item_type.is_none() {
                            item_type = app_state.get_xml_item_type(&path, &element.attributes());
                        }
                    }
                    End(_) => {
                        if let Some(idx) = path.rfind('/') {
                            path.truncate(idx);
                        }
                    }
                    Text(_) => {
                        if system.is_none() && path.ends_with("/mgsSystem") {
                            system = Some(String::from_utf8_lossy(&buffer).to_string());
                        }
                    }
                    Eof => break,
                    _ => {}
                }
            } else {
                break;
            }
        }
    }

    let item = Item {
        id: None,
        submit_date: Utc::now()
            .naive_utc()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        system,
        r#type: item_type.map(ToString::to_string),
        headers,
        body: body.to_vec(),
    };

    app_state
        .call_db(move |db| db.insert_item(&item))
        .await
        .map(Json)
        .map_err(Into::into)
}
