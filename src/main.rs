use reqwest::{Client, Url};
use serde_json::json;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::env;
use dotenv::dotenv;
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let username = env::var("STRONG_USER").expect("STRONG_USER must be set");
    let password = env::var("STRONG_PASS").expect("STRONG_PASS must be set");
    let strong_backend = env::var("STRONG_BACKEND").expect("STRONG_BACKEND must be set");

    let url = Url::parse(format!("{strong_backend}/auth/login").as_str()).ok().unwrap();

    let mut strong_api = StrongApi::new(url);

    strong_api.login(username.as_str(), password.as_str()).await?;

    dbg!(strong_api.access_token);
    dbg!(strong_api.refresh_token);
    dbg!(strong_api.user_id);

    Ok(())
}

pub struct StrongApi {
    url: Url,
    headers: HeaderMap,
    client: Client,
    refresh_token: Option<String>,
    access_token: Option<String>,
    user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(rename = "userId")]
    user_id: Option<String>,
}

impl StrongApi {
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

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let body = json!({
            "usernameOrEmail": username,
            "password": password
        });

        let response = self.client
            .post(self.url.clone())
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
}