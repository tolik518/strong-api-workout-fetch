use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_link: Link,
    pub next: Option<Link>,
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
