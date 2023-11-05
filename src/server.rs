use std::path;

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
use rusqlite::Connection;
use rust_embed::RustEmbed;
use tower::ServiceBuilder;

use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use tracing::info;

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
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

#[derive(RustEmbed)]
#[folder = "web/build/client"]
struct Assets;

type JsonResponse<T> = Result<Json<T>, AppError>;

async fn call_db<F, T>(db_pool: &Pool, f: F) -> Result<T>
where
    F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    db_pool
        .get()
        .await?
        .interact(f)
        .await
        .map_err(|error| anyhow!("cannot call db: {error}"))?
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

async fn get_item(State(db_pool): State<Pool>, Path(id): Path<i64>) -> JsonResponse<Item> {
    call_db(&db_pool, move |db| db.get_item(id))
        .await
        .map(Json)
        .map_err(Into::into)
}

async fn get_items(
    State(db_pool): State<Pool>,
    filter: Query<ItemFilter>,
) -> JsonResponse<ItemSearchResult> {
    call_db(&db_pool, move |db| {
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

    let db_pool = Config::new(db).create_pool(Runtime::Tokio1)?;

    call_db(&db_pool, |db| db.prepare_schema())
        .await
        .with_context(|| format!("cannot prepare database schema in {}", db.display()))?;

    let app = Router::new()
        .route("/api/item/:id", get(get_item))
        .route("/api/items", get(get_items))
        .fallback(get(get_asset).post(submit_item))
        .with_state(db_pool)
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
    State(db_pool): State<Pool>,
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

    let mut item = Item {
        id: None,
        submit_date: Utc::now()
            .naive_utc()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        system: None,
        r#type: None,
        headers,
        body: body.to_vec(),
    };

    item.update_metadata();

    call_db(&db_pool, move |db| db.insert_item(&item))
        .await
        .map(Json)
        .map_err(Into::into)
}
