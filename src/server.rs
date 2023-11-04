use std::{
    fmt::{self, Display, Formatter},
    path,
};

use actix_web::{
    dev::Service,
    get,
    http::header::{HeaderValue, CACHE_CONTROL},
    middleware::{Compress, Logger, NormalizePath, TrailingSlash},
    post,
    web::{Bytes, Data, Json, Path, Query},
    App, HttpRequest, HttpServer, Responder, ResponseError,
};

use actix_web_static_files::ResourceFiles;
use anyhow::{anyhow, Context, Error, Result};
use chrono::Utc;
use deadpool_sqlite::{Config, InteractError, Pool, Runtime};
use rusqlite::Connection;

use crate::{
    repository::Repository,
    shared::{Item, ItemFilter, ItemHeader},
};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

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

#[get("/api/item/{id}")]
async fn get_item(
    db_pool: Data<Pool>,
    path: Path<i64>,
) -> Response<impl Responder> {
    let id = path.into_inner();
    let item = call_db(&db_pool, move |db| db.get_item(id)).await?;

    Ok(Json(item))
}

#[get("/api/items")]
async fn get_items(
    db_pool: Data<Pool>,
    filter: Query<ItemFilter>,
) -> Response<impl Responder> {
    let filter = filter.into_inner();
    let items = call_db(&db_pool, move |db| db.get_items(&filter)).await?;

    Ok(Json(items))
}

fn map_to_anyhow_error(error: InteractError) -> Error {
    anyhow!(match error {
        InteractError::Panic(_) => "panic",
        InteractError::Aborted => "aborted",
    })
}

#[actix_web::main]
pub async fn start_server(host: &str, port: u16, db: &path::Path) -> Result<()> {
    let db_pool = Config::new(db).create_pool(Runtime::Tokio1)?;

    db_pool
        .get()
        .await?
        .interact(|db| db.prepare_schema())
        .await
        .map_err(map_to_anyhow_error)?
        .with_context(|| format!("cannot prepare database schema in {}", db.display()))?;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
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

    item.update_metadata();

    let id = call_db(&db_pool, move |db| db.insert_item(&item)).await?;

    Ok(Json(id))
}
