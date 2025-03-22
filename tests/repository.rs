use anyhow::Result;
use axum::{extract::Query, http::Uri};
use rstest::rstest;

use sink::{
    model::{ItemFilter, NewItem, NewItemHeader},
    repository::{Repository, register_match_function},
};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

#[sqlx::test]
async fn test_get_item_nonexistent(repository: SqlitePool) -> Result<()> {
    let item = repository.get_item(0).await?;
    assert!(item.is_none());
    Ok(())
}

#[rstest]
#[case("", &[5, 4, 3, 2, 1], 5)]
#[case("query=event-id:1", &[1], 1)]
#[case("query=header-1:value-1", &[2, 1], 2)]
#[case("query=header-1:value-1%20header-2:value-2", &[1], 1)]
#[case("query=id:5", &[5], 1)]
#[case("query=body:id:5", &[3], 1)]
#[case("query=regex:body-[12]", &[2, 1], 2)]
#[case("query=body-1", &[1], 1)]
#[case("system=system-1", &[2], 1)]
#[case("system=system-1,system-2", &[3, 2], 2)]
#[case("system=system-xxx", &[], 0)]
#[case("type=type-1", &[3], 1)]
#[case("type=type-1,type-2", &[3, 2], 2)]
#[case("type=type-xxx", &[], 0)]
#[case("eventType=1", &[4, 3, 2], 3)]
#[case("eventType=1,2", &[5, 4, 3, 2], 4)]
#[case("eventType=1&type=event_payload", &[4], 1)]
#[case("eventType=1,2&type=event_payload", &[5, 4], 2)]
#[case("from=2025-01-02", &[5, 4, 3, 2], 4)]
#[case("to=2025-01-02", &[2, 1], 2)]
#[case("firstItemId=2", &[5, 4, 3, 2], 5)]
#[case("lastItemId=2", &[2, 1], 5)]
#[case("asc=true", &[1, 2, 3, 4, 5], 5)]
#[case("asc=false", &[5, 4, 3, 2, 1], 5)]
#[case("batchSize=1", &[5], 5)]
#[case("batchSize=2", &[5, 4], 5)]
#[case("batchSize=3", &[5, 4, 3], 5)]
#[sqlx::test(fixtures("items"))]
async fn test_get_items(
    #[case] filter: &str,
    #[case] expected_item_ids: &[i64],
    #[case] expected_total_items: i32,
    #[ignore] pool_options: SqlitePoolOptions,
    #[ignore] connect_options: SqliteConnectOptions,
) -> Result<()> {
    let repository = pool_options
        .after_connect(|connection, _| {
            Box::pin(async move {
                unsafe {
                    register_match_function(connection).await.unwrap();
                }

                Ok(())
            })
        })
        .connect_with(connect_options)
        .await?;

    let uri: Uri = format!("http://localhost?{filter}").parse()?;
    let filter: Query<ItemFilter> = Query::try_from_uri(&uri)?;
    let (items, total_items) = repository.get_items(&filter).await?;
    let item_ids: Vec<i64> = items.iter().map(|item| item.id).collect();

    assert_eq!(item_ids, expected_item_ids);
    assert_eq!(total_items, expected_total_items);

    Ok(())
}

#[sqlx::test(fixtures("items"))]
async fn test_get_systems(repository: SqlitePool) -> Result<()> {
    let systems = repository.get_systems().await?;
    assert_eq!(systems, &["system-1", "system-2"]);
    Ok(())
}

#[sqlx::test]
async fn test_insert_and_get_item(repository: SqlitePool) -> Result<()> {
    const SYSTEM: Option<&str> = Some("system");
    const TYPE: Option<&str> = Some("type");
    const EVENT_ID: Option<i64> = Some(123);
    const ENTITY_EVENT_ID: Option<i64> = Some(456);
    const USER_AGENT: Option<&str> = Some("user-agent");
    const HEADER_1_NAME: &str = "header-1";
    const HEADER_1_VALUE: &[u8] = b"value-1";
    const HEADER_2_NAME: &str = "header-2";
    const HEADER_2_VALUE: &[u8] = b"value-2";
    const BODY: &[u8] = b"body";

    let new_item = NewItem {
        system: SYSTEM,
        r#type: TYPE,
        event_id: EVENT_ID,
        entity_event_id: ENTITY_EVENT_ID,
        user_agent: USER_AGENT,
        headers: &[
            NewItemHeader {
                name: HEADER_1_NAME,
                value: HEADER_1_VALUE,
            },
            NewItemHeader {
                name: HEADER_2_NAME,
                value: HEADER_2_VALUE,
            },
        ],
        body: BODY,
    };

    let id = repository.insert_item(&new_item).await?;
    let item = repository.get_item(id).await?.unwrap();

    let summary = item.summary;
    let headers = item.headers;

    assert_eq!(summary.id, id);
    assert_eq!(summary.system.as_deref(), SYSTEM);
    assert_eq!(summary.r#type.as_deref(), TYPE);
    assert_eq!(summary.event_id, EVENT_ID);
    assert_eq!(summary.entity_event_id, ENTITY_EVENT_ID);
    assert_eq!(summary.user_agent.as_deref(), USER_AGENT);
    assert!(!summary.submit_date.is_empty());
    assert_eq!(headers.len(), 2);
    assert_eq!(headers[0].name, HEADER_1_NAME);
    assert_eq!(*headers[0].value, *HEADER_1_VALUE);
    assert_eq!(headers[1].name, HEADER_2_NAME);
    assert_eq!(*headers[1].value, *HEADER_2_VALUE);
    assert_eq!(*item.body, *BODY);

    Ok(())
}
