use std::{
    net::SocketAddr,
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use anyhow::{Error, Result};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{ConnectInfo, Path, Query, State},
    http::{
        HeaderMap, HeaderName, HeaderValue, Request, StatusCode, Uri,
        header::{CACHE_CONTROL, CONTENT_TYPE},
        uri::PathAndQuery,
    },
    response::{IntoResponse, Redirect, Response},
    routing::get,
    serve,
};

use memchr::memmem;
use rust_embed::RustEmbed;
use serde::Serialize;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{Span, error, error_span, field, instrument, trace};

use crate::{
    model::{ItemFilter, NewItemHeader},
    service::Service,
};

static REQUEST_ID: AtomicUsize = AtomicUsize::new(1);

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

async fn get_asset(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if let Some(content) = Assets::get(path) {
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

#[instrument(skip_all, fields(filter))]
async fn get_index_html<S: Service>(
    State(service): State<S>,
    Query(mut filter): Query<ItemFilter>,
) -> impl IntoResponse {
    filter.batch_size = Some(51);
    filter.load_first_item = Some(true);

    let items = service.get_items(&filter).await?;

    respond_with_data(items)
}

#[instrument(skip_all, fields(id))]
async fn get_item<S: Service>(
    State(service): State<S>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    Ok(service.get_item(id).await?.map_or_else(
        || StatusCode::NOT_FOUND.into_response(),
        |item| Json(item).into_response(),
    ))
}

#[instrument(skip_all, fields(id))]
async fn get_item_html<S: Service>(
    State(service): State<S>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    Ok(service.get_item(id).await?.map_or_else(
        || StatusCode::NOT_FOUND.into_response(),
        |item| respond_with_data(item).into_response(),
    ))
}

#[instrument(skip_all, fields(filter))]
async fn get_items<S: Service>(
    State(service): State<S>,
    Query(filter): Query<ItemFilter>,
) -> impl IntoResponse {
    service.get_items(&filter).await.to_json_response()
}

#[instrument(skip_all, fields(id))]
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

async fn redirect_to_base(uri: Uri) -> impl IntoResponse {
    Redirect::permanent(&format!(
        "/sink{}",
        uri.path_and_query()
            .map(PathAndQuery::as_str)
            .unwrap_or_default()
    ))
}

fn respond_with_data(data: impl Serialize) -> Result<impl IntoResponse, AppError> {
    const INITIAL_DATA: &[u8] = b"'%INITIAL_DATA%'";

    let response = if let Some(page) = Assets::get("index.html") {
        let body = page.data;
        let data = serde_json::to_string(&data)?;

        let mut body_with_data = Vec::with_capacity(body.len() + data.len());

        if let Some(i) = memmem::find(&body, INITIAL_DATA) {
            body_with_data.extend_from_slice(&body[0..i]);
            body_with_data.extend_from_slice(data.as_bytes());
            body_with_data.extend_from_slice(&body[i + INITIAL_DATA.len()..]);
        }

        ([(CONTENT_TYPE, page.metadata.mimetype())], body_with_data).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    };

    Ok(response)
}

pub async fn start<S>(host: &str, port: u16, service: S) -> Result<()>
where
    S: Service + 'static,
{
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(trace_layer_make_span_with)
        .on_request(trace_layer_on_request)
        .on_response(trace_layer_on_response);

    let router = Router::new()
        .nest(
            "/sink/",
            Router::new()
                .route("/", get(get_index_html::<S>))
                .route("/item/{id}", get(get_item_html::<S>))
                .nest(
                    "/api",
                    Router::new()
                        .route("/item/{id}", get(get_item::<S>))
                        .route("/items", get(get_items::<S>))
                        .route(
                            "/raw-item/{id}",
                            get(get_raw_item::<S>).post(get_raw_item::<S>),
                        ),
                )
                .fallback(get(get_asset).post(submit_item::<S>)),
        )
        .fallback(get(redirect_to_base).post(submit_item::<S>))
        .with_state(service)
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(trace_layer),
        );

    let listener = TcpListener::bind((host, port)).await?;

    serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(Into::into)
}

#[instrument(skip_all)]
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

fn trace_layer_make_span_with(request: &Request<Body>) -> Span {
    let request_id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);

    error_span!("request",
        id = request_id,
        uri = %request.uri(),
        method = %request.method(),
        source = request.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map_or_else(|| field::display(String::from("<unknown>")), |connect_info| field::display(connect_info.ip().to_string())),
        status = field::Empty,
        latency = field::Empty,
    )
}

fn trace_layer_on_request(_request: &Request<Body>, _span: &Span) {
    trace!("got request");
}

fn trace_layer_on_response(response: &Response, latency: Duration, span: &Span) {
    span.record(
        "latency",
        field::display(format!("{}ms", latency.as_millis())),
    );

    span.record("status", field::display(response.status()));

    trace!("responded");
}
