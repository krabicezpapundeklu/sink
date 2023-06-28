use std::{fmt::Debug, mem::take, str::FromStr, string::ToString, sync::Arc};

use anyhow::Result;
use log::debug;

use regex::bytes::Regex;

use rusqlite::{
    functions::FunctionFlags,
    params, params_from_iter,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
    Connection, Error, ToSql,
};

use crate::shared::{Item, ItemFilter, ItemHeader, ItemSearchResult, ItemSummary, ItemType};

macro_rules! store_as_sql_text {
    ($($type:ty)+) => {
        $(
            impl FromSql for $type {
                fn column_result(value: ValueRef) -> FromSqlResult<Self> {
                    Self::from_str(value.as_str()?).map_err(|e| FromSqlError::Other(e.into()))
                }
            }

            impl ToSql for $type {
                fn to_sql(&self) -> Result<ToSqlOutput, Error> {
                    Ok(ToSqlOutput::Owned(Value::Text(self.to_string())))
                }
            }
        )+
    }
}

store_as_sql_text! {
    ItemType
}

const REGEX_PREFIX: &'static str = "regex:";

trait Parameter: Debug + ToSql {}

impl<T: Debug + ToSql> Parameter for T {}

pub trait Repository {
    fn get_item(&self, id: i64) -> Result<Item>;
    fn get_items(&self, filter: &ItemFilter) -> Result<ItemSearchResult>;
    fn get_last_item_id(&self) -> Result<Option<i64>>;

    fn init(&self) -> Result<()>;

    fn insert_item(&mut self, item: &Item) -> Result<i64>;

    fn prepare_schema(&self) -> Result<()>;
}

impl Repository for Connection {
    fn get_item(&self, id: i64) -> Result<Item> {
        debug!("get_item START");

        let mut item = self.query_row("SELECT i.id, i.submit_date, i.system, i.type, ib.body FROM item i JOIN item_body ib ON ib.item_id = i.id WHERE i.id = ?", [id], |row| {
            Ok(Item {
                id: Some(row.get(0)?),
                submit_date: row.get(1)?,
                system: row.get(2)?,
                r#type: row.get(3)?,
                headers: Vec::new(),
                body: row.get(4)?
            })
        })?;

        let mut stmt = self.prepare(
            "SELECT name, value FROM item_header WHERE item_id = ? ORDER BY name, value",
        )?;

        let mut rows = stmt.query([id])?;

        while let Some(row) = rows.next()? {
            let header = ItemHeader {
                name: row.get(0)?,
                value: row.get(1)?,
            };

            item.headers.push(header);
        }

        debug!("get_item END");

        Ok(item)
    }

    fn get_items(&self, filter: &ItemFilter) -> Result<ItemSearchResult> {
        debug!("get_items START");

        let mut params: Vec<&dyn Parameter> = Vec::new();

        let mut sql = "
            SELECT id, submit_date, system, type, total_items FROM (
                SELECT id, submit_date, system, type, COUNT(1) OVER () total_items FROM item i
                WHERE 1 = 1
        "
        .to_string();

        let mut terms = Vec::new();

        if let Some(query) = &filter.query {
            sql.push_str(" AND EXISTS (SELECT 1 FROM item_body WHERE item_id = id");

            parse_query(query, &mut terms);

            for term in &terms {
                params.push(term);

                if term.starts_with(REGEX_PREFIX) {
                    sql.push_str(" AND matches(?, body)");
                } else {
                    sql.push_str(" AND body LIKE '%' || ? || '%'");
                }
            }

            sql.push_str(") ");
        }

        let systems: Vec<String> = if let Some(system) = &filter.system {
            system.split(',').map(ToString::to_string).collect()
        } else {
            vec![]
        };

        if !systems.is_empty() {
            sql.push_str(" AND system IN (");

            for (i, system) in systems.iter().enumerate() {
                params.push(system);

                if i > 0 {
                    sql.push_str(", ");
                }

                sql.push('?');
            }

            sql.push(')');
        }

        let types: Vec<String> = if let Some(r#type) = &filter.r#type {
            r#type.split(',').map(ToString::to_string).collect()
        } else {
            vec![]
        };

        if !types.is_empty() {
            sql.push_str(" AND type IN (");

            for (i, r#type) in types.iter().enumerate() {
                params.push(r#type);

                if i > 0 {
                    sql.push_str(", ");
                }

                sql.push('?');
            }

            sql.push(')');
        }

        if let Some(from) = &filter.from {
            params.push(from);
            sql.push_str(" AND submit_date >= ?");
        }

        if let Some(to) = &filter.to {
            params.push(to);
            sql.push_str(" AND submit_date <= ?");
        }

        sql.push_str(") WHERE 1 = 1");

        if let Some(first_item_id) = &filter.first_item_id {
            params.push(first_item_id);
            sql.push_str(" AND id >= ?");
        }

        if let Some(last_item_id) = &filter.last_item_id {
            params.push(last_item_id);
            sql.push_str(" AND id <= ?");
        }

        let asc = filter.asc.unwrap_or(false);

        sql.push_str(" ORDER BY id ");
        sql.push_str(if asc { "ASC" } else { "DESC" });

        if let Some(batch_size) = &filter.batch_size {
            params.push(batch_size);
            sql.push_str(" LIMIT ? + 1");
        }

        debug!("sql: {sql}, params: {params:?}");

        let mut stmt = self.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params.iter()))?;

        let mut result = ItemSearchResult {
            items: Vec::new(),
            systems: Vec::new(),
            total_items: 0,
        };

        while let Some(row) = rows.next()? {
            let item = ItemSummary {
                id: row.get(0)?,
                submit_date: row.get(1)?,
                system: row.get(2)?,
                r#type: row.get(3)?,
            };

            result.items.push(item);
            result.total_items = row.get(4)?;
        }

        let mut stmt = self
            .prepare("SELECT DISTINCT system FROM item WHERE system IS NOT NULL ORDER BY system")?;

        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            result.systems.push(row.get(0)?);
        }

