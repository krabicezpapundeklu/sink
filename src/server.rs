use std::{
    fmt::{self, Display, Formatter},
    path,
};

use actix_cors::Cors;

use actix_web::{
    dev::Service,
    get,
    http::header::{ContentType, HeaderValue, CACHE_CONTROL},
    middleware::{Compress, Logger, NormalizePath, TrailingSlash},
    post,
    web::{Bytes, Data, Json, Path, Query},
    App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
};

use actix_web_static_files::ResourceFiles;
use anyhow::{anyhow, Context, Error, Result};
use deadpool_sqlite::{Config, InteractError, Pool, Runtime};

use rquickjs::{embed, Context as JsContext, Function, Promise, Runtime as JsRuntime, Tokio};

use rusqlite::Connection;
use serde::Serialize;

use crate::{
    repository::Repository,
    shared::{DateTime, Item, ItemFilter, ItemHeader},
};

#[embed(name = "main", path = "./web/build/server")]
mod server_module {}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

type Response<T> = Result<T, ServerError>;

struct RouteData {
    serialized: String,
}

impl RouteData {
    fn new<T: Serialize>(path: &str, data: &T) -> Result<Self> {
        let path = serde_json::to_string_pretty(path)?;
        let data = serde_json::to_string_pretty(data)?;

        Ok(Self {
            serialized: format!("{{url: {path}, data: {data}}}"),
        })
    }

    fn null() -> Self {
        Self {
            serialized: "null".to_string(),
        }
    }
}
#[derive(Debug)]
struct ServerError(Error);

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Error> for ServerError {
    fn from(value: Error) -> Self {
        Self(value)
    }
}

impl From<InteractError> for ServerError {
    fn from(value: InteractError) -> Self {
        Self(map_to_anyhow_error(value))
    }
}

impl ResponseError for ServerError {}

async fn call_db<F, T, E>(pool: &Data<Pool>, f: F) -> Response<T>
where
    F: FnOnce(&mut Connection) -> Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Send + Into<Error> + 'static,
{
    pool.get()
        .await
        .map_err(Error::new)?
        .interact(f)
        .await?
        .map_err(|err| ServerError(err.into()))
}

#[get("/")]
async fn get_index_html(
    pool: Data<Pool>,
    request: HttpRequest,
    filter: Query<ItemFilter>,
) -> Response<impl Responder> {
    let query_string = request.query_string();
    let mut filter = filter.clone();

    filter.batch_size.get_or_insert(100);

    let items = call_db(&pool, move |db| db.get_items(&filter)).await?;

    let result = render_route("/", &[
            RouteData::null(),
            RouteData::new(&format!("/api/items?firstItemId=0&lastItemId=9007199254740991&batchSize=100&{query_string}"), &items)?
    ]).await?;

    Ok(HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(result))
}

#[get("/api/item/{id}")]
async fn get_item(pool: Data<Pool>, path: Path<i64>) -> Response<impl Responder> {
    let id = path.into_inner();
    call_db(&pool, move |db| db.get_item(id)).await.map(Json)
}

#[get("/item/{id}")]
async fn get_item_html(pool: Data<Pool>, path: Path<i64>) -> Response<impl Responder> {
    let id = path.into_inner();
    let item = call_db(&pool, move |db| db.get_item(id)).await?;

    let result = render_route(
        &format!("/item/{id}"),
        &[
            RouteData::null(),
            RouteData::new(&format!("/api/item/{id}"), &item)?,
        ],
    )
    .await?;

    Ok(HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(result))
}

#[get("/api/items")]
async fn get_items(pool: Data<Pool>, filter: Query<ItemFilter>) -> Response<impl Responder> {
    call_db(&pool, move |db| db.get_items(&filter))
        .await
        .map(Json)
}

fn map_to_anyhow_error(error: InteractError) -> Error {
    anyhow!(match error {
        InteractError::Panic(_) => "panic",
        InteractError::Aborted => "aborted",
    })
}

async fn render_route(path: &str, data: &[RouteData]) -> Result<String> {
    let runtime = JsRuntime::new()?;
    let context = JsContext::full(&runtime)?;

    runtime.set_loader(SERVER_MODULE, SERVER_MODULE);

    let mut serialized_data = String::new();

    for data in data {
        if !serialized_data.is_empty() {
            serialized_data.push_str(", ");
        }

        serialized_data.push_str(&data.serialized);
    }

    let path = serde_json::to_string_pretty(path)?;
    let source = format!("render_route({path}, [{serialized_data}])");

    runtime.spawn_executor(Tokio);

    let result: Promise<String> = context.with(|ctx| {
        let module = ctx.compile(
            "server",
            "import { render_route } from 'main'; export { render_route};",
        )?;

        let render_route: Function = module.get("render_route")?;

        ctx.globals().set("render_route", render_route)?;
        ctx.eval(source)
    })?;

    let result = result.await?;

    runtime.idle().await;

    Ok(result)
}

#[actix_web::main]
pub async fn start_server(host: &str, port: u16, db: &path::Path) -> Result<()> {
    let pool = Config::new(db).create_pool(Runtime::Tokio1)?;

    pool.get()
        .await?
        .interact(|db| db.prepare_schema())
        .await
        .map_err(map_to_anyhow_error)?
        .with_context(|| format!("cannot prepare database schema in {}", db.display()))?;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(get_index_html)
            .service(get_item)
            .service(get_item_html)
            .service(get_items)
            .service(submit_item)
            .service(ResourceFiles::new("/", generate()).resolve_not_found_to_root())
            .wrap(Compress::default())
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap_fn(|request, service| {
                let is_immutable = request.path().starts_with("/_app/immutable/");
                let response = service.call(request);

                async move {
                    let mut response = response.await?;

                    if is_immutable {
                        response.headers_mut().insert(
                            CACHE_CONTROL,
                            HeaderValue::from_static("public, max-age=31536000, immutable"),
                        );
                    }

                    Ok(response)
                }
            })
    })
    .bind((host, port))?
    .run()
    .await
    .map_err(Into::into)
}

#[post("/item")]
async fn submit_item(
    pool: Data<Pool>,
    request: HttpRequest,
    body: Bytes,
) -> Response<impl Responder> {
    let headers: Vec<ItemHeader> = request
        .headers()
        .iter()
        .map(|(name, value)| ItemHeader {
            name: name.to_string(),
            value: value.as_bytes().to_vec(),
        })
        .collect();

    let mut item = Item {
        id: None,
        submit_date: DateTime::now(),
        system: None,
        r#type: None,
        headers,
        body: body.to_vec(),
    };

    call_db(&pool, move |db| {
        item.update_metadata();
        db.insert_item(&item)
    })
    .await
    .map(Json)
}
