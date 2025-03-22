use anyhow::Result;
use axum::{extract::Query, http::Uri};
use rstest::rstest;

use sink::{
    model::{ItemFilter, NewItemHeader},
    service::{Service, new_service},
};

use sqlx::SqlitePool;

#[rstest]
#[case(1, Some(1))]
#[case(0, None)]
#[sqlx::test(fixtures("items"))]
async fn test_get_item(
    #[case] id: i64,
    #[case] expected_item_id: Option<i64>,
    #[ignore] repository: SqlitePool,
) -> Result<()> {
    let service = new_service(repository)?;
    let item = service.get_item(id).await?;

    assert_eq!(item.map(|item| item.summary.id), expected_item_id);

    Ok(())
}

#[rstest]
#[case("", &[5, 4, 3, 2, 1], 5, None)]
#[case("loadFirstItem=true", &[5, 4, 3, 2, 1], 5, Some(5))]
#[sqlx::test(fixtures("items"))]
async fn test_get_items(
    #[case] filter: &str,
    #[case] expected_item_ids: &[i64],
    #[case] expected_total_items: i32,
    #[case] expected_first_item_id: Option<i64>,
    #[ignore] repository: SqlitePool,
) -> Result<()> {
    let service = new_service(repository)?;
    let uri: Uri = format!("http://localhost?{filter}").parse()?;
    let filter: Query<ItemFilter> = Query::try_from_uri(&uri)?;
    let result = service.get_items(&filter).await?;
    let item_ids: Vec<i64> = result.items.iter().map(|item| item.id).collect();

    assert_eq!(item_ids, expected_item_ids);
    assert_eq!(result.total_items, expected_total_items);
    assert_eq!(result.systems, &["system-1", "system-2"]);

    assert_eq!(
        result.first_item.map(|item| item.summary.id),
        expected_first_item_id
    );

    Ok(())
}

#[sqlx::test]
async fn test_save_item(repository: SqlitePool) -> Result<()> {
    const BODY: &[u8] = br#"{"entityEventId": 567}"#;
    const EVENT_ID: i64 = 123;
    const SYSTEM: &str = "system";
    const USER_AGENT: &str = "user-agent";

    let service = new_service(repository)?;

    let id = service
        .save_item(
            &[
                NewItemHeader {
                    name: "mgs-event-id",
                    value: EVENT_ID.to_string().as_bytes(),
                },
                NewItemHeader {
                    name: "mgs-system-id",
                    value: SYSTEM.as_bytes(),
                },
                NewItemHeader {
                    name: "user-agent",
                    value: USER_AGENT.as_bytes(),
                },
            ],
            BODY,
        )
        .await?;

    let item = service.get_item(id).await?.unwrap();
    let summary = item.summary;

    assert_eq!(id, 1);
    assert_eq!(id, summary.id);
    assert_eq!(Some(SYSTEM), summary.system.as_deref());
    assert_eq!(Some("event_notification"), summary.r#type.as_deref());
    assert_eq!(Some(EVENT_ID), summary.event_id);
    assert_eq!(Some(567), summary.entity_event_id);
    assert_eq!(Some(USER_AGENT), summary.user_agent.as_deref());

    assert_eq!(
        vec!["mgs-event-id", "mgs-system-id", "user-agent"],
        item.headers
            .into_iter()
            .map(|header| header.name)
            .collect::<Vec<String>>()
    );

    assert_eq!(BODY, item.body);

    Ok(())
}
