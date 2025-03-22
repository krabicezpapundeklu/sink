use std::{borrow::Cow, future::Future};

use crate::{
    model::{Item, ItemFilter, ItemSearchResult, NewItem, NewItemHeader},
    repository::Repository,
};

use anyhow::Result;
use regex::bytes::{Regex, RegexSet};
use serde::Deserialize;

pub trait Service: Clone + Send + Sync {
    fn get_item(&self, id: i64) -> impl Future<Output = Result<Option<Item>>> + Send;

    fn get_items(
        &self,
        filter: &ItemFilter,
    ) -> impl Future<Output = Result<ItemSearchResult>> + Send;

    fn save_item(
        &self,
        headers: &[NewItemHeader<'_>],
        body: &[u8],
    ) -> impl Future<Output = Result<i64>> + Send;
}

#[derive(Clone)]
struct ServiceImpl<R>
where
    R: Repository,
{
    repository: R,
    item_types: Vec<(String, usize)>,
    item_type_regexes: RegexSet,
    system_regex: Regex,
    entity_event_id_regex: Regex,
}

impl<R> Service for ServiceImpl<R>
where
    R: Repository,
{
    async fn get_item(&self, id: i64) -> Result<Option<Item>> {
        self.repository.get_item(id).await
    }

    async fn get_items(&self, filter: &ItemFilter) -> Result<ItemSearchResult> {
        let (items, total_items) = self.repository.get_items(filter).await?;

        let first_item = if filter.load_first_item.unwrap_or_default() && !items.is_empty() {
            self.repository.get_item(items[0].id).await?
        } else {
            None
        };

        let systems = self.repository.get_systems().await?;

        Ok(ItemSearchResult {
            items,
            total_items,
            systems,
            first_item,
        })
    }

    async fn save_item(&self, headers: &[NewItemHeader<'_>], body: &[u8]) -> Result<i64> {
        let system = self.get_system(headers, body);
        let r#type = self.get_item_type(body);
        let event_id = get_event_id(headers);
        let entity_event_id = self.get_entity_event_id(body);
        let user_agent = get_user_agent(headers);

        self.repository
            .insert_item(&NewItem {
                system: system.as_deref(),
                r#type,
                event_id,
                entity_event_id,
                user_agent: user_agent.as_deref(),
                headers,
                body,
            })
            .await
    }
}

impl<R> ServiceImpl<R>
where
    R: Repository,
{
    fn get_entity_event_id(&self, body: &[u8]) -> Option<i64> {
        self.entity_event_id_regex
            .captures(body)
            .and_then(|captures| captures.get(1))
            .and_then(|group| {
                String::from_utf8_lossy(group.as_bytes())
                    .parse::<i64>()
                    .ok()
            })
    }

    fn get_item_type(&self, body: &[u8]) -> Option<&str> {
        let matches = self.item_type_regexes.matches(body);

        if matches.matched_any() {
            let mut i = 0;

            'next_item_type: for (key, patterns) in &self.item_types {
                for j in 0..*patterns {
                    if !matches.matched(i + j) {
                        i += patterns;
                        continue 'next_item_type;
                    }
                }

                return Some(key);
            }
        }

        None
    }

    fn get_system<'a>(
        &self,
        headers: &[NewItemHeader<'a>],
        body: &'a [u8],
    ) -> Option<Cow<'a, str>> {
        headers
            .iter()
            .find(|header| header.name == "mgs-system-id" || header.name == "mgssystem")
            .map(|header| header.value)
            .or_else(|| {
                self.system_regex
                    .captures(body)
                    .and_then(|captures| captures.get(1))
                    .map(|group| group.as_bytes())
            })
            .map(|system| String::from_utf8_lossy(system))
    }

