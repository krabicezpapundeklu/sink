use std::path::PathBuf;

use anyhow::{Context, Error, Result, anyhow};
use axum::{body::Bytes, extract::Query, http::Uri};
use const_format::concatcp;
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use regex::bytes::{Regex, RegexSet};
use rusqlite::Connection;
use serde::{Deserialize, Serialize, Serializer};
use tokio::task::spawn_blocking;

use crate::repository::Repository;

pub const BASE: &str = "/sink";

#[derive(Clone)]
pub struct AppContext {
    pub db_pool: DbPool,
    pub item_types: Vec<(String, usize)>,
    pub item_type_patterns: RegexSet,
    pub system_pattern: Regex,
    pub initial_data_pattern: Regex,
    pub entity_event_id_pattern: Regex,
}

impl AppContext {
    pub async fn call_db<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let mut db = self.get_db().await?;

        spawn_blocking(move || f(&mut db)).await?
    }

    pub async fn get_all_item_ids(&self) -> Result<Vec<i64>> {
        self.call_db(move |db| db.get_all_item_ids()).await
    }

    pub async fn get_db(&self) -> Result<Object<DbManager>> {
        self.db_pool
            .get()
            .await
            .map_err(|error| anyhow!("cannot get db: {error}"))
    }

    pub fn get_entity_event_id(&self, body: &[u8]) -> Option<i64> {
        if let Some(captures) = self.entity_event_id_pattern.captures(body) {
            if let Some(group) = captures.get(1) {
                let entity_event_id = String::from_utf8_lossy(group.as_bytes());

                if let Ok(entity_event_id) = entity_event_id.parse::<i64>() {
                    return Some(entity_event_id);
                }
            }
        }

        None
    }

    pub fn get_event_id(headers: &[ItemHeader]) -> Option<i64> {
        for header in headers {
            if header.name == "mgs-event-id" {
                let event_id = String::from_utf8_lossy(&header.value);

                if let Ok(event_id) = event_id.parse::<i64>() {
                    return Some(event_id);
                }
            }
        }

        None
    }

    pub async fn get_initial_data(&self, uri: &Uri) -> Result<Option<(String, String)>> {
        let path = uri.path();

        if path == concatcp!(BASE, '/') {
            let mut initial_uri = format!("{BASE}/api/items?batchSize=51&loadFirstItem=true");

            if let Some(query) = uri.query() {
                initial_uri.push('&');
                initial_uri.push_str(query);
            }

            let filter: Query<ItemFilter> = Query::try_from_uri(&initial_uri.parse()?)?;
            let items = self.get_items(filter.0).await?;

            Ok(Some((initial_uri, serde_json::to_string(&items)?)))
        } else {
            let id = path.strip_prefix(concatcp!(BASE, "/item/"));

            if let Some(id) = id {
                let id: i64 = id.parse()?;
                let item = self.get_item(id).await?;

                Ok(Some((
                    format!("{BASE}/api/item/{id}"),
                    serde_json::to_string(&item)?,
                )))
            } else {
                Ok(None)
            }
        }
    }

    pub async fn get_item(&self, id: i64) -> Result<Item> {
        self.call_db(move |db| db.get_item(id)).await
    }

    pub fn get_item_type(&self, body: &[u8]) -> Option<String> {
        let matches = self.item_type_patterns.matches(body);

        if matches.matched_any() {
            let mut i = 0;

            'next_item_type: for (key, patterns) in &self.item_types {
                for j in 0..*patterns {
                    if !matches.matched(i + j) {
                        i += patterns;
                        continue 'next_item_type;
                    }
                }

                return Some(key.to_string());
            }
        }

        None
    }

    pub async fn get_items(&self, filter: ItemFilter) -> Result<ItemSearchResult> {
        self.call_db(move |db| {
            let load_first_item = filter.load_first_item.unwrap_or_default();
            let (items, total_items) = db.get_items(filter)?;

            let first_item = if load_first_item && !items.is_empty() {
                Some(db.get_item(items[0].id)?)
            } else {
                None
            };

            let systems = db.get_systems()?;

            Ok(ItemSearchResult {
                items,
                total_items,
                systems,
                first_item,
            })
        })
        .await
    }

    pub fn get_system(&self, headers: &[ItemHeader], body: &[u8]) -> Option<String> {
        let mut system = headers
            .iter()
            .find(|header| header.is_mgs_system_header())
            .map(|header| String::from_utf8_lossy(&header.value).into_owned());

        if system.is_none() {
            if let Some(captures) = self.system_pattern.captures(body) {
                if let Some(group) = captures.get(1) {
                    system = Some(String::from_utf8_lossy(group.as_bytes()).into_owned());
                }
            }
        }

        system
    }

    pub fn get_user_agent(headers: &[ItemHeader]) -> Option<String> {
        headers
            .iter()
            .filter(|header| header.name == "user-agent")
            .map(|header| String::from_utf8_lossy(&header.value).into_owned())
            .next()
    }

    pub async fn new(db: PathBuf) -> Result<Self> {
        #[derive(Deserialize)]
        struct ItemType {
            key: String,
            matches: Vec<String>,
        }

        let db_manager = DbManager { db };
        let db_pool = DbPool::builder(db_manager).build()?;

        let item_types: Vec<ItemType> = serde_json::from_str(include_str!("../item.types.json"))
            .context("cannot parse item.types.json")?;

        let app_context = Self {
            db_pool,
            item_types: item_types
                .iter()
                .map(|item_type| (item_type.key.to_string(), item_type.matches.len()))
                .collect(),
            item_type_patterns: RegexSet::new(
                item_types
                    .iter()
                    .flat_map(|item_type| item_type.matches.iter()),
            )?,
            system_pattern: Regex::new("<mgsSystem>([^<]+)")?,
            initial_data_pattern: Regex::new(r"<!--\s*%INITIAL_DATA%\s*-->")?,
            entity_event_id_pattern: Regex::new(r#""entityEventId"\s*:\s*(\d+)"#)?,
        };

        app_context
            .call_db(|db| db.prepare_schema())
            .await
            .context("cannot prepare database schema")?;

        Ok(app_context)
    }

    pub async fn submit_item(&self, headers: Vec<ItemHeader>, body: Bytes) -> Result<i64> {
        let system = self.get_system(&headers, &body);
        let item_type = self.get_item_type(&body);
        let event_id = Self::get_event_id(&headers);
        let entity_event_id = self.get_entity_event_id(&body);
        let user_agent = Self::get_user_agent(&headers);

        self.call_db(move |db| {
            let item = NewItem {
                system,
                r#type: item_type,
                event_id,
                entity_event_id,
                user_agent,
                headers,
                body: &body,
            };

            db.insert_item(&item)
        })
        .await
    }

    pub async fn update_item(&self, item: ItemSummary) -> Result<usize> {
        self.call_db(move |db| db.update_item(&item)).await
    }
}

