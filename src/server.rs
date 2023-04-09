use std::{
    fmt::{self, Display, Formatter},
    path,
    sync::atomic::{AtomicI64, Ordering::Relaxed},
};

use actix_web::{
    dev::Service,
    get,
    http::header::{ContentType, ETag, EntityTag, HeaderValue, CACHE_CONTROL, IF_NONE_MATCH},
    middleware::{Compress, Logger, NormalizePath, TrailingSlash},
    post, routes,
    web::{Bytes, Data, Json, Path, Query},
    App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
};

use actix_web_static_files::ResourceFiles;
use anyhow::{anyhow, bail, Context, Error, Result};
use deadpool::unmanaged::Pool as UnmanagedPool;
use deadpool_sqlite::{Config, InteractError, Pool, Runtime};
use once_cell::sync::OnceCell;

use rquickjs::{
    embed, Async, Context as JsContext, Ctx, Func, Function, IntoJs, Object, Promise,
    Result as JsResult, Runtime as JsRuntime, Tokio, Value as JsValue,
};

use rusqlite::Connection;
use serde_json::to_string;

use crate::{
    repository::Repository,
    shared::{DateTime, Item, ItemFilter, ItemHeader},
};

#[embed(name = "main", path = "./web/build/server")]
mod server_module {}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

const VERSION: &str = env!("CARGO_PKG_VERSION");

static DB_POOL: OnceCell<Pool> = OnceCell::new();
static LAST_ITEM_ID: AtomicI64 = AtomicI64::new(0);

pub enum FetchDataResult {
    Data(String),
    Error(String),
}

impl<'js> IntoJs<'js> for FetchDataResult {
    fn into_js(self, ctx: Ctx<'js>) -> JsResult<JsValue<'js>> {
        let obj = Object::new(ctx)?;

        match self {
            Self::Data(data) => obj.set("data", data),
            Self::Error(error) => obj.set("error", error),
        }?;

        Ok(JsValue::from_object(obj))
    }
}

struct Js {
    runtime: JsRuntime,
    context: JsContext,
}

impl Js {
    async fn idle(&self) {
        self.runtime.idle().await;
    }

    fn new() -> Result<Self> {
        let runtime = JsRuntime::new()?;
        let context = JsContext::full(&runtime)?;

        runtime.set_loader(SERVER_MODULE, SERVER_MODULE);
        runtime.spawn_executor(Tokio);

        context.with(|ctx| {
            let globals = ctx.globals();

            let module = ctx.compile(
                "server",
                "import { render } from 'main'; export { render };",
            )?;

            let fetch_data = Func::from(Async(fetch_data));
            let render: Function = module.get("render")?;

            globals.set("fetchData", fetch_data)?;
            globals.set("render", render)
        })?;

        Ok(Self { runtime, context })
    }

    fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Ctx) -> R,
    {
        self.context.with(f)
    }

    fn run_gc(&self) {
        self.runtime.run_gc();
    }
}

type JsPool = UnmanagedPool<Js>;
type Response<T> = Result<T, ServerError>;

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

async fn call_db<F, T, E>(db_pool: &Pool, f: F) -> Response<T>
where
    F: FnOnce(&mut Connection) -> Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Send + Into<Error> + 'static,
{
    db_pool
        .get()
        .await
        .map_err(Error::new)?
        .interact(f)
        .await?
        .map_err(|err| ServerError(err.into()))
}

async fn fetch_data(path: String, search: String) -> FetchDataResult {
    async fn get_data(path: String, search: String) -> Result<String> {
        let db = {
            let db_pool = DB_POOL.get().ok_or_else(|| anyhow!("cannot get db pool"))?;
            db_pool.get().await?
        };

        if let Some(id) = path.strip_prefix("/api/item/") {
            let id: i64 = id.parse()?;

            let item = db
                .interact(move |db| db.get_item(id))
                .await
                .map_err(map_to_anyhow_error)??;

            to_string(&item).map_err(Into::into)
        } else if path == "/api/items" {
            let filter = Query::<ItemFilter>::from_query(&search)?.0;

            let items = db
                .interact(move |db| db.get_items(&filter))
                .await
                .map_err(map_to_anyhow_error)??;

            to_string(&items).map_err(Into::into)
        } else {
            bail!("wrong path {path}")
        }
    }

    let result = get_data(path, search).await;

    match result {
        Ok(data) => FetchDataResult::Data(data),
        Err(error) => FetchDataResult::Error(error.to_string()),
    }
}

