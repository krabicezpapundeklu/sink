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
use chrono::{DateTime, Datelike, Locale, NaiveDateTime, Utc};
use chrono_tz::Tz;
use deadpool::unmanaged::Pool as UnmanagedPool;
use deadpool_sqlite::{Config, InteractError, Pool, Runtime};
use log::{debug, info};

use rquickjs::{
    embed, Async, Context as JsContext, Ctx, Func, Function, IntoJs, Object, Promise,
    Result as JsResult, Runtime as JsRuntime, Tokio, Value as JsValue,
};

use rusqlite::Connection;
use serde_json::to_string;

use crate::{
    repository::Repository,
    shared::{Item, ItemFilter, ItemHeader, ItemSummary},
};

#[embed(name = "main", path = "./web/build/server")]
mod server_module {}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    async fn new() -> Result<Self> {
        let runtime = JsRuntime::new()?;
        let context = JsContext::full(&runtime)?;

        runtime.set_loader(SERVER_MODULE, SERVER_MODULE);
        runtime.spawn_executor(Tokio);

        let result: Promise<()> = context.with(|ctx| {
            let globals = ctx.globals();

            globals.set("debug", Func::from(|message: String| debug!("{message}")))?;

            let module = ctx.compile(
                "server",
                "import { init, render } from 'main'; export { init, render };",
            )?;

            let init: Function = module.get("init")?;
            let render: Function = module.get("render")?;

            globals.set("render", render)?;

            init.call((VERSION,))
        })?;

        result.await?;
        runtime.idle().await;

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

unsafe impl Send for Js {}

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

async fn fetch_data(db_pool: Pool, path: String, search: String, tz: String) -> FetchDataResult {
    async fn get_data(db_pool: Pool, path: String, search: String, tz: String) -> Result<String> {
        let db = db_pool.get().await?;

        let data = if let Some(id) = path.strip_prefix("/api/item/") {
            let id: i64 = id.parse()?;

            let mut item = db
                .interact(move |db| db.get_item(id))
                .await
                .map_err(map_to_anyhow_error)??;

            format_submit_date(&mut item, &tz)?;
            to_string(&item)
        } else if path == "/api/items" {
            let filter = Query::<ItemFilter>::from_query(&search)?.0;

            let mut items = db
                .interact(move |db| db.get_items(&filter))
                .await
                .map_err(map_to_anyhow_error)??;

            format_submit_dates(items.items.as_mut_slice(), &tz)?;
            to_string(&items)
        } else {
            bail!("wrong path {path}")
        }?;

        debug!("get_data END");

        Ok(data)
    }

    let result = get_data(db_pool, path, search, tz).await;

    match result {
        Ok(data) => FetchDataResult::Data(data),
        Err(error) => FetchDataResult::Error(error.to_string()),
    }
}

fn format_submit_date(item: &mut Item, tz: &str) -> Result<()> {
    let tz: Tz = tz.parse().unwrap_or(Tz::UTC);

    let sd = DateTime::<Utc>::from_utc(
        NaiveDateTime::parse_from_str(&item.submit_date, "%Y-%m-%d %H:%M:%S")?,
        Utc,
    )
    .with_timezone(&tz);

    item.submit_date = sd
        .format_localized("%A, %B %-e, %Y at %-l:%M:%S %p (%Z)", Locale::en_US)
        .to_string();

    Ok(())
}

fn format_submit_dates(items: &mut [ItemSummary], tz: &str) -> Result<()> {
    let tz: Tz = tz.parse().unwrap_or(Tz::UTC);
    let today = Utc::now().with_timezone(&tz);

    for item in items {
        let sd = DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(&item.submit_date, "%Y-%m-%d %H:%M:%S")?,
            Utc,
        )
        .with_timezone(&tz);

        let format = if sd.day() == today.day()
            && sd.month() == today.month()
            && sd.year() == today.year()
        {
            "%-l:%M %p"
        } else {
            "%-m/%-e/%y %-l:%M %p"
        };

        item.submit_date = sd.format_localized(format, Locale::en_US).to_string();
    }

    Ok(())
}

fn get_etag(path: &str, tz: &str, use_last_item_id: bool) -> String {
    let mut etag = format!("{VERSION}|{path}|{tz}");

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
async fn get_html(
    db_pool: Data<Pool>,
    js_pool: Data<JsPool>,
    request: HttpRequest,
) -> Response<impl Responder> {
    let path = get_path(&request);
    let tz = get_tz(&request).unwrap_or_default();
    let etag = get_etag(&path, &tz, request.path() == "/");

    if let Some(expected_etag) = request.headers().get(IF_NONE_MATCH) {
        let expected_etag = expected_etag.as_bytes();

        if expected_etag.len() > 2 && &expected_etag[1..expected_etag.len() - 1] == etag.as_bytes()
        {
            return Ok(HttpResponse::NotModified().finish());
        }
    }

    let js = js_pool.get().await.map_err(Error::new)?;
    let result = render(db_pool.get_ref().clone(), &js, &path, tz).await?;

    Ok(HttpResponse::Ok()
        .insert_header(ContentType::html())
        .insert_header(ETag(EntityTag::new_strong(etag)))
        .body(result))
}

#[get("/api/item/{id}")]
async fn get_item(
    db_pool: Data<Pool>,
    path: Path<i64>,
    request: HttpRequest,
) -> Response<impl Responder> {
    let id = path.into_inner();
    let tz = get_tz(&request).unwrap_or_default();

    let mut item = call_db(&db_pool, move |db| db.get_item(id)).await?;

    format_submit_date(&mut item, &tz)?;

    Ok(Json(item))
}

#[get("/api/items")]
async fn get_items(
    db_pool: Data<Pool>,
    filter: Query<ItemFilter>,
    request: HttpRequest,
) -> Response<impl Responder> {
    let filter = filter.into_inner();
    let tz = get_tz(&request).unwrap_or_default();

    let mut items = call_db(&db_pool, move |db| db.get_items(&filter)).await?;

    format_submit_dates(items.items.as_mut_slice(), &tz)?;

    Ok(Json(items))
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

async fn render(db_pool: Pool, js: &Js, path: &str, tz: String) -> Result<String> {
    debug!("render START");

    let result: Promise<String> = js.run(|ctx| {
        let globals = ctx.globals();

        let fetch_data = Func::from(Async(move |path, search| {
            fetch_data(db_pool.clone(), path, search, tz.clone())
        }));

        let render: Function = globals.get("render")?;

        globals.set("fetchData", fetch_data)?;

        render.call((path,))
    })?;

    let result = result.await?;

    js.idle().await;

    debug!("run_gc START");

    js.run_gc();

    debug!("run_gc END");
    debug!("render END");

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

    let js_pool_size = num_cpus::get();
    let mut js = Vec::with_capacity(js_pool_size);

    for i in 0..js_pool_size {
        info!("creating js runtime ({} of {js_pool_size})", i + 1);
        js.push(Js::new().await?);
    }

    let js_pool = UnmanagedPool::from(js);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(js_pool.clone()))
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
        submit_date: Utc::now()
            .naive_utc()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
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
