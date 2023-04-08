use std::{
    fmt::{self, Display, Formatter},
    path,
};

use actix_web::{
    dev::Service,
    get,
    http::header::{ContentType, HeaderValue, CACHE_CONTROL},
    middleware::{Compress, Logger, NormalizePath, TrailingSlash},
    post, routes,
    web::{Bytes, Data, Json, Path, Query},
    App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
};

use actix_web_static_files::ResourceFiles;
use anyhow::{anyhow, Context, Error, Result};
use deadpool_sqlite::{Config, InteractError, Pool, Runtime};

use once_cell::sync::OnceCell;

use rquickjs::{
    bind, embed, Context as JsContext, Ctx, Function, IntoJs, Object, Promise, Result as JsResult,
    Runtime as JsRuntime, Tokio, Value as JsValue,
};

use rusqlite::Connection;

use crate::{
    repository::Repository,
    shared::{DateTime, Item, ItemFilter, ItemHeader},
};

#[embed(name = "main", path = "./web/build/server")]
mod server_module {}

#[bind(object)]
#[quickjs(bare)]
mod server_runtime {
    use actix_web::web::Query;
    use anyhow::{anyhow, bail, Result};
    use serde_json::to_string;

    use super::{FetchDataResult, POOL};
    use crate::repository::Repository;
    use crate::server::map_to_anyhow_error;
    use crate::shared::ItemFilter;

    #[quickjs(rename = "fetchData")]
    pub async fn fetch_data(path: String, search: String) -> FetchDataResult {
        async fn get_data(path: String, search: String) -> Result<String> {
            let db = {
                let pool = POOL.get().ok_or_else(|| anyhow!("cannot get pool"))?;
                pool.get().await?
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
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

static POOL: OnceCell<Pool> = OnceCell::new();

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

            globals.init_def::<ServerRuntime>()?;

            let module = ctx.compile(
                "server",
                "import { render } from 'main'; export { render };",
            )?;

            let render: Function = module.get("render")?;

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

async fn call_db<F, T, E>(pool: &Pool, f: F) -> Response<T>
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

#[routes]
#[get("/")]
#[get("/item/{id}")]
async fn get_html(js: Data<Js>, request: HttpRequest) -> Response<impl Responder> {
    let result = render(&js, &request).await?;

    Ok(HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(result))
}

#[get("/api/item/{id}")]
async fn get_item(pool: Data<Pool>, path: Path<i64>) -> Response<impl Responder> {
    let id = path.into_inner();
    call_db(&pool, move |db| db.get_item(id)).await.map(Json)
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

async fn render(js: &Js, request: &HttpRequest) -> Result<String> {
    let path = format!(
        "{}://{}{}",
        request.connection_info().scheme(),
        request.connection_info().host(),
        request.uri()
    );

    let result: Promise<String> = js.run(|ctx| {
        let globals = ctx.globals();
        let render: Function = globals.get("render")?;

        render.call((path,))
    })?;

    let result = result.await?;

    js.idle().await;
    js.run_gc();

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

    POOL.set(pool.clone())
        .map_err(|_| anyhow!("cannot set pool"))?;

    HttpServer::new(move || {
        let js = Js::new().expect("cannot init js");

        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(js))
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
