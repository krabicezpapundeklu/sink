use serde::{Deserialize, Serialize, Serializer};
use sqlx::FromRow;

pub const CONTENT_TYPE: &str = "content-type";
pub const X_RESPONSE_HEADER_PREFIX: &str = "x-response-header-";

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(flatten)]
    pub summary: ItemSummary,
    pub headers: Vec<ItemHeader>,
    #[serde(serialize_with = "bytes_as_string")]
    pub body: Vec<u8>,
}

impl Item {
    pub fn content_type(&self) -> Option<&[u8]> {
        self.headers.iter().find_map(|header| {
            if header.name == CONTENT_TYPE {
                Some(header.value.as_slice())
            } else {
                None
            }
        })
    }

    pub fn x_response_headers(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.headers.iter().filter_map(|header| {
            header
                .name
                .strip_prefix(X_RESPONSE_HEADER_PREFIX)
                .map(|name| (name, header.value.as_slice()))
        })
    }
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
    pub fn new(name: &str, value: &[u8]) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
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

#[derive(Default, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: i64,
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
    pub submit_date: String,
}

pub struct NewItem<'a> {
    pub system: Option<&'a str>,
    pub r#type: Option<&'a str>,
    pub event_id: Option<i64>,
    pub entity_event_id: Option<i64>,
    pub user_agent: Option<&'a str>,
    pub headers: &'a [NewItemHeader<'a>],
    pub body: &'a [u8],
}

pub struct NewItemHeader<'a> {
    pub name: &'a str,
    pub value: &'a [u8],
}

fn bytes_as_string<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&String::from_utf8_lossy(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_content_type() {
        const CONTENT_TYPE_VALUE: &[u8] = b"text";

        let headers = vec![
            ItemHeader::new("header", b"value"),
            ItemHeader::new(CONTENT_TYPE, CONTENT_TYPE_VALUE),
        ];

        let item = Item {
            headers,
            ..Default::default()
        };

        assert_eq!(item.content_type(), Some(CONTENT_TYPE_VALUE));
    }

    #[test]
    fn test_item_x_response_headers() {
        const X_HEADER_NAME_1: &str = "header-1";
        const X_HEADER_VALUE_1: &[u8] = b"value-1";
        const X_HEADER_NAME_2: &str = "header-2";
        const X_HEADER_VALUE_2: &[u8] = b"value-2";

        let headers = vec![
            ItemHeader::new("header", b"value"),
            ItemHeader::new(
                &format!("{X_RESPONSE_HEADER_PREFIX}{X_HEADER_NAME_1}"),
                X_HEADER_VALUE_1,
            ),
            ItemHeader::new(
                &format!("{X_RESPONSE_HEADER_PREFIX}{X_HEADER_NAME_2}"),
                X_HEADER_VALUE_2,
            ),
        ];

        let item = Item {
            headers,
            ..Default::default()
        };

        let mut x_headers = item.x_response_headers();

        assert_eq!(x_headers.next(), Some((X_HEADER_NAME_1, X_HEADER_VALUE_1)));
        assert_eq!(x_headers.next(), Some((X_HEADER_NAME_2, X_HEADER_VALUE_2)));
        assert_eq!(x_headers.next(), None);
    }
}
