use std::fmt;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}, Url};
use serde::Deserialize;
use serde_json::json;

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

pub enum Includes {
    Log,
    Measurement,
    Tag,
    Widget,
    Template,
    Folder,
    MeasuredValue
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
        headers.insert(HeaderName::from_static("user-agent"), HeaderValue::from_static("Strong Android"));
        headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"));
        headers.insert(HeaderName::from_static("accept"), HeaderValue::from_static("application/json"));
        headers.insert(HeaderName::from_static("x-client-build"), HeaderValue::from_static("600013"));
        headers.insert(HeaderName::from_static("x-client-platform"), HeaderValue::from_static("android"));

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
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.url.join("auth/login").unwrap();

        let body = json!({
            "usernameOrEmail": username,
            "password": password
        });

        let response = self.client
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

    /// Refreshes the access token using the access and refresh token which were obtained during login.
    /// Should be called when you receive a 401 Unauthorized response from the Strong backend.
    pub async fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.url.join("auth/login/refresh").unwrap();

        let body = json!({
            "accessToken": self.access_token,
            "refreshToken": self.refresh_token
        });

        let response = self.client
            .post(url)
            .bearer_auth(self.access_token.clone().unwrap())
            .headers(self.headers.clone())
            .json(&body)
            .send()
            .await?;

        dbg!(response.status());

        let response_text = response.text().await?;

        let parsed: LoginResponse = serde_json::from_str(&response_text)?;

        self.access_token = parsed.access_token;
        self.refresh_token = parsed.refresh_token;

        Ok(())
    }

    #[cfg(feature = "full")]
    pub async fn refresh_by_tokens(
        &mut self, access_token: String,
        refresh_token: String
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.url.join("auth/login/refresh").unwrap();

        let body = json!({
            "accessToken": access_token.clone(),
            "refreshToken": refresh_token,
        });

        let response = self.client
            .post(url)
            .bearer_auth(access_token)
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

    pub async fn get_user(&self, continuation: &str, limit: i16, includes: Vec<Includes>) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = &*self.user_id.clone().unwrap();
        let mut url = self.url.join(format!("api/users/{user_id}").as_str()).unwrap();

        url.set_query(Some(&format!("limit={}&continuation={}", limit, continuation)));

        for include in includes {
            url.set_query(Some(&format!("{}&include={}", url.query().unwrap(), include)));
        }

        dbg!(&url.to_string());

        let response = self.client
            .get(url)
            .bearer_auth(self.access_token.clone().unwrap())
            .headers(self.headers.clone())
            .send()
            .await?;

        let response_text = response.text().await?;

        dbg!(response_text);

        Ok(())
    }

    pub async fn get_measurements(&self) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = &*self.user_id.clone().unwrap();
        let url = self.url.join(format!("api/measurements/{user_id}").as_str()).unwrap();

        let response = self.client
            .get(url)
            .bearer_auth(self.access_token.clone().unwrap())
            .headers(self.headers.clone())
            .send()
            .await?;

        let response_text = response.text().await?;

        dbg!(response_text);

        Ok(())
    }
}
