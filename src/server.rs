use std::path::PathBuf;

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

use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use regex::bytes::{Regex, RegexSet};
use rusqlite::Connection;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tokio::task::spawn_blocking;
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
struct AppContext {
    db_pool: DbPool,
    item_types: Vec<(String, usize)>,
    item_type_patterns: RegexSet,
    system_pattern: Regex,
    initial_data_pattern: Regex,
}

impl AppContext {
    async fn call_db<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let mut db = self.get_db().await?;

        spawn_blocking(move || f(&mut db))
            .await?
            .map_err(Into::into)
    }

    async fn get_db(&self) -> Result<Object<DbManager>> {
        self.db_pool
            .get()
            .await
            .map_err(|error| anyhow!("cannot get db: {error}"))
    }

    async fn get_initial_data(&self, uri: &Uri) -> Result<Option<(String, String)>> {
        let id = uri.path().strip_prefix("/item/");

        if let Some(id) = id {
            let id: i64 = id.parse()?;
            let item = self.get_item(id).await?;

            Ok(Some((
                format!("/api/item/{id}"),
                serde_json::to_string(&item)?,
            )))
        } else {
            Ok(None)
        }
    }

    async fn get_item(&self, id: i64) -> Result<Item> {
        self.call_db(move |db| db.get_item(id)).await
    }

    fn get_item_type(&self, body: &[u8]) -> Option<String> {
        let matches = self.item_type_patterns.matches(body);

        if matches.matched_any() {
            let mut i = 0;

            'next_item_type: for (key, patterns) in &self.item_types {
                for j in 0..*patterns {
                    if !matches.matched(i + j) {
                        i += patterns;
                        continue 'next_item_type;
                    }
                }

                return Some(key.to_string());
            }
        }

        None
    }

    async fn get_items(&self, filter: ItemFilter) -> Result<ItemSearchResult> {
        self.call_db(move |db| {
            let load_first_item = filter.load_first_item.unwrap_or_default();
            let (items, total_items) = db.get_items(filter)?;

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
    }

    fn get_system(&self, headers: &[ItemHeader], body: &[u8]) -> Option<String> {
        let mut system = headers
            .iter()
            .find(|header| header.is_mgs_system_header())
            .map(|header| String::from_utf8_lossy(&header.value).into_owned());

        if system.is_none() {
            if let Some(captures) = self.system_pattern.captures(body) {
                if let Some(group) = captures.get(1) {
                    system = Some(String::from_utf8_lossy(group.as_bytes()).into_owned());
                }
            }
        }

        system
    }

    fn new(db: PathBuf) -> Result<Self> {
        #[derive(Deserialize)]
        struct ItemType {
            key: String,
            matches: Vec<String>,
        }

        let db_manager = DbManager { db };
        let db_pool = DbPool::builder(db_manager).build()?;

        let item_types: Vec<ItemType> =
            from_str(include_str!("../item.types.json")).context("cannot parse item.types.json")?;

        let app_context = Self {
            db_pool,
            item_types: item_types
                .iter()
                .map(|item_type| (item_type.key.to_string(), item_type.matches.len()))
                .collect(),
            item_type_patterns: RegexSet::new(
                item_types
                    .iter()
                    .flat_map(|item_type| item_type.matches.iter()),
            )?,
            system_pattern: Regex::new("<mgsSystem>([^<]+)")?,
            initial_data_pattern: Regex::new(r"<!--\s*%INITIAL_DATA%\s*-->")?,
        };

        Ok(app_context)
    }

    async fn submit_item(&self, headers: Vec<ItemHeader>, body: Bytes) -> Result<i64> {
        let system = self.get_system(&headers, &body);
        let item_type = self.get_item_type(&body);

        self.call_db(move |db| {
            let item = NewItem {
                system,
                r#type: item_type,
                headers,
                body: &body,
            };

            db.insert_item(&item)
        })
        .await
    }
}

#[derive(RustEmbed)]
#[folder = "web/build"]
struct Assets;

struct DbManager {
    db: PathBuf,
}

#[async_trait]
impl Manager for DbManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> Result<Connection, Error> {
        let db = Connection::open(&self.db)?;
        db.init()?;
        Ok(db)
    }

    async fn recycle(&self, _: &mut Connection, _: &Metrics) -> RecycleResult<Error> {
        Ok(())
    }
}

type DbPool = Pool<DbManager>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemSearchResult {
    items: Vec<ItemSummary>,
    total_items: i32,
    systems: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    first_item: Option<Item>,
}

type JsonResponse<T> = Result<Json<T>, AppError>;

trait ResultExt<T> {
    fn to_json_response(self) -> JsonResponse<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn to_json_response(self) -> JsonResponse<T> {
        self.map(Json).map_err(Into::into)
    }
}

async fn get_asset(State(app_context): State<AppContext>, uri: Uri) -> Result<Response, AppError> {
    let path = uri.path().trim_start_matches('/');

    let response = if let Some(content) = Assets::get(path).or_else(|| Assets::get("index.html")) {
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
        } else if let Ok(Some((url, initial_data))) = app_context.get_initial_data(&uri).await {
            let initial_data = format!(
                r#"<script type="application/json" data-sveltekit-fetched data-url="{url}">
                    {{"status": 200, "statusText": "OK", "headers": {{}}, "body": {}}}
                    </script>"#,
                serde_json::to_string(&initial_data)?
            );

            let body = app_context
                .initial_data_pattern
                .replace(&content.data, initial_data.as_bytes())
                .to_vec();

            ([(CONTENT_TYPE, mime)], body).into_response()
        } else {
            ([(CONTENT_TYPE, mime)], content.data).into_response()
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    };

    Ok(response)
}

async fn get_item(
    State(app_context): State<AppContext>,
    Path(id): Path<i64>,
) -> JsonResponse<Item> {
    app_context.get_item(id).await.to_json_response()
}

async fn get_items(
    State(app_context): State<AppContext>,
    filter: Query<ItemFilter>,
) -> JsonResponse<ItemSearchResult> {
    app_context.get_items(filter.0).await.to_json_response()
}

#[tokio::main]
pub async fn start(host: &str, port: u16, db: PathBuf) -> Result<()> {
    info!(host, port, ?db, "starting server");

    let app_context = AppContext::new(db)?;

    app_context
        .call_db(|db| db.prepare_schema())
        .await
        .context("cannot prepare database schema")?;

    let app = Router::new()
        .route("/api/item/:id", get(get_item))
        .route("/api/items", get(get_items))
        .fallback(get(get_asset).post(submit_item))
        .with_state(app_context)
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
    State(app_context): State<AppContext>,
    headers: HeaderMap,
    body: Bytes,
) -> JsonResponse<i64> {
    let headers: Vec<ItemHeader> = headers.iter().map(Into::into).collect();

    app_context
        .submit_item(headers, body)
        .await
        .to_json_response()
}
