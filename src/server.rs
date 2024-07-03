use std::{path::PathBuf, sync::OnceLock};

use aho_corasick::{AhoCorasick, MatchKind::LeftmostFirst};
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

macro_rules! render_page {
    ($page:expr, $data:expr, $data_url:expr, $($name:literal = $value:expr),*) => {{
        static AC: OnceLock<AhoCorasick> = OnceLock::new();

        let initial_data = serde_json::to_string(&serde_json::to_string(&$data)?)?;
        let replacements = &[$data_url, &initial_data[1..initial_data.len() - 1], $($value),*];
        let page = Assets::get($page).unwrap();

        let body = AC
            .get_or_init(|| {
                AhoCorasick::builder()
                    .match_kind(LeftmostFirst)
                    .build(["#initial_data_url#", "#initial_data_body#", $($name),*])
                    .unwrap()
            })
            .replace_all_bytes(&page.data, replacements);

        Ok(Html(body))
    }};
}

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
) -> Result<Html<Vec<u8>>, AppError> {
    let mut filter = filter.0;

    filter.load_first_item = Some(true);

    let mut initial_uri = format!("{BASE}/api/items?batchSize=51&loadFirstItem=true");

    if let Some(query) = uri.query() {
        initial_uri.push('&');
        initial_uri.push_str(query);
    }

    let sort_by = if filter.asc.unwrap_or(false) {
        "Oldest"
    } else {
        "Latest"
    };

    let items = app_context.get_items(filter).await?;

    render_page!(
        "index.html",
        items,
        &initial_uri,
        "#total_items#" = &items.total_items.to_formatted_string(&Locale::en),
        "#sort_by#" = sort_by
    )
}

async fn get_item(
    State(app_context): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Html<Vec<u8>>, AppError> {
    let item = app_context.get_item(id).await?;

    render_page!(
        "item/0.html",
        item,
        &format!("{BASE}/api/item/{id}"),
        "#id#" = &id.to_formatted_string(&Locale::en)
    )
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
