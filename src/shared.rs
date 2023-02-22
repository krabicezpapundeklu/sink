use std::{
    fmt::{self, Display, Formatter},
    result,
    str::FromStr,
};

use anyhow::{bail, Error};
use chrono::{NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_slice, Value};

use xml::{
    attribute::OwnedAttribute,
    reader::XmlEvent::{Characters, EndElement, StartElement},
    EventReader,
};

macro_rules! item_types {
    ($($id:ident = $name:literal)+) => {
        #[derive(Debug)]
        pub enum ItemType {
            $($id),+
        }

        impl Display for ItemType {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                let name = match &self {
                    $(Self::$id => $name),+
                };

                name.fmt(f)
            }
        }

        impl FromStr for ItemType {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let r#type = match s {
                    $($name => Self::$id),+,
                    _ => bail!("wrong item type {s}"),
                };

                Ok(r#type)
            }
        }
    };
}

macro_rules! serializable_as_string {
    ($($type:ty)+) => {
        $(
            impl<'de> Deserialize<'de> for $type {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de> {
                    let s = String::deserialize(deserializer)?;
                    FromStr::from_str(&s).map_err(de::Error::custom)
                }
            }

            impl Serialize for $type {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer {
                    serializer.serialize_str(&self.to_string())
                }
            }
        )+
    }
}

serializable_as_string! {
    DateTime
    ItemType
}

item_types! {
    ApplicationCreated = "application_created"
    ApplicationUpdated = "application_updated"
    CertificateCreated = "certificate_created"
    CertificateUpdated = "certificate_updated"
    EventNotification = "event_notification"
    EventPayload = "event_payload"
    FolderCL = "folder_cl"
    FolderFS = "folder_fs"
    FolderIDPD = "folder_idpd"
    FolderPD = "folder_pd"
    SelecteeCreated = "selectee_created"
    SelecteeUpdated = "selectee_updated"
    StatusUpdated = "status_updated"
    VacancyCreated = "vacancy_created"
    VacancyUpdated = "vacancy_updated"
}

#[derive(Debug)]
pub struct DateTime(NaiveDateTime);

impl DateTime {
    pub fn now() -> Self {
        DateTime(Utc::now().naive_utc())
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.format("%Y-%m-%d %H:%M:%S").fmt(f)
    }
}

impl FromStr for DateTime {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let date_time = NaiveDateTime::parse_from_str(
            s,
            if s.len() == 16 {
                "%Y-%m-%d %H:%M"
            } else {
                "%Y-%m-%d %H:%M:%S"
            },
        )?;

        Ok(Self(date_time))
    }
}

#[derive(Serialize)]
pub struct ItemHeader {
    pub name: String,

    #[serde(serialize_with = "bytes_as_string")]
    pub value: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Option<i64>,
    pub submit_date: DateTime,
    pub system: Option<String>,
    pub r#type: Option<ItemType>,
    pub headers: Vec<ItemHeader>,

    #[serde(serialize_with = "bytes_as_string")]
    pub body: Vec<u8>,
}

impl Item {
    fn map_xml_path_to_item_type(path: &str, attributes: &[OwnedAttribute]) -> Option<ItemType> {
        let r#type = match path {
            "/Envelope/Body/Create__CompIntfc__Z_USAS_UPDT_STATUS"
            | "/Envelope/Body/updateStatusRequest" => ItemType::StatusUpdated,
            "/Envelope/Body/createdApplication" => ItemType::ApplicationCreated,
            "/Envelope/Body/createdCertificate" => ItemType::CertificateCreated,
            "/Envelope/Body/createdVacancy" => ItemType::VacancyCreated,
            "/Envelope/Body/createSelecteeRequest" => ItemType::SelecteeCreated,
            "/Envelope/Body/Update__CompIntfc__Z_USAS_LD_SELECTEE" => ItemType::SelecteeUpdated,
            "/Envelope/Body/updatedApplication" => ItemType::ApplicationUpdated,
            "/Envelope/Body/updatedCertificate" => ItemType::CertificateUpdated,
            "/Envelope/Body/updatedVacancy" => ItemType::VacancyUpdated,
            "/Folder" => {
                if let Some(type_attr) = attributes
                    .iter()
                    .find(|attr| attr.name.local_name == "type")
                {
                    match type_attr.value.as_str() {
                        "CL" => ItemType::FolderCL,
                        "FS" => ItemType::FolderFS,
                        "IDPD" => ItemType::FolderIDPD,
                        "SINGLEPD" => ItemType::FolderPD,
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        Some(r#type)
    }

    pub fn update_metadata(&mut self) {
        self.system = self
            .headers
            .iter()
            .filter(|header| header.name == "mgs-system-id" || header.name == "mgssystem")
            .map(|header| String::from_utf8_lossy(&header.value).to_string())
            .next();

        self.r#type = None;

        if let Ok(json) = from_slice::<Value>(&self.body) {
            if json.get("entityEventId").is_some() {
                if json.get("eventDesc").is_some() {
                    self.r#type = Some(ItemType::EventPayload);
                } else {
                    self.r#type = Some(ItemType::EventNotification);
                }
            }
        } else {
            let xml = EventReader::new(self.body.as_slice());
            let mut path = String::new();

            for e in xml {
                if let Ok(e) = e {
                    match e {
                        Characters(data) => {
                            if self.system.is_none() && path.ends_with("/mgsSystem") {
                                self.system = Some(data);
                            }
                        }
                        EndElement { .. } => {
                            if let Some(idx) = path.rfind('/') {
                                path.truncate(idx);
                            }
                        }
                        StartElement {
                            name, attributes, ..
                        } => {
                            path.push('/');
                            path.push_str(&name.local_name);

                            if self.r#type.is_none() {
                                self.r#type = Self::map_xml_path_to_item_type(&path, &attributes);
                            }
                        }
                        _ => (),
                    };
                } else {
                    break;
                }
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFilter {
    pub batch_size: u32,
    pub next_item_id: i64,
    pub query: Option<String>,
    pub system: Option<String>,
    pub r#type: Option<ItemType>,
    pub from: Option<DateTime>,
    pub to: Option<DateTime>,
    pub asc: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSearchResult {
    pub items: Vec<ItemSummary>,
    pub systems: Vec<String>,
    pub total_items: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: i64,
    pub submit_date: DateTime,
    pub system: Option<String>,
    pub r#type: Option<ItemType>,
}

fn bytes_as_string<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&String::from_utf8_lossy(bytes))
}
