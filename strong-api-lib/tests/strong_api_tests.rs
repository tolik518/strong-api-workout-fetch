use reqwest::Url;
use strong_api_lib::strong_api::{Includes, StrongApi};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
async fn start_server() -> MockServer {
    MockServer::start().await
}
fn api(server: &MockServer) -> StrongApi {
    StrongApi::new(Url::parse(&server.uri()).unwrap())
}
fn refused_api() -> StrongApi {
    StrongApi::new(Url::parse("http://127.0.0.1:1/").unwrap())
}
fn login_body() -> serde_json::Value {
    serde_json::json!({
        "accessToken": "test-access-token",
        "refreshToken": "test-refresh-token",
        "userId": "00000000-0000-0000-0000-000000000001"
    })
}
fn user_response_body() -> serde_json::Value {
    serde_json::json!({
        "_links": {},
        "_embedded": { "log": [] },
        "id": "00000000-0000-0000-0000-000000000001",
        "created": "2020-01-01T00:00:00Z",
        "lastChanged": "2020-01-01T00:00:00Z",
        "username": "testuser",
        "email": "test@example.com",
        "emailVerified": true,
        "name": null,
        "avatar": null,
        "preferences": {},
        "legacyPurchase": null,
        "legacyGoals": null,
        "startHistoryFromDate": "2020-01-01",
        "firstWeekDay": "MONDAY",
        "availableLogins": [],
        "migrated": "false"
    })
}
fn measurements_body() -> serde_json::Value {
    serde_json::json!({
        "_links": { "self": { "href": "/api/measurements?page=1" } },
        "total": 1,
        "_embedded": {
            "measurement": [{
                "_links": { "self": { "href": "/api/measurements/aaaaaaaa-0000-0000-0000-000000000001" } },
                "id": "aaaaaaaa-0000-0000-0000-000000000001",
                "created": "2021-01-01T00:00:00Z",
                "lastChanged": "2021-01-01T00:00:00Z",
                "name": { "en": "Squat", "custom": null },
                "instructions": null,
                "media": [],
                "cellTypeConfigs": [],
                "isGlobal": true,
                "measurementType": "WEIGHT_AND_REPS"
            }]
        }
    })
}
// ---------------------------------------------------------------------------
// Includes Display
// ---------------------------------------------------------------------------
#[test]
fn test_includes_display() {
    assert_eq!(Includes::Log.to_string(), "log");
    assert_eq!(Includes::Measurement.to_string(), "measurement");
    assert_eq!(Includes::Tag.to_string(), "tag");
    assert_eq!(Includes::Widget.to_string(), "widget");
    assert_eq!(Includes::Template.to_string(), "template");
    assert_eq!(Includes::Folder.to_string(), "folder");
    assert_eq!(Includes::MeasuredValue.to_string(), "measuredValue");
}
// ---------------------------------------------------------------------------
// StrongApi::new
// ---------------------------------------------------------------------------
#[test]
fn test_strong_api_new_has_no_tokens() {
    let api = StrongApi::new(Url::parse("https://example.com").unwrap());
    assert!(api.access_token.is_none());
    assert!(api.refresh_token.is_none());
    assert!(api.user_id.is_none());
}
// ---------------------------------------------------------------------------
// login — happy path + error paths
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_login_sets_tokens() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(login_body()))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.login("user", "pass").await.unwrap();
    assert_eq!(api.access_token.as_deref(), Some("test-access-token"));
    assert_eq!(api.refresh_token.as_deref(), Some("test-refresh-token"));
    assert_eq!(api.user_id.as_deref(), Some("00000000-0000-0000-0000-000000000001"));
}
#[tokio::test]
async fn test_login_invalid_json_returns_error() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    let mut api = api(&server);
    assert!(api.login("user", "pass").await.is_err());
}
#[tokio::test]
async fn test_login_send_failure_returns_error() {
    let mut api = refused_api();
    assert!(api.login("user", "pass").await.is_err());
}
// ---------------------------------------------------------------------------
// refresh — happy path + error paths
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_refresh_updates_tokens() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/auth/login/refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "accessToken": "new-access-token",
            "refreshToken": "new-refresh-token",
            "userId": "00000000-0000-0000-0000-000000000001"
        })))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.access_token = Some("old-token".to_string());
    api.refresh_token = Some("old-refresh".to_string());
    api.refresh().await.unwrap();
    assert_eq!(api.access_token.as_deref(), Some("new-access-token"));
    assert_eq!(api.refresh_token.as_deref(), Some("new-refresh-token"));
}
#[tokio::test]
async fn test_refresh_without_access_token_returns_error() {
    let mut api = refused_api();
    let result = api.refresh().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing access token"));
}
#[tokio::test]
async fn test_refresh_invalid_json_returns_error() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/auth/login/refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.access_token = Some("token".to_string());
    api.refresh_token = Some("refresh".to_string());
    assert!(api.refresh().await.is_err());
}
#[tokio::test]
async fn test_refresh_send_failure_returns_error() {
    let mut api = refused_api();
    api.access_token = Some("token".to_string());
    api.refresh_token = Some("refresh".to_string());
    assert!(api.refresh().await.is_err());
}
// ---------------------------------------------------------------------------
// get_user — happy path + all error paths
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_get_user_success() {
    let server = start_server().await;
    Mock::given(method("GET"))
        .and(path("/api/users/00000000-0000-0000-0000-000000000001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_response_body()))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.access_token = Some("token".to_string());
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    let user = api.get_user("", 10, vec![Includes::Log]).await.unwrap();
    assert_eq!(user.id, "00000000-0000-0000-0000-000000000001");
}
#[tokio::test]
async fn test_get_user_without_user_id_returns_error() {
    let mut api = refused_api();
    api.access_token = Some("token".to_string());
    let result = api.get_user("", 10, vec![]).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing user id"));
}
#[tokio::test]
async fn test_get_user_without_access_token_returns_error() {
    let mut api = refused_api();
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    assert!(api.get_user("", 10, vec![]).await.is_err());
}
#[tokio::test]
async fn test_get_user_known_api_error_response() {
    let server = start_server().await;
    Mock::given(method("GET"))
        .and(path("/api/users/00000000-0000-0000-0000-000000000001"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "code": "UNAUTHORIZED",
            "description": "Invalid token"
        })))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.access_token = Some("bad-token".to_string());
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    let result = api.get_user("", 10, vec![]).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("UNAUTHORIZED"));
}
/// Hits the serde_json error branch inside `if !status.is_success()` (line 214)
#[tokio::test]
async fn test_get_user_error_response_with_invalid_json_body_returns_error() {
    let server = start_server().await;
    Mock::given(method("GET"))
        .and(path("/api/users/00000000-0000-0000-0000-000000000001"))
        .respond_with(ResponseTemplate::new(403).set_body_string("not json"))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.access_token = Some("token".to_string());
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    assert!(api.get_user("", 10, vec![]).await.is_err());
}
/// Hits the serde_json error branch for a 200 response with invalid JSON (line ~220)
#[tokio::test]
async fn test_get_user_invalid_json_success_response_returns_error() {
    let server = start_server().await;
    Mock::given(method("GET"))
        .and(path("/api/users/00000000-0000-0000-0000-000000000001"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.access_token = Some("token".to_string());
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    assert!(api.get_user("", 10, vec![]).await.is_err());
}
#[tokio::test]
async fn test_get_user_send_failure_returns_error() {
    let mut api = refused_api();
    api.access_token = Some("token".to_string());
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    assert!(api.get_user("", 10, vec![]).await.is_err());
}
// ---------------------------------------------------------------------------
// get_measurements — happy path + error paths
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_get_measurements_success() {
    let server = start_server().await;
    Mock::given(method("GET"))
        .and(path("/api/measurements"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(measurements_body()))
        .mount(&server)
        .await;
    let api = api(&server);
    let result = api.get_measurements(1).await.unwrap();
    assert_eq!(result.total, 1);
    assert_eq!(result.embedded.measurements[0].name.to_string(), "Squat");
}
#[tokio::test]
async fn test_get_measurements_invalid_json_returns_error() {
    let server = start_server().await;
    Mock::given(method("GET"))
        .and(path("/api/measurements"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    assert!(api(&server).get_measurements(1).await.is_err());
}
#[tokio::test]
async fn test_get_measurements_send_failure_returns_error() {
    assert!(refused_api().get_measurements(1).await.is_err());
}
// ---------------------------------------------------------------------------
// get_logs_raw — happy path + all error paths
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_get_logs_raw_success() {
    let server = start_server().await;
    Mock::given(method("GET"))
        .and(path("/api/logs/00000000-0000-0000-0000-000000000001"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"logs":[]}"#))
        .mount(&server)
        .await;
    let mut api = api(&server);
    api.access_token = Some("token".to_string());
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    let raw = api.get_logs_raw().await.unwrap();
    assert_eq!(raw, r#"{"logs":[]}"#);
}
#[tokio::test]
async fn test_get_logs_raw_without_user_id_returns_error() {
    let mut api = refused_api();
    api.access_token = Some("token".to_string());
    let result = api.get_logs_raw().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing user id"));
}
#[tokio::test]
async fn test_get_logs_raw_without_access_token_returns_error() {
    let mut api = refused_api();
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    let result = api.get_logs_raw().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing access token"));
}
#[tokio::test]
async fn test_get_logs_raw_send_failure_returns_error() {
    let mut api = refused_api();
    api.access_token = Some("token".to_string());
    api.user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
    assert!(api.get_logs_raw().await.is_err());
}
