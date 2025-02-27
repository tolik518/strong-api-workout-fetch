use reqwest::Url;
use std::env;
use dotenv::dotenv;
use serde::Deserialize;
use crate::strong_api::StrongApi;

mod strong_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let username = env::var("STRONG_USER").expect("STRONG_USER must be set");
    let password = env::var("STRONG_PASS").expect("STRONG_PASS must be set");
    let strong_backend = env::var("STRONG_BACKEND").expect("STRONG_BACKEND must be set");

    let url = Url::parse(&*strong_backend).ok().unwrap();

    let mut strong_api = StrongApi::new(url);

    strong_api.login(username.as_str(), password.as_str()).await?;
    strong_api.get_user().await?;

    dbg!(strong_api.access_token);
    dbg!(strong_api.refresh_token);
    dbg!(strong_api.user_id);

    Ok(())
}