use std::{path, sync::Arc};

use anyhow::{anyhow, Context, Error, Result};

use axum::{
    async_trait,
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

use deadpool_sqlite::{Config, Pool, Runtime};
use regex::bytes::Regex;
use rusqlite::Connection;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{error, info};

use crate::{
    repository::Repository,
    shared::{Item, ItemFilter, ItemHeader, ItemSummary, NewItem},
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
    item_types: Arc<Vec<ItemType>>,
}

#[derive(RustEmbed)]
#[folder = "web/build"]
struct Assets;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemSearchResult {
    items: Vec<ItemSummary>,
    total_items: i32,
    systems: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    first_item: Option<Item>,
}

struct ItemType {
    key: String,
    patterns: Vec<Regex>,
}

type JsonResponse<T> = Result<Json<T>, AppError>;

#[async_trait]
trait PoolExt {
    async fn call_db<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static;
}

#[async_trait]
impl PoolExt for Pool {
    async fn call_db<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        self.get()
            .await?
            .interact(f)
            .await
            .map_err(|error| anyhow!("cannot call db: {error}"))?
    }
}

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
        .db_pool
        .call_db(move |db| db.get_item(id))
        .await
        .map(Json)
        .map_err(Into::into)
}

fn get_item_type(item_types: &[ItemType], body: &[u8]) -> Option<String> {
    'next_type: for item_type in item_types {
        for pattern in &item_type.patterns {
            if !pattern.is_match(body) {
                continue 'next_type;
            }
        }

        return Some(item_type.key.to_string());
    }

    None
}

fn get_item_types() -> Result<Vec<ItemType>> {
    #[derive(Deserialize)]
    struct ItemTypeConfig {
        key: String,
        matches: Vec<String>,
    }

    let configs: Vec<ItemTypeConfig> =
        from_str(include_str!("../item.types.json")).context("cannot parse item.types.json")?;

    let mut item_types = Vec::new();

    for config in configs {
        let mut item_type = ItemType {
            key: config.key,
            patterns: Vec::new(),
        };

        for pattern in config.matches {
            item_type.patterns.push(
                Regex::new(&pattern).with_context(|| {
                    format!("wrong pattern {pattern} for key {}", &item_type.key)
                })?,
            );
        }

        item_types.push(item_type);
    }

    Ok(item_types)
}

async fn get_items(
    State(app_state): State<AppState>,
    filter: Query<ItemFilter>,
) -> JsonResponse<ItemSearchResult> {
    app_state
        .db_pool
        .call_db(move |db| {
            let load_first_item = filter.load_first_item.unwrap_or_default();
            let (items, total_items) = db.get_items(filter.0)?;

            let first_item = if load_first_item && !items.is_empty() {
                Some(db.get_item(items[0].id)?)
            } else {
                None
            };

            let systems = db.get_systems()?;

            Ok(ItemSearchResult {
                items,
                total_items,
                systems,
                first_item,
            })
        })
        .await
        .map(Json)
        .map_err(Into::into)
}

fn get_system(headers: &[ItemHeader], body: &[u8]) -> Result<Option<String>> {
    let mut system = headers
        .iter()
        .find(|header| header.is_mgs_system_header())
        .map(|header| String::from_utf8_lossy(&header.value).into_owned());

    if system.is_none() {
        let regex = Regex::new("<mgsSystem>([^<]+)")?;

        if let Some(captures) = regex.captures(body) {
            if let Some(group) = captures.get(1) {
                system = Some(String::from_utf8_lossy(group.as_bytes()).into_owned());
            }
        }
    }

    Ok(system)
}

#[tokio::main]
pub async fn start(host: &str, port: u16, db: &path::Path) -> Result<()> {
    info!(host, port, ?db, "starting server");

    let db_pool = Config::new(db).create_pool(Runtime::Tokio1)?;
    let item_types = get_item_types()?;

    db_pool
        .call_db(|db| {
            db.prepare_schema()?;
            db.init()
        })
        .await
        .with_context(|| format!("cannot prepare database schema in {}", db.display()))?;

    let app_state = AppState {
        db_pool,
        item_types: Arc::new(item_types),
    };

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
    app_state
        .db_pool
        .call_db(move |db| {
            let headers: Vec<ItemHeader> = headers.iter().map(Into::into).collect();
            let system = get_system(&headers, &body)?;
            let item_type = get_item_type(&app_state.item_types, &body);

            let item = NewItem {
                system,
                r#type: item_type,
                headers,
                body: &body,
            };

            db.insert_item(&item)
        })
        .await
        .map(Json)
        .map_err(Into::into)
}
