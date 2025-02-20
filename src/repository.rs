use std::mem::take;

use anyhow::Result;
use regex::bytes::Regex;
use rusqlite::{Connection, Row, functions::FunctionFlags, params, params_from_iter, types::Value};
use tracing::trace;

use crate::shared::{Item, ItemFilter, ItemHeader, ItemSummary, NewItem};

const EVENT_ID_PREFIX: &str = "event-id:";
const REGEX_PREFIX: &str = "regex:";

struct QueryBuilder {
    sql: String,
    params: Vec<Value>,
}

impl QueryBuilder {
    fn append_sql(&mut self, sql: &str) -> &mut Self {
        self.sql.push_str(sql);
        self
    }

    fn append_param<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Value>,
    {
        self.params.push(item.into());
        self
    }

    fn append_if_is_some<T>(&mut self, sql: &str, item: Option<T>) -> &mut Self
    where
        T: Into<Value>,
    {
        if let Some(item) = item {
            self.append_sql(sql).append_param(item);
        }

        self
    }

    fn append_list<T>(&mut self, items: impl Iterator<Item = T>) -> &mut Self
    where
        T: Into<Value>,
    {
        for (i, item) in items.enumerate() {
            if i > 0 {
                self.append_sql(", ");
            }

            self.append_sql("?").append_param(item);
        }

        self
    }

    const fn new(sql: String) -> Self {
        Self {
            sql,
            params: Vec::new(),
        }
    }

    fn params(&self) -> &[Value] {
        &self.params
    }

    fn sql(&self) -> &str {
        &self.sql
    }
}

pub trait Repository {
    fn get_all_item_ids(&self) -> Result<Vec<i64>>;
    fn get_item(&self, id: i64) -> Result<Item>;
    fn get_items(&self, filter: ItemFilter) -> Result<(Vec<ItemSummary>, i32)>;
    fn get_systems(&self) -> Result<Vec<String>>;

    fn init(&self) -> Result<()>;

    fn insert_item(&mut self, item: &NewItem) -> Result<i64>;
    fn update_item(&mut self, item: &ItemSummary) -> Result<usize>;

    fn prepare_schema(&self) -> Result<()>;
}

impl Repository for Connection {
    fn get_all_item_ids(&self) -> Result<Vec<i64>> {
        let mut stmt = self.prepare_cached("SELECT id FROM item ORDER BY id")?;

        let mut rows = stmt.query([])?;
        let mut items = Vec::new();

        while let Some(row) = rows.next()? {
            items.push(row.get(0)?);
        }

        Ok(items)
    }

