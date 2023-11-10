use axum::http::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Option<i64>,
    pub submit_date: String,
    pub system: Option<String>,
    pub r#type: Option<String>,
    pub headers: Vec<ItemHeader>,

    #[serde(serialize_with = "bytes_as_string")]
    pub body: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFilter {
    pub query: Option<String>,
    pub system: Option<String>,
    pub r#type: Option<String>,
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

impl From<(&HeaderName, &HeaderValue)> for ItemHeader {
    fn from((name, value): (&HeaderName, &HeaderValue)) -> Self {
        Self {
            name: name.to_string(),
            value: value.as_bytes().into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: i64,
    pub submit_date: String,
    pub system: Option<String>,
    pub r#type: Option<String>,
}

pub struct NewItem {
    pub system: Option<String>,
    pub r#type: Option<String>,
    pub headers: Vec<ItemHeader>,
    pub body: Vec<u8>,
}

fn bytes_as_string<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&String::from_utf8_lossy(bytes))
}
