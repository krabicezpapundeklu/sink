use std::{fmt::Debug, mem::take, str::FromStr};

use anyhow::Result;
use chrono::{DateTime, Locale, NaiveDateTime, Utc};
use chrono_tz::Tz;
use log::debug;

use rusqlite::{
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

trait Parameter: Debug + ToSql {}

impl<T: Debug + ToSql> Parameter for T {}

pub trait Repository {
    fn get_item(&self, id: i64, tz: &str) -> Result<Item>;
    fn get_items(&self, filter: ItemFilter, tz: &str) -> Result<ItemSearchResult>;
    fn get_last_item_id(&self) -> Result<Option<i64>>;

    fn insert_item(&mut self, item: &Item) -> Result<i64>;

    fn prepare_schema(&self) -> Result<()>;
}

impl Repository for Connection {
    fn get_item(&self, id: i64, tz: &str) -> Result<Item> {
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

        let tz: Tz = tz.parse().unwrap_or(Tz::UTC);

        let sd = DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(&item.submit_date, "%Y-%m-%d %H:%M:%S")?,
            Utc,
        )
        .with_timezone(&tz);

        item.submit_date = sd
            .format_localized("%A, %B %-e, %Y at %l:%M:%S %p (%Z)", Locale::en_US)
            .to_string();

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

    fn get_items(&self, filter: ItemFilter, tz: &str) -> Result<ItemSearchResult> {
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
            parse_query(query, &mut terms);

            sql.push_str(" AND EXISTS (SELECT 1 FROM item_body WHERE item_id = id");

            for term in &terms {
                params.push(term);
                sql.push_str(" AND body LIKE '%' || ? || '%'");
            }

            sql.push_str(") ");
        }

        if let Some(system) = &filter.system {
            params.push(system);
            sql.push_str(" AND system = ?");
        }

        if let Some(r#type) = &filter.r#type {
            params.push(r#type);
            sql.push_str(" AND type = ?");
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
            filter,
        };

        let tz: Tz = tz.parse().unwrap_or(Tz::UTC);
        let today = Utc::now().naive_utc().format("%Y-%m-%d").to_string();

        while let Some(row) = rows.next()? {
            let mut item = ItemSummary {
                id: row.get(0)?,
                submit_date: row.get(1)?,
                system: row.get(2)?,
                r#type: row.get(3)?,
            };

            let format = if item.submit_date.starts_with(&today) {
                "%l:%M %p"
            } else {
                "%-m/%-e/%y %l:%M %p"
            };

            let sd = DateTime::<Utc>::from_utc(
                NaiveDateTime::parse_from_str(&item.submit_date, "%Y-%m-%d %H:%M:%S")?,
                Utc,
            )
            .with_timezone(&tz);

            item.submit_date = sd.format_localized(format, Locale::en_US).to_string();

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
        if c == '"' {
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