    fn get_item(&self, id: i64) -> Result<Item> {
        let mut stmt = self.prepare_cached(
            "
            SELECT i.id, i.submit_date, i.system, i.type, i.event_id, i.entity_event_id, i.user_agent, ib.body
            FROM item i JOIN item_body ib ON ib.item_id = i.id WHERE i.id = ?
        ",
        )?;

        let mut item = stmt.query_row([id], |row| {
            Ok(Item {
                summary: map_item_summary(row)?,
                headers: Vec::new(),
                body: row.get(7)?,
            })
        })?;

        let mut stmt = self.prepare_cached(
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

    fn get_items(&self, filter: ItemFilter) -> Result<(Vec<ItemSummary>, i32)> {
        let mut builder = QueryBuilder::new(
            "
                SELECT id, submit_date, system, type, event_id, entity_event_id, user_agent, total_items FROM (
                    SELECT id, submit_date, system, type, event_id, entity_event_id, user_agent, COUNT(1) OVER () total_items FROM item i
                    WHERE 1 = 1
            "
            .to_string(),
        );

        if let Some(query) = &filter.query {
            builder.append_sql(" AND ((1 = 1");

            for term in parse_query(query) {
                builder.append_sql(" AND ");

                if term.starts_with(EVENT_ID_PREFIX) {
                    builder.append_sql("event_id = ?");
                    builder.append_param(term.strip_prefix(EVENT_ID_PREFIX).map(ToOwned::to_owned));
                } else if term.starts_with(REGEX_PREFIX) {
                    builder.append_sql(
                        "EXISTS (SELECT 1 FROM item_body WHERE item_id = id AND matches(?, body))",
                    );

                    builder.append_param(term);
                } else {
                    if let Ok(id) = term.parse::<i64>() {
                        builder.append_sql("(id = ? OR event_id = ? OR ");
                        builder.append_param(id);
                        builder.append_param(id);
                    } else if let Some(idx) = term.find(':') {
                        let name = &term[..idx];
                        let value = &term[idx + 1..];

                        builder.append_sql("(EXISTS (SELECT 1 FROM item_header WHERE item_id = id AND name = ? AND value LIKE '%' || ? || '%') OR ");
                        builder.append_param(name.to_owned());
                        builder.append_param(value.to_owned());
                    } else {
                        builder.append_sql("(");
                    }

                    builder.append_sql("EXISTS (SELECT 1 FROM item_body WHERE item_id = id AND body LIKE '%' || ? || '%'))");
                    builder.append_param(term);
                }
            }

            builder.append_sql("))");
        }

        if let Some(system) = &filter.system {
            builder
                .append_sql(" AND system IN (")
                .append_list(system.split(',').map(ToString::to_string))
                .append_sql(")");
        }

        if let Some(r#type) = &filter.r#type {
            builder
                .append_sql(" AND type IN (")
                .append_list(r#type.split(',').map(ToString::to_string))
                .append_sql(")");
        }

        if let Some(event_type) = &filter.event_type {
            builder.append_sql(
                " AND (type NOT IN ('event_notification', 'event_payload') OR entity_event_id IN (",
            ).append_list(event_type.split(',').map(ToString::to_string)).append_sql("))");
        }

        builder
            .append_if_is_some(" AND submit_date >= ?", filter.from)
            .append_if_is_some(" AND submit_date <= ?", filter.to)
            .append_sql(") WHERE 1 = 1")
            .append_if_is_some(" AND id >= ?", filter.first_item_id)
            .append_if_is_some(" AND id <= ?", filter.last_item_id)
            .append_sql(" ORDER BY id");

        if !filter.asc.unwrap_or(false) {
            builder.append_sql(" DESC");
        }

        builder.append_if_is_some(" LIMIT ?", filter.batch_size);

        let sql = builder.sql();
        let params = builder.params();

        trace!(sql = %sql, params = ?params);

        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.query(params_from_iter(params))?;

        let mut items = Vec::new();
        let mut total_items = 0;

        while let Some(row) = rows.next()? {
            items.push(map_item_summary(row)?);
            total_items = row.get(7)?;
        }

        Ok((items, total_items))
    }

    fn get_systems(&self) -> Result<Vec<String>> {
        let mut stmt = self.prepare_cached(
            "SELECT DISTINCT system FROM item WHERE system IS NOT NULL ORDER BY system",
        )?;

        let mut rows = stmt.query([])?;
        let mut systems = Vec::new();

        while let Some(row) = rows.next()? {
            systems.push(row.get(0)?);
        }

        Ok(systems)
    }

    fn init(&self) -> Result<()> {
        self.create_scalar_function("matches", 2, FunctionFlags::SQLITE_DETERMINISTIC, |ctx| {
            let regex = ctx.get_or_create_aux(0, |vr| -> Result<_> {
                let regex = vr.as_str()?;
                Regex::new(regex.strip_prefix(REGEX_PREFIX).unwrap_or(regex)).map_err(Into::into)
            })?;

            let text = ctx.get_raw(1).as_bytes()?;

            Ok(regex.is_match(text))
        })
        .map_err(Into::into)
    }

    fn insert_item(&mut self, item: &NewItem) -> Result<i64> {
        let tx = self.transaction()?;
        let id;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO item (system, type, event_id, entity_event_id, user_agent) VALUES (?, ?, ?, ?, ?)",
            )?;

            id = stmt.insert(params![
                &item.system,
                &item.r#type,
                &item.event_id,
                &item.entity_event_id,
                &item.user_agent
            ])?;

            let mut stmt = tx.prepare_cached(
                "INSERT INTO item_header (item_id, name, value) VALUES (?, ?, ?)",
            )?;

            for header in &item.headers {
                stmt.execute(params![id, header.name, header.value])?;
            }

            let mut stmt =
                tx.prepare_cached("INSERT INTO item_body (item_id, body) VALUES (?, ?)")?;

            stmt.execute(params![id, item.body])?;
        }

        tx.commit()?;

        Ok(id)
    }

    fn prepare_schema(&self) -> Result<()> {
        self.execute_batch("
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS item (id INTEGER PRIMARY KEY AUTOINCREMENT, submit_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, system TEXT, type TEXT) STRICT;
            CREATE TABLE IF NOT EXISTS item_body (item_id INTEGER PRIMARY KEY REFERENCES item (id) ON DELETE CASCADE, body BLOB NOT NULL) STRICT;
            CREATE TABLE IF NOT EXISTS item_header (item_id INTEGER REFERENCES item (id) ON DELETE CASCADE, name TEXT NOT NULL, value BLOB NOT NULL) STRICT;
        ")?;

        if !has_column(self, "item", "event_id")? {
            self.execute("ALTER TABLE item ADD COLUMN event_id INTEGER", [])?;
        }

        if !has_column(self, "item", "entity_event_id")? {
            self.execute("ALTER TABLE item ADD COLUMN entity_event_id INTEGER", [])?;
        }

        if !has_column(self, "item", "user_agent")? {
            self.execute("ALTER TABLE item ADD COLUMN user_agent TEXT", [])?;
        }

        self.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_item_entity_event_id ON item (entity_event_id);
            CREATE INDEX IF NOT EXISTS idx_item_event_id ON item (event_id);
            CREATE INDEX IF NOT EXISTS idx_item_header_item_id ON item_header (item_id);
            CREATE INDEX IF NOT EXISTS idx_item_submit_date ON item (submit_date);
            CREATE INDEX IF NOT EXISTS idx_item_system ON item (system);
            CREATE INDEX IF NOT EXISTS idx_item_type ON item (type);
            CREATE INDEX IF NOT EXISTS idx_item_user_agent ON item (user_agent);
        ",
        )?;

        Ok(())
    }

    fn update_item(&mut self, item: &ItemSummary) -> Result<usize> {
        let mut stmt = self.prepare_cached(
            "UPDATE item SET submit_date = ?, type = ?, system = ?, event_id = ?, entity_event_id = ?, user_agent = ? WHERE id = ?",
        )?;

        stmt.execute(params![
            item.submit_date,
            item.r#type,
            item.system,
            item.event_id,
            item.entity_event_id,
            item.user_agent,
            item.id
        ])
        .map_err(Into::into)
    }
}

fn has_column(db: &Connection, table: &str, column: &str) -> Result<bool> {
    let count: i32 = db.query_row(
        "SELECT COUNT(1) FROM pragma_table_info(?) WHERE name = ?",
        [table, column],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

fn map_item_summary(row: &Row) -> rusqlite::Result<ItemSummary> {
    Ok(ItemSummary {
        id: row.get(0)?,
        submit_date: row.get(1)?,
        system: row.get(2)?,
        r#type: row.get(3)?,
        event_id: row.get(4)?,
        entity_event_id: row.get(5)?,
        user_agent: row.get(6)?,
    })
}

fn parse_query(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut term = String::new();

    let mut escaping = false;
    let mut in_quotes = false;

    for c in query.chars() {
        if c == '"' && !term.starts_with(EVENT_ID_PREFIX) && !term.starts_with(REGEX_PREFIX) {
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

    terms
}