    fn new(repository: R) -> Result<Self> {
        #[derive(Deserialize)]
        struct ItemType {
            key: String,
            matches: Vec<String>,
        }

        let item_types: Vec<ItemType> = serde_json::from_str(include_str!("../item.types.json"))?;

        let item_type_regexes = RegexSet::new(
            item_types
                .iter()
                .flat_map(|item_type| item_type.matches.iter()),
        )?;

        Ok(Self {
            repository,
            item_types: item_types
                .into_iter()
                .map(|item_type| (item_type.key, item_type.matches.len()))
                .collect(),
            item_type_regexes,
            system_regex: Regex::new("<mgsSystem>([^<]+)")?,
            entity_event_id_regex: Regex::new(r#""entityEventId"\s*:\s*(\d+)"#)?,
        })
    }
}

fn get_event_id(headers: &[NewItemHeader<'_>]) -> Option<i64> {
    headers
        .iter()
        .find(|header| header.name == "mgs-event-id")
        .and_then(|header| String::from_utf8_lossy(header.value).parse::<i64>().ok())
}

fn get_user_agent<'a>(headers: &[NewItemHeader<'a>]) -> Option<Cow<'a, str>> {
    headers
        .iter()
        .find(|header| header.name == "user-agent")
        .map(|header| String::from_utf8_lossy(header.value))
}

pub fn new_service(repository: impl Repository) -> Result<impl Service> {
    ServiceImpl::new(repository)
}

#[cfg(test)]
mod tests {
    use crate::repository::tests::MockRepository;

    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn repository() -> impl Repository {
        MockRepository::new()
    }

    #[fixture]
    fn service(repository: impl Repository) -> ServiceImpl<impl Repository> {
        ServiceImpl::new(repository).unwrap()
    }

    #[rstest]
    #[case(b"", None)]
    #[case(br#"{"entityEventId": 123}"#, Some(123))]
    #[case(br#"{"entityEventId": abc}"#, None)]
    fn test_get_entity_event_id(
        #[case] body: &[u8],
        #[case] expected_entity_event_id: Option<i64>,
        service: ServiceImpl<impl Repository>,
    ) -> Result<()> {
        let entity_event_id = service.get_entity_event_id(body);
        assert_eq!(entity_event_id, expected_entity_event_id);
        Ok(())
    }

    #[rstest]
    #[case(b"123", Some(123))]
    #[case(b"xxx", None)]
    fn test_get_event_id(
        #[case] header_value: &[u8],
        #[case] expected_event_id: Option<i64>,
    ) -> Result<()> {
        let event_id = get_event_id(&[NewItemHeader {
            name: "mgs-event-id",
            value: header_value,
        }]);

        assert_eq!(event_id, expected_event_id);

        Ok(())
    }

    #[test]
    fn test_get_event_id_no_header() -> Result<()> {
        let event_id = get_event_id(&[]);
        assert!(event_id.is_none());
        Ok(())
    }

    #[rstest]
    #[case(b"", None)]
    #[case(br#"{"entityEventId": 123}"#, Some("event_notification"))]
    #[case(
        br#"{"entityEventId": 123, "eventDesc": "abc"}"#,
        Some("event_payload")
    )]
    #[case(br#"<Folder type="CL">"#, Some("folder_cl"))]
    #[case(br#"<createdApplication>"#, Some("application_created"))]
    #[case(br#"<Create__CompIntfc__Z_USAS_UPDT_STATUS>"#, Some("status_updated"))]
    #[case(br#"<updateStatusRequest>"#, Some("status_updated"))]
    #[case(br#"<tns:updatedVacancy>"#, Some("vacancy_updated"))]
    fn test_get_item_type(
        #[case] body: &[u8],
        #[case] expected_item_type: Option<&str>,
        service: ServiceImpl<impl Repository>,
    ) -> Result<()> {
        let item_type = service.get_item_type(body);
        assert_eq!(item_type, expected_item_type);
        Ok(())
    }

    #[rstest]
    #[case("", b"", b"", None)]
    #[case("mgs-system-id", b"system", b"", Some("system"))]
    #[case("mgssystem", b"system", b"", Some("system"))]
    #[case("", b"", b"<mgsSystem>system</mgsSystem>", Some("system"))]
    fn test_get_system(
        #[case] header_name: &str,
        #[case] header_value: &[u8],
        #[case] body: &[u8],
        #[case] expected_system: Option<&str>,
        service: ServiceImpl<impl Repository>,
    ) -> Result<()> {
        let system = service.get_system(
            &[NewItemHeader {
                name: header_name,
                value: header_value,
            }],
            body,
        );

        assert_eq!(system.as_deref(), expected_system);

        Ok(())
    }
}
