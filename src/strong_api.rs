use crate::user_response::UserResponse;
use reqwest::{
    Client, Url,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde::Deserialize;
use serde_json::json;
use std::fmt;

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    code: String,
    description: String,
}

impl fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.description)
    }
}

impl std::error::Error for ApiErrorResponse {}

#[derive(Debug)]
pub struct StrongApi {
    url: Url,
    headers: HeaderMap,
    client: Client,
    pub(crate) refresh_token: Option<String>,
    pub(crate) access_token: Option<String>,
    pub(crate) user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(rename = "userId")]
    user_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Includes {
    Log,
    Measurement,
    Tag,
    Widget,
    Template,
    Folder,
    MeasuredValue,
}

impl fmt::Display for Includes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Includes::Log => "log",
            Includes::Measurement => "measurement",
            Includes::Tag => "tag",
            Includes::Widget => "widget",
            Includes::Template => "template",
            Includes::Folder => "folder",
            Includes::MeasuredValue => "measuredValue",
        };
        write!(f, "{}", value)
    }
}

impl StrongApi {
    /// Creates a new StrongApi instance with the provided backend URL.
    pub fn new(url: Url) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("user-agent"),
            HeaderValue::from_static("Strong Android"),
        );
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            HeaderName::from_static("x-client-build"),
            HeaderValue::from_static("600013"),
        );
        headers.insert(
            HeaderName::from_static("x-client-platform"),
            HeaderValue::from_static("android"),
        );

        Self {
            url,
            headers,
            client: Client::new(),
            refresh_token: None,
            access_token: None,
            user_id: None,
        }
    }

    /// Logs in to the Strong backend using the provided username and password.
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.url.join("auth/login")?;
        let body = json!({
            "usernameOrEmail": username,
            "password": password
        });

        let response = self
            .client
            .post(url)
            .headers(self.headers.clone())
            .json(&body)
            .send()
            .await?;
        let response_text = response.text().await?;

        let parsed: LoginResponse = serde_json::from_str(&response_text)?;

        self.access_token = parsed.access_token;
        self.refresh_token = parsed.refresh_token;
        self.user_id = parsed.user_id;

        Ok(())
    }

    /// Refreshes the access token using tokens obtained during login.
    pub async fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.url.join("auth/login/refresh")?;
        let body = json!({
            "accessToken": self.access_token,
            "refreshToken": self.refresh_token
        });

        // Ensure the access token exists
        let access_token = self.access_token.clone().ok_or("Missing access token")?;
        let response = self
            .client
            .post(url)
            .bearer_auth(&access_token)
            .headers(self.headers.clone())
            .json(&body)
            .send()
            .await?;

        // Log the status (consider replacing with proper logging)
        eprintln!("Refresh status: {}", response.status());
        let response_text = response.text().await?;
        let parsed: LoginResponse = serde_json::from_str(&response_text)?;

        self.access_token = parsed.access_token;
        self.refresh_token = parsed.refresh_token;

        Ok(())
    }

    #[cfg(feature = "full")]
    pub async fn refresh_by_tokens(
        &mut self,
        access_token: String,
        refresh_token: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.url.join("auth/login/refresh")?;
        let body = json!({
            "accessToken": access_token.clone(),
            "refreshToken": refresh_token,
        });

        let response = self
            .client
            .post(url)
            .bearer_auth(&access_token)
            .headers(self.headers.clone())
            .json(&body)
            .send()
            .await?;
        let response_text = response.text().await?;
        let parsed: LoginResponse = serde_json::from_str(&response_text)?;

        self.access_token = parsed.access_token;
        self.refresh_token = parsed.refresh_token;

        Ok(())
    }

    pub async fn get_user(
        &self,
        continuation: &str,
        limit: i16,
        includes: Vec<Includes>,
    ) -> Result<UserResponse, Box<dyn std::error::Error>> {
        let user_id = self.user_id.as_ref().ok_or("Missing user id")?;
        let mut url = self.url.join(&format!("api/users/{user_id}"))?;

        {
            // Use query_pairs_mut to build the query string.
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("limit", &limit.to_string());
            query_pairs.append_pair("continuation", continuation);
            for include in includes {
                query_pairs.append_pair("include", &include.to_string());
            }
        }
        // Drop the mutable borrow here.
        eprintln!("Request URL: {}", url);

        let response = self
            .client
            .get(url)
            .bearer_auth(self.access_token.as_ref().ok_or("Missing access token")?)
            .headers(self.headers.clone())
            .send()
            .await?;

        // Capture the status before consuming the response.
        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            let api_error: ApiErrorResponse = serde_json::from_str(&response_text)?;
            return Err(Box::new(api_error));
        }

        let parsed: UserResponse = serde_json::from_str(&response_text)?;
        Ok(parsed)
    }

    pub async fn get_measurements(&self) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = self.user_id.as_ref().ok_or("Missing user id")?;
        let url = self.url.join(&format!("api/measurements/{user_id}"))?;
        let response = self
            .client
            .get(url)
            .bearer_auth(self.access_token.as_ref().ok_or("Missing access token")?)
            .headers(self.headers.clone())
            .send()
            .await?;
        let response_text = response.text().await?;
        eprintln!("Measurements response: {}", response_text);
        Ok(())
    }

    pub async fn get_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = self.user_id.as_ref().ok_or("Missing user id")?;
        let url = self.url.join(&format!("api/logs/{user_id}"))?;
        let response = self
            .client
            .get(url)
            .bearer_auth(self.access_token.as_ref().ok_or("Missing access token")?)
            .headers(self.headers.clone())
            .send()
            .await?;
        let response_text = response.text().await?;
        eprintln!("Logs response: {}", response_text);
        Ok(())
    }
}