        debug!("get_items END");

        Ok(result)
    }

    fn init(&self) -> Result<()> {
        self.create_scalar_function("matches", 2, FunctionFlags::SQLITE_DETERMINISTIC, |ctx| {
            type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

            let regex: Arc<Regex> = ctx.get_or_create_aux(0, |vr| -> Result<_, BoxError> {
                let regex = vr.as_str()?;
                Ok(Regex::new(
                    regex.strip_prefix(REGEX_PREFIX).unwrap_or(regex),
                )?)
            })?;

            let is_match = {
                let text = ctx
                    .get_raw(1)
                    .as_bytes()
                    .map_err(|e| Error::UserFunctionError(e.into()))?;

                regex.is_match(text)
            };

            Ok(is_match)
        })?;

        Ok(())
    }

    fn insert_item(&mut self, item: &Item) -> Result<i64> {
        debug!("insert_item START");

        let tx = self.transaction()?;

        tx.execute(
            "INSERT INTO item (id, submit_date, system, type) VALUES (?, ?, ?, ?)",
            params![item.id, item.submit_date, item.system, item.r#type],
        )?;

        let id = tx.last_insert_rowid();

        for header in &item.headers {
            tx.execute(
                "INSERT INTO item_header (item_id, name, value) VALUES (?, ?, ?)",
                params![id, header.name, header.value],
            )?;
        }

        tx.execute(
            "INSERT INTO item_body (item_id, body) VALUES (?, ?)",
            params![id, item.body],
        )?;

        tx.commit()?;

        debug!("insert_item END");

        Ok(id)
    }

    fn get_last_item_id(&self) -> Result<Option<i64>> {
        debug!("get_last_item_id START");
        let id = self.query_row("SELECT MAX(id) FROM item", [], |row| row.get(0))?;
        debug!("get_last_item_id END");

        Ok(id)
    }

    fn prepare_schema(&self) -> Result<()> {
        debug!("prepare_schema START");

        self.execute_batch("
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS item (id INTEGER PRIMARY KEY AUTOINCREMENT, submit_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, system TEXT, type TEXT) STRICT;
            CREATE TABLE IF NOT EXISTS item_body (item_id INTEGER PRIMARY KEY REFERENCES item (id) ON DELETE CASCADE, body BLOB NOT NULL) STRICT;
            CREATE TABLE IF NOT EXISTS item_header (item_id INTEGER REFERENCES item (id) ON DELETE CASCADE, name TEXT NOT NULL, value BLOB NOT NULL) STRICT;

            CREATE INDEX IF NOT EXISTS idx_item_submit_date ON item (submit_date);
            CREATE INDEX IF NOT EXISTS idx_item_system ON item (system);
            CREATE INDEX IF NOT EXISTS idx_item_type ON item (type);
            CREATE INDEX IF NOT EXISTS idx_item_header_item_id ON item_header (item_id);
        ")?;

        debug!("prepare_schema END");

        Ok(())
    }
}

fn parse_query(query: &str, terms: &mut Vec<String>) {
    let mut term = String::new();

    let mut escaping = false;
    let mut in_quotes = false;

    for c in query.chars() {
        if c == '"' && !term.starts_with(REGEX_PREFIX) {
            if escaping {
                escaping = false;
                term.push('"');
            } else {
                escaping = true;
            }

            continue;
        }

        if escaping {
            escaping = false;
            in_quotes = !in_quotes;
        }

        if !in_quotes && c.is_whitespace() {
            if !term.is_empty() {
                terms.push(take(&mut term));
            }

            continue;
        }

        term.push(c);
    }

    if !term.is_empty() {
        terms.push(term);
    }
}
