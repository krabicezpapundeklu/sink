use std::path::PathBuf;

use anyhow::{Error, Result};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{
        HeaderMap, HeaderName, HeaderValue, StatusCode, Uri,
        header::{CACHE_CONTROL, CONTENT_TYPE},
        uri::PathAndQuery,
    },
    response::{IntoResponse, Redirect, Response},
    routing::get,
    serve,
};

use const_format::concatcp;
use rust_embed::RustEmbed;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{error, info};

use crate::shared::{AppContext, BASE, Item, ItemFilter, ItemHeader, ItemSearchResult};

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
        if self.0.downcast_ref::<rusqlite::Error>() == Some(&rusqlite::Error::QueryReturnedNoRows) {
            return (StatusCode::NOT_FOUND, "Not found").into_response();
        }

        let error = format!("{}", self.0);
        error!(error);
        (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
    }
}

#[derive(RustEmbed)]
#[folder = "web/build"]
#[include = "*.css"]
#[include = "*.html"]
#[include = "*.js"]
#[include = "*.json"]
#[include = "*.png"]
#[include = "*.txt"]
#[cfg_attr(debug_assertions, include = "*.map")]
struct Assets;

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
    let original_path = uri.path();

    let path = original_path
        .trim_start_matches(concatcp!(BASE, '/'))
        .trim_start_matches('/');

    let mut asset = if path == "index.html" {
        None
    } else {
        Assets::get(path)
    };

    if asset.is_none() {
        if !original_path.starts_with(concatcp!(BASE, '/')) {
            return Ok(Redirect::permanent(&format!(
                "{BASE}{}",
                uri.path_and_query()
                    .map(PathAndQuery::as_str)
                    .unwrap_or_default()
            ))
            .into_response());
        }

        asset = Assets::get("index.html");
    }

    let response = if let Some(content) = asset {
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

async fn get_raw_item(
    State(app_context): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let item = app_context.get_item(id).await?;
    let mut headers = HeaderMap::new();

    if let Some(content_type) = item
        .headers
        .iter()
        .find(|header| header.name == "content-type")
    {
        headers.insert(
            "content-type",
            HeaderValue::from_bytes(&content_type.value)?,
        );
    }

    for header in item.headers {
        if let Some(header_name) = header.name.strip_prefix("x-response-header-") {
            headers.insert(
                HeaderName::from_bytes(header_name.as_bytes())?,
                HeaderValue::from_bytes(&header.value)?,
            );
        }
    }

    Ok((headers, item.body))
}

pub async fn start(host: &str, port: u16, db: PathBuf) -> Result<()> {
    info!(host, port, ?db, "starting server");

    let app_context = AppContext::new(db).await?;

    let app = Router::new()
        .route(concatcp!(BASE, "/api/item/:id"), get(get_item))
        .route(concatcp!(BASE, "/api/items"), get(get_items))
        .route(
            concatcp!(BASE, "/api/raw-item/:id"),
            get(get_raw_item).post(get_raw_item),
        )
        .fallback(get(get_asset).post(submit_item))
        .with_state(app_context)
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http()),
        );

    let listener = TcpListener::bind(&format!("{host}:{port}")).await?;

    serve(listener, app).await?;

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
