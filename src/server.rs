use std::path::PathBuf;

use anyhow::{Error, Result};

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        uri::PathAndQuery,
        HeaderMap, StatusCode, Uri,
    },
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    serve, Json, Router,
};

use const_format::concatcp;
use num_format::{Locale, ToFormattedString};
use rust_embed::RustEmbed;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{error, info};

use crate::shared::{AppContext, Item, ItemFilter, ItemHeader, ItemSearchResult, BASE};

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

mod api {
    use super::{
        AppContext, Item, ItemFilter, ItemSearchResult, JsonResponse, Path, Query, ResultExt, State,
    };

    pub async fn get_item(
        State(app_context): State<AppContext>,
        Path(id): Path<i64>,
    ) -> JsonResponse<Item> {
        app_context.get_item(id).await.to_json_response()
    }

    pub async fn get_items(
        State(app_context): State<AppContext>,
        filter: Query<ItemFilter>,
    ) -> JsonResponse<ItemSearchResult> {
        app_context.get_items(filter.0).await.to_json_response()
    }
}

async fn get_asset(uri: Uri) -> Result<Response, AppError> {
    let original_path = uri.path();

    let path = original_path
        .trim_start_matches(concatcp!(BASE, '/'))
        .trim_start_matches('/');

    let mut asset = if path == "fallback.html" {
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

        asset = Assets::get("fallback.html");
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

async fn get_index(
    State(app_context): State<AppContext>,
    filter: Query<ItemFilter>,
    uri: Uri,
) -> Result<Html<String>, AppError> {
    let mut filter = filter.0;

    filter.load_first_item = Some(true);

    let items = app_context.get_items(filter).await?;
    let initial_data = serde_json::to_string(&serde_json::to_string(&items)?)?;

    let mut initial_uri = format!("{BASE}/api/items?batchSize=51&loadFirstItem=true");

    if let Some(query) = uri.query() {
        initial_uri.push('&');
        initial_uri.push_str(query);
    }

    let page = Assets::get("index.html").unwrap();

    let body = String::from_utf8_lossy(&page.data)
        .replace("#initial_data_url#", &initial_uri)
        .replace(
            "#initial_data_body#",
            &initial_data[1..initial_data.len() - 1],
        )
        .replace(
            "#total_items#",
            &items.total_items.to_formatted_string(&Locale::en),
        );

    Ok(Html(body))
}

async fn get_item(
    State(app_context): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let item = app_context.get_item(id).await?;
    let initial_data = serde_json::to_string(&serde_json::to_string(&item)?)?;
    let page = Assets::get("item/0.html").unwrap();

    let body = String::from_utf8_lossy(&page.data)
        .replace("#initial_data_url#", &format!("{BASE}/api/item/{id}"))
        .replace(
            "#initial_data_body#",
            &initial_data[1..initial_data.len() - 1],
        )
        .replace("#id#", &id.to_formatted_string(&Locale::en));

    Ok(Html(body))
}

pub async fn start(host: &str, port: u16, db: PathBuf) -> Result<()> {
    info!(host, port, ?db, "starting server");

    let app_context = AppContext::new(db).await?;

    let app = Router::new()
        .route(concatcp!(BASE, "/"), get(get_index))
        .route(concatcp!(BASE, "/item/:id"), get(get_item))
        .route(concatcp!(BASE, "/api/item/:id"), get(api::get_item))
        .route(concatcp!(BASE, "/api/items"), get(api::get_items))
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
