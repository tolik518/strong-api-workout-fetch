use serde::Deserialize;
use std::fmt;

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
