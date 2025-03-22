use std::str::FromStr;

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

use rust_embed::RustEmbed;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

use crate::{
    model::{ItemFilter, NewItemHeader},
    service::Service,
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

async fn get_asset(uri: Uri) -> Result<impl IntoResponse, AppError> {
    let original_path = uri.path();

    let path = original_path
        .trim_start_matches("/sink/")
        .trim_start_matches('/');

    let mut asset = if path == "index.html" {
        None
    } else {
        Assets::get(path)
    };

    if asset.is_none() {
        if !original_path.starts_with("/sink/") {
            return Ok(Redirect::permanent(&format!(
                "/sink{}",
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
        } else {
            ([(CONTENT_TYPE, mime)], content.data).into_response()
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    };

    Ok(response)
}

async fn get_item<S: Service>(
    State(service): State<S>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    Ok(service.get_item(id).await?.map_or_else(
        || StatusCode::NOT_FOUND.into_response(),
        |item| Json(item).into_response(),
    ))
}

async fn get_items<S: Service>(
    State(service): State<S>,
    Query(filter): Query<ItemFilter>,
) -> impl IntoResponse {
    service.get_items(&filter).await.to_json_response()
}

async fn get_raw_item<S: Service>(
    State(service): State<S>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let response = if let Some(item) = service.get_item(id).await? {
        let mut headers = HeaderMap::new();

        if let Some(content_type) = item.content_type() {
            headers.insert(
                CONTENT_TYPE.as_str(),
                HeaderValue::from_bytes(content_type)?,
            );
        }

        for (name, value) in item.x_response_headers() {
            headers.insert(HeaderName::from_str(name)?, HeaderValue::from_bytes(value)?);
        }

        (headers, item.body).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    };

    Ok(response)
}

pub async fn start<S>(host: &str, port: u16, service: S) -> Result<()>
where
    S: Service + 'static,
{
    let api_routes = Router::new()
        .route("/item/{id}", get(get_item::<S>))
        .route("/items", get(get_items::<S>))
        .route(
            "/raw-item/{id}",
            get(get_raw_item::<S>).post(get_raw_item::<S>),
        );

    let router = Router::new()
        .nest("/sink/api", api_routes)
        .fallback(get(get_asset).post(submit_item::<S>))
        .with_state(service)
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()));

    let listener = TcpListener::bind((host, port)).await?;

    serve(listener, router).await.map_err(Into::into)
}

async fn submit_item<S: Service>(
    State(service): State<S>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let headers: Vec<_> = headers
        .iter()
        .map(|(name, value)| NewItemHeader {
            name: name.as_str(),
            value: value.as_bytes(),
        })
        .collect();

    service.save_item(&headers, &body).await.to_json_response()
}