fn get_etag(path: &str, tz: Option<String>, use_last_item_id: bool) -> String {
    let mut etag = format!("{VERSION}|{path}|{}", tz.unwrap_or_default());

    if use_last_item_id {
        let id = LAST_ITEM_ID.load(Relaxed);

        etag.push('|');
        etag.push_str(&id.to_string());
    }

    format!("{:x}", md5::compute(etag.as_bytes()))
}

#[routes]
#[get("/")]
#[get("/item/{id}")]
async fn get_html(js_pool: Data<JsPool>, request: HttpRequest) -> Response<impl Responder> {
    let path = get_path(&request);
    let tz = get_tz(&request);
    let etag = get_etag(&path, tz.clone(), request.path() == "/");

    if let Some(expected_etag) = request.headers().get(IF_NONE_MATCH) {
        let expected_etag = expected_etag.as_bytes();

        if expected_etag.len() > 2 && &expected_etag[1..expected_etag.len() - 1] == etag.as_bytes()
        {
            return Ok(HttpResponse::NotModified().finish());
        }
    }

    let js = js_pool.get().await.map_err(Error::new)?;
    let result = render(&js, &path, &tz).await?;

    Ok(HttpResponse::Ok()
        .insert_header(ContentType::html())
        .insert_header(ETag(EntityTag::new_strong(etag)))
        .body(result))
}

#[get("/api/item/{id}")]
async fn get_item(db_pool: Data<Pool>, path: Path<i64>) -> Response<impl Responder> {
    let id = path.into_inner();
    call_db(&db_pool, move |db| db.get_item(id)).await.map(Json)
}

#[get("/api/items")]
async fn get_items(db_pool: Data<Pool>, filter: Query<ItemFilter>) -> Response<impl Responder> {
    call_db(&db_pool, move |db| db.get_items(&filter))
        .await
        .map(Json)
}

fn get_path(request: &HttpRequest) -> String {
    format!(
        "{}://{}{}",
        request.connection_info().scheme(),
        request.connection_info().host(),
        request.uri()
    )
}

fn get_tz(request: &HttpRequest) -> Option<String> {
    request.cookie("tz").map(|tz| tz.value().to_string())
}

fn map_to_anyhow_error(error: InteractError) -> Error {
    anyhow!(match error {
        InteractError::Panic(_) => "panic",
        InteractError::Aborted => "aborted",
    })
}

async fn render(js: &Js, path: &str, tz: &Option<String>) -> Result<String> {
    let result: Promise<String> = js.run(|ctx| {
        let globals = ctx.globals();
        let render: Function = globals.get("render")?;

        globals.set("TIME_ZONE", tz)?;

        render.call((path,))
    })?;

    let result = result.await?;

    js.idle().await;
    js.run_gc();

    Ok(result)
}

#[actix_web::main]
pub async fn start_server(host: &str, port: u16, db: &path::Path) -> Result<()> {
    let db_pool = Config::new(db).create_pool(Runtime::Tokio1)?;

    let last_item_id = db_pool
        .get()
        .await?
        .interact(|db| {
            db.prepare_schema()?;
            db.get_last_item_id()
        })
        .await
        .map_err(map_to_anyhow_error)?
        .with_context(|| format!("cannot prepare database schema in {}", db.display()))?;

    if let Some(last_item_id) = last_item_id {
        LAST_ITEM_ID.store(last_item_id, Relaxed);
    }

    DB_POOL
        .set(db_pool.clone())
        .map_err(|_| anyhow!("cannot set db pool"))?;

    HttpServer::new(move || {
        let mut js = Vec::new();

        for _ in 0..4 {
            js.push(Js::new().expect("cannot init js"));
        }

        let js_pool = UnmanagedPool::from(js);

        App::new()
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(js_pool))
            .service(get_html)
            .service(get_item)
            .service(get_items)
            .service(submit_item)
            .service(ResourceFiles::new("/", generate()).resolve_not_found_to_root())
            .wrap(Compress::default())
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
    db_pool: Data<Pool>,
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

    let id = call_db(&db_pool, move |db| {
        item.update_metadata();
        db.insert_item(&item)
    })
    .await?;

    LAST_ITEM_ID.store(id, Relaxed);

    Ok(Json(id))
}
