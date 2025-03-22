use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub description: String,
}

impl fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.description)
    }
}

impl std::error::Error for ApiErrorResponse {}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserResponse {
    #[serde(rename = "_links")]
    pub links: Value,
    #[serde(rename = "_embedded")]
    pub embedded: Embedded,
    pub id: String,
    pub created: String,
    #[serde(rename = "lastChanged")]
    pub last_changed: String,
    pub username: String,
    pub email: String,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    pub name: Option<String>,
    pub avatar: Value,
    pub preferences: Value,
    #[serde(rename = "legacyPurchase")]
    pub legacy_purchase: Value,
    #[serde(rename = "legacyGoals")]
    pub legacy_goals: Value,
    #[serde(rename = "startHistoryFromDate")]
    pub start_history_from_date: String,
    #[serde(rename = "firstWeekDay")]
    pub first_week_day: String,
    #[serde(rename = "availableLogins")]
    pub available_logins: Vec<Value>,
    pub migrated: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embedded {
    pub measurement: Option<Vec<Value>>,
    #[serde(rename = "measuredValue")]
    pub measured_value: Option<Vec<Value>>,
    pub template: Option<Vec<Value>>,
    pub log: Option<Vec<Log>>,
    pub tag: Option<Vec<Value>>,
    pub folder: Option<Vec<Value>>,
    pub widget: Option<Vec<Value>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Log {
    #[serde(rename = "_links")]
    pub links: Value,
    #[serde(rename = "_embedded")]
    pub embedded: LogEmbedded,
    #[serde(rename = "timezoneId")]
    pub timezone_id: Option<String>,
    pub id: String,
    pub created: String,
    #[serde(rename = "lastChanged")]
    pub last_changed: String,
    pub name: Option<Name>,
    pub access: String,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    #[serde(rename = "logType")]
    pub log_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub en: Option<String>,
    pub custom: Option<String>,
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.en {
            Some(en) => write!(f, "{}", en),
            None => match &self.custom {
                Some(custom) => write!(f, "{}", custom),
                None => write!(f, "Unknown"),
            },
        }
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        match name.en {
            Some(en) => en,
            None => match name.custom {
                Some(custom) => custom,
                None => "Unknown".to_string(),
            },
        }
    }
}

impl From<String> for Name {
    fn from(name: String) -> Self {
        Name {
            en: Some(name),
            custom: None,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEmbedded {
    #[serde(rename = "cellSetGroup")]
    pub cell_set_group: Vec<CellSetGroup>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellSetGroup {
    #[serde(rename = "_links")]
    pub links: CellSetGroupLinks,
    #[serde(rename = "_embedded")]
    pub embedded: CellSetGroupEmbedded,
    pub id: String,
    #[serde(rename = "cellSets")]
    pub cell_sets: Vec<CellSet>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellSetGroupLinks {
    pub measurement: Option<Link>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellSetGroupEmbedded {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellSet {
    pub id: String,
    pub cells: Vec<Cell>,
    #[serde(rename = "isCompleted")]
    pub is_completed: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    #[serde(rename = "cellType")]
    pub cell_type: String,
    pub value: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instructions {
    pub en: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Media {
    pub url: String,
    #[serde(rename = "type")]
    pub media_type: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellTypeConfig {
    #[serde(rename = "cellType")]
    pub cell_type: String,
    pub mandatory: Option<bool>,
    #[serde(rename = "isExponent")]
    pub is_exponent: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementLinks {
    #[serde(rename = "self")]
    pub self_link: Link,
    pub tag: Option<Vec<Link>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    #[serde(rename = "_links")]
    pub links: MeasurementLinks,
    pub id: String,
    pub created: String,
    #[serde(rename = "lastChanged")]
    pub last_changed: String,
    pub name: Name,
    pub instructions: Option<Instructions>,
    pub media: Vec<Media>,
    #[serde(rename = "cellTypeConfigs")]
    pub cell_type_configs: Vec<CellTypeConfig>,
    #[serde(rename = "isGlobal")]
    pub is_global: bool,
    #[serde(rename = "measurementType")]
    pub measurement_type: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementsResponse {
    #[serde(rename = "_links")]
    pub links: Links,
    pub total: u32,
    #[serde(rename = "_embedded")]
    pub embedded: EmbeddedMeasurements,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_link: Link,
    pub next: Option<Link>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedMeasurements {
    #[serde(rename = "measurement")]
    pub measurements: Vec<Measurement>,
}

impl MeasurementsResponse {
    pub fn merge(self, other: Self) -> Self {
        MeasurementsResponse {
            links: self.links,
            total: self.total,
            // Concatenate the measurement vectors.
            embedded: EmbeddedMeasurements {
                measurements: {
                    let mut merged = self.embedded.measurements;
                    merged.extend(other.embedded.measurements);
                    merged
                },
            },
        }
    }
}
