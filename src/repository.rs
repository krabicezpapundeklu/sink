use std::{borrow::Cow, future::Future, mem::take, path::Path};

use crate::model::{Item, ItemFilter, ItemHeader, ItemSummary, NewItem};

use anyhow::Result;
use futures::stream::StreamExt;
use regex::bytes::Regex;
use rusqlite::{Connection, functions::FunctionFlags};

use sqlx::{
    Database, Encode, QueryBuilder, Sqlite, SqliteConnection, SqlitePool, Type, migrate,
    prelude::FromRow,
    query, query_as, query_scalar,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

trait QueryBuilderExt<'a, DB: Database> {
    fn append_if_is_some<T>(&mut self, sql: &str, value: Option<T>) -> &mut Self
    where
        T: 'a + Encode<'a, DB> + Type<DB>;

    fn append_in<T>(&mut self, items: impl Iterator<Item = T>) -> &mut Self
    where
        T: 'a + Encode<'a, DB> + Type<DB>;
}

impl<'a, DB: Database> QueryBuilderExt<'a, DB> for QueryBuilder<'a, DB> {
    fn append_if_is_some<T>(&mut self, sql: &str, value: Option<T>) -> &mut Self
    where
        T: 'a + Encode<'a, DB> + Type<DB>,
    {
        if let Some(value) = value {
            self.push(sql).push_bind(value);
        }

        self
    }

    fn append_in<T>(&mut self, items: impl Iterator<Item = T>) -> &mut Self
    where
        T: 'a + Encode<'a, DB> + Type<DB>,
    {
        self.push(" IN (");

        let mut separated = self.separated(", ");

        for item in items {
            separated.push_bind(item);
        }

        separated.push_unseparated(")");

        self
    }
}

