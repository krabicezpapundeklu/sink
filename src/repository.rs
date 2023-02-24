use std::{fmt::Debug, mem::take, str::FromStr};

use anyhow::Result;
use log::debug;

use rusqlite::{
    params, params_from_iter,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
    Connection, Error, ToSql,
};

use crate::shared::{
    DateTime, Item, ItemFilter, ItemHeader, ItemSearchResult, ItemSummary, ItemType,
};

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
    DateTime
    ItemType
}

trait Parameter: Debug + ToSql {}

impl<T: Debug + ToSql> Parameter for T {}

pub trait Repository {
    fn get_item(&self, id: i64) -> Result<Item>;
    fn get_items(&self, filter: &ItemFilter) -> Result<ItemSearchResult>;

    fn insert_item(&mut self, item: &Item) -> Result<i64>;
    fn insert_item_no_tx(&self, item: &Item) -> Result<i64>;

    fn prepare_schema(&self) -> Result<()>;
}

impl Repository for Connection {
    fn get_item(&self, id: i64) -> Result<Item> {
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

        sql.push_str(") ");

        let asc = filter.asc.unwrap_or(false);

        if filter.next_item_id > 0 {
            params.push(&filter.next_item_id);

            sql.push_str(" WHERE id ");
            sql.push_str(if asc { ">=" } else { "<=" });
            sql.push_str("? ");
        }

        params.push(&filter.batch_size);

        sql.push_str(" ORDER BY id ");
        sql.push_str(if asc { "ASC" } else { "DESC" });
        sql.push_str(" LIMIT ? + 1");

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

    fn insert_item(&mut self, item: &Item) -> Result<i64> {
        let tx = self.transaction()?;
        let id = tx.insert_item_no_tx(item)?;
        tx.commit()?;
        Ok(id)
    }

    fn insert_item_no_tx(&self, item: &Item) -> Result<i64> {
        self.execute(
            "INSERT INTO item (id, submit_date, system, type) VALUES (?, ?, ?, ?)",
            params![item.id, item.submit_date, item.system, item.r#type],
        )?;

        let id = self.last_insert_rowid();

        for header in &item.headers {
            self.execute(
                "INSERT INTO item_header (item_id, name, value) VALUES (?, ?, ?)",
                params![id, header.name, header.value],
            )?;
        }

        self.execute(
            "INSERT INTO item_body (item_id, body) VALUES (?, ?)",
            params![id, item.body],
        )?;

        Ok(id)
    }

    fn prepare_schema(&self) -> Result<()> {
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
        ").map_err(Into::into)
    }
}

fn parse_query(query: &str, terms: &mut Vec<String>) {
    enum State {
        Initial,
        InQuotes,
        Quote,
    }

    let mut term = String::new();
    let mut state = State::Initial;

    for c in query.chars() {
        if c == '"' {
            match state {
                State::Initial => {
                    state = State::Quote;
                    continue;
                }
                State::InQuotes => {
                    state = State::Initial;
                    continue;
                }
                State::Quote => {
                    state = State::Initial;
                }
            }
        } else if c.is_whitespace() {
            if let State::Initial = state {
                if !term.is_empty() {
                    terms.push(take(&mut term));
                }

                continue;
            }
        } else if let State::Quote = state {
            state = State::InQuotes;
        }

        term.push(c);
    }

    if !term.is_empty() {
        terms.push(term);
    }
}
