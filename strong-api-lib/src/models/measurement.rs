use serde::{Deserialize, Serialize};

use super::common::{Link, Links, Name};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementsResponse {
    #[serde(rename = "_links")]
    pub links: Links,
    pub total: u32,
    #[serde(rename = "_embedded")]
    pub embedded: EmbeddedMeasurements,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedMeasurements {
    #[serde(rename = "measurement")]
    pub measurements: Vec<Measurement>,
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

impl MeasurementsResponse {
    pub fn merge(self, other: Self) -> Self {
        MeasurementsResponse {
            links: self.links,
            total: self.total,
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