pub struct DbManager {
    db: PathBuf,
}

impl Manager for DbManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> Result<Connection, Error> {
        let db = Connection::open(&self.db)?;
        db.init()?;
        Ok(db)
    }

    async fn recycle(&self, _: &mut Connection, _: &Metrics) -> RecycleResult<Error> {
        Ok(())
    }
}

type DbPool = Pool<DbManager>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(flatten)]
    pub summary: ItemSummary,
    pub headers: Vec<ItemHeader>,

    #[serde(serialize_with = "bytes_as_string")]
    pub body: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFilter {
    pub query: Option<String>,
    pub system: Option<String>,
    pub r#type: Option<String>,
    pub event_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub asc: Option<bool>,
    pub first_item_id: Option<i64>,
    pub last_item_id: Option<i64>,
    pub batch_size: Option<u32>,
    pub load_first_item: Option<bool>,
}

#[derive(Serialize)]
pub struct ItemHeader {
    pub name: String,

    #[serde(serialize_with = "bytes_as_string")]
    pub value: Vec<u8>,
}

impl ItemHeader {
    pub fn is_mgs_system_header(&self) -> bool {
        self.name == "mgs-system-id" || self.name == "mgssystem"
    }
}

impl<N, V> From<(N, V)> for ItemHeader
where
    N: AsRef<str>,
    V: AsRef<[u8]>,
{
    fn from((name, value): (N, V)) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: value.as_ref().into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSearchResult {
    pub items: Vec<ItemSummary>,
    pub total_items: i32,
    pub systems: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_item: Option<Item>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: i64,
    pub submit_date: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_event_id: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

pub struct NewItem<'a> {
    pub system: Option<String>,
    pub r#type: Option<String>,
    pub event_id: Option<i64>,
    pub entity_event_id: Option<i64>,
    pub user_agent: Option<String>,
    pub headers: Vec<ItemHeader>,
    pub body: &'a [u8],
}

fn bytes_as_string<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&String::from_utf8_lossy(bytes))
}
