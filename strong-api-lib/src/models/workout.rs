use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::common::{Link, Name};

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