#[derive(Debug, Eq, PartialEq)]
enum QueryExpression<'a> {
    EventId(&'a str),
    Header(&'a str, &'a str),
    Id(&'a str),
    Regex(&'a str),
    Text(&'a str),
}

impl<'a> From<&'a str> for QueryExpression<'a> {
    fn from(value: &'a str) -> Self {
        if let Some((name, value)) = value.split_once(':') {
            match name {
                "body" => QueryExpression::Text(value),
                "event-id" => QueryExpression::EventId(value),
                "id" => QueryExpression::Id(value),
                "regex" => QueryExpression::Regex(value),
                _ => QueryExpression::Header(name, value),
            }
        } else {
            QueryExpression::Text(value)
        }
    }
}

pub trait Repository: Clone + Send + Sync {
    fn get_item(&self, id: i64) -> impl Future<Output = Result<Option<Item>>> + Send;

    fn get_items(
        &self,
        filter: &ItemFilter,
    ) -> impl Future<Output = Result<(Vec<ItemSummary>, i32)>> + Send;

    fn get_systems(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
    fn insert_item(&self, item: &NewItem<'_>) -> impl Future<Output = Result<i64>> + Send;
}

impl Repository for SqlitePool {
    async fn get_item(&self, id: i64) -> Result<Option<Item>> {
        let summary = query_as!(
            ItemSummary,
            "SELECT id, system, type, event_id, entity_event_id, user_agent, submit_date FROM item WHERE id = ?",
            id
        ).fetch_optional(self).await?;

        let item = if let Some(summary) = summary {
            let headers = query_as!(
                ItemHeader,
                "SELECT name, value FROM item_header WHERE item_id = ? ORDER BY name, value",
                id
            )
            .fetch_all(self)
            .await?;

            let body = query_scalar!("SELECT body FROM item_body WHERE item_id = ?", id)
                .fetch_one(self)
                .await?;

            let item = Item {
                summary,
                headers,
                body,
            };

            Some(item)
        } else {
            None
        };

        Ok(item)
    }

    async fn get_items(&self, filter: &ItemFilter) -> Result<(Vec<ItemSummary>, i32)> {
        #[derive(FromRow)]
        struct Row {
            #[sqlx(flatten)]
            item: ItemSummary,
            total_items: i32,
        }

        let mut builder = QueryBuilder::<Sqlite>::new(
            "SELECT * FROM (SELECT id, system, type, event_id, entity_event_id, user_agent, submit_date, COUNT(1) OVER() total_items FROM item WHERE 1 = 1",
        );

        let query_tokens;

        if let Some(query) = &filter.query {
            query_tokens = tokenize_query(query);

            builder.push(" AND (1 = 1");

            for expression in query_tokens
                .iter()
                .map(|token| QueryExpression::from(token.as_ref()))
            {
                builder.push(" AND ");

                match expression {
                    QueryExpression::EventId(event_id) =>
                        builder
                            .push("event_id = ")
                            .push_bind(event_id),
                    QueryExpression::Header(name, value) =>
                        builder
                            .push("EXISTS (SELECT 1 FROM item_header WHERE item_id = id AND name = ")
                            .push_bind(name)
                            .push(" AND value LIKE '%' || ")
                            .push_bind(value)
                            .push(" || '%')"),
                    QueryExpression::Id(id) =>
                        builder
                            .push("id = ")
                            .push_bind(id),
                    QueryExpression::Regex(regex) =>
                        builder
                            .push("EXISTS (SELECT 1 FROM item_body WHERE item_id = id AND matches(")
                            .push_bind(regex)
                            .push(", body))"),
                    QueryExpression::Text(text) =>
                        builder
                            .push("EXISTS (SELECT 1 FROM item_body WHERE item_id = id AND body LIKE '%' || ")
                            .push_bind(text)
                            .push(" || '%')")
                };
            }

            builder.push(')');
        }

        if let Some(systems) = &filter.system {
            builder
                .push(" AND system")
                .append_in(systems.comma_separated());
        }

        if let Some(types) = &filter.r#type {
            builder.push(" AND type").append_in(types.comma_separated());
        }

        if let Some(event_types) = &filter.event_type {
            builder
                .push(
                    " AND (type NOT IN ('event_notification', 'event_payload') OR entity_event_id",
                )
                .append_in(event_types.comma_separated())
                .push(')');
        }

        builder
            .append_if_is_some(" AND submit_date >= ", filter.from.as_ref())
            .append_if_is_some(" AND submit_date <= ", filter.to.as_ref())
            .push(") WHERE 1 = 1")
            .append_if_is_some(" AND id >= ", filter.first_item_id)
            .append_if_is_some(" AND id <= ", filter.last_item_id)
            .push(" ORDER BY id");

        if !filter.asc.unwrap_or_default() {
            builder.push(" DESC");
        }

        builder.append_if_is_some(" LIMIT ", filter.batch_size);

        let query = builder.build_query_as::<Row>();
        let mut rows = query.fetch(self);

        let mut items = Vec::new();
        let mut total_items = 0;

        while let Some(row) = rows.next().await {
            let row = row?;
            items.push(row.item);
            total_items = row.total_items;
        }

        Ok((items, total_items))
    }

    async fn get_systems(&self) -> Result<Vec<String>> {
        query_scalar!(
            "SELECT DISTINCT system AS 'system!' FROM item WHERE system IS NOT NULL ORDER BY system"
        )
        .fetch_all(self)
        .await
        .map_err(Into::into)
    }

    async fn insert_item(&self, item: &NewItem<'_>) -> Result<i64> {
        let mut tx = self.begin().await?;

        let id = query!(
            "INSERT INTO item (system, type, event_id, entity_event_id, user_agent) VALUES (?, ?, ?, ?, ?)",
            item.system,
            item.r#type,
            item.event_id,
            item.entity_event_id,
            item.user_agent
        )
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

        for header in item.headers {
            query!(
                "INSERT INTO item_header (item_id, name, value) VALUES (?, ?, ?)",
                id,
                header.name,
                header.value
            )
            .execute(&mut *tx)
            .await?;
        }

        query!(
            "INSERT INTO item_body (item_id, body) VALUES (?, ?)",
            id,
            item.body
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(id)
    }
}

trait SplitExt {
    fn comma_separated(&self) -> impl Iterator<Item = &str>;
}

impl SplitExt for str {
    fn comma_separated(&self) -> impl Iterator<Item = &Self> {
        self.split(',').map(Self::trim)
    }
}

pub async fn open_repository(db: impl AsRef<Path> + Send) -> Result<impl Repository> {
    let pool = SqlitePoolOptions::new()
        .after_connect(|connection, _| {
            Box::pin(async move {
                unsafe {
                    register_match_function(connection)
                        .await
                        .map_err(|e| sqlx::Error::Configuration(e.into()))
                }
            })
        })
        .connect_with(
            SqliteConnectOptions::new()
                .filename(db)
                .journal_mode(SqliteJournalMode::Wal)
                .create_if_missing(true),
        )
        .await?;

    migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub async unsafe fn register_match_function(connection: &mut SqliteConnection) -> Result<()> {
    unsafe {
        let mut handle = connection.lock_handle().await?;
        let connection = Connection::from_handle(handle.as_raw_handle().as_mut())?;

        connection
            .create_scalar_function("matches", 2, FunctionFlags::SQLITE_DETERMINISTIC, |ctx| {
                let regex = ctx.get_or_create_aux(0, |vr| -> Result<Regex> {
                    let regex = vr.as_str()?;
                    Regex::new(regex).map_err(Into::into)
                })?;

                let text = ctx.get_raw(1).as_bytes()?;
                Ok(regex.is_match(text))
            })
            .map_err(Into::into)
    }
}

fn tokenize_query(query: &str) -> Vec<Cow<'_, str>> {
    let mut tokens = Vec::new();
    let mut chars = query.char_indices();
    let mut quoted_token = String::new();

    while let Some((start, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        let mut cc = c;

        loop {
            if cc == '"' {
                while let Some((_, c)) = chars.next() {
                    if c == '"' {
                        if let Some((_, '"')) = chars.clone().next() {
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    quoted_token.push(c);
                }

                tokens.push(take(&mut quoted_token).into());
                break;
            }

            let mut end = start;

            for (i, c) in chars.by_ref() {
                if c == '"' || c.is_whitespace() {
                    cc = c;
                    break;
                }

                end = i;
            }

            tokens.push(Cow::from(&query[start..=end]));

            if cc != '"' {
                break;
            }
        }
    }

    tokens
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use mockall::mock;
    use rstest::rstest;

    mock! {
        pub Repository{}

        impl Clone for Repository {
            fn clone(&self) -> Self;
        }

        impl super::Repository for Repository {
            fn get_item(&self, id: i64) -> impl Future<Output = Result<Option<Item>>> + Send;

            fn get_items(
                &self,
                filter: &ItemFilter,
            ) -> impl Future<Output = Result<(Vec<ItemSummary>, i32)>> + Send;

            fn get_systems(&self) -> impl Future<Output = Result<Vec<String>>> + Send;
            fn insert_item<'a>(&self, item: &NewItem<'a>) -> impl Future<Output = Result<i64>> + Send;
        }
    }

    #[test]
    fn test_query_expressions() {
        let tokens = tokenize_query(r#"event-id:123 id:123 regex:abc abc:def abc "abc def""#);

        let mut iter = tokens
            .iter()
            .map(|token| QueryExpression::from(token.as_ref()));

        assert_eq!(iter.next(), Some(QueryExpression::EventId("123")));
        assert_eq!(iter.next(), Some(QueryExpression::Id("123")));
        assert_eq!(iter.next(), Some(QueryExpression::Regex("abc")));
        assert_eq!(iter.next(), Some(QueryExpression::Header("abc", "def")));
        assert_eq!(iter.next(), Some(QueryExpression::Text("abc")));
        assert_eq!(iter.next(), Some(QueryExpression::Text("abc def")));
        assert_eq!(iter.next(), None);
    }

    #[rstest]
    #[case("", &[])]
    #[case(" ", &[])]
    #[case("a", &["a"])]
    #[case(" a ", &["a"])]
    #[case("a b", &["a", "b"])]
    #[case(" a b ", &["a", "b"])]
    #[case("abc", &["abc"])]
    #[case(" abc ", &["abc"])]
    #[case("abc def", &["abc", "def"])]
    #[case(" abc def ", &["abc", "def"])]
    #[case(r#""""#, &[""])]
    #[case(r#""a""#, &["a"])]
    #[case(r#"""#, &[""])]
    #[case(r#""a"#, &["a"])]
    #[case(r#"" a ""#, &[" a "])]
    #[case(r#""abc""#, &["abc"])]
    #[case(r#"" abc def ""#, &[" abc def "])]
    #[case(r#""a" "b""#, &["a", "b"])]
    #[case(r#"a"b"c"#, &["a", "b", "c"])]
    #[case(r#""a"b"#, &["a", "b"])]
    #[case(r#"""a"b"#, &["", "a", "b"])]
    #[case(r#""""a"b"#, &[r#""a"#, "b"])]
    #[case(r#"""""a"b"#, &[r#"""#, "a", "b"])]
    #[case(r#""""""a"b"#, &[r#"""a"#, "b"])]
    #[case(r#""""""#, &[r#"""#])]
    #[case(r#""regex:abc "" def""#, &[r#"regex:abc " def"#])]
    fn test_tokenize_query(#[case] query: &str, #[case] expected_terms: &[&str]) {
        assert_eq!(tokenize_query(query), expected_terms, "query = {query}");
    }
}
