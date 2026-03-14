use strong_api_lib::models::common::Name;
use strong_api_lib::models::measurement::MeasurementsResponse;

fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    ))
    .unwrap_or_else(|_| panic!("fixture '{name}' not found"))
}

fn response_from_fixture() -> MeasurementsResponse {
    let json = load_fixture("measurements_response.json");
    serde_json::from_str(&json).unwrap()
}

// ---------------------------------------------------------------------------
// Name — Display and From impls (models/common.rs)
// These are the only non-trivial logic in common.rs; each branch needs a test.
// ---------------------------------------------------------------------------

#[test]
fn test_name_display_en() {
    let name = Name { en: Some("Bench Press".to_string()), custom: None };
    assert_eq!(name.to_string(), "Bench Press");
}

#[test]
fn test_name_display_custom_fallback() {
    let name = Name { en: None, custom: Some("My Exercise".to_string()) };
    assert_eq!(name.to_string(), "My Exercise");
}

#[test]
fn test_name_display_unknown_fallback() {
    let name = Name { en: None, custom: None };
    assert_eq!(name.to_string(), "Unknown");
}

#[test]
fn test_name_from_name_en() {
    let name = Name { en: Some("Squat".to_string()), custom: None };
    assert_eq!(String::from(name), "Squat");
}

#[test]
fn test_name_from_name_custom_fallback() {
    let name = Name { en: None, custom: Some("Custom".to_string()) };
    assert_eq!(String::from(name), "Custom");
}

#[test]
fn test_name_from_name_unknown_fallback() {
    let name = Name { en: None, custom: None };
    assert_eq!(String::from(name), "Unknown");
}

#[test]
fn test_name_from_string() {
    let name = Name::from("Deadlift".to_string());
    assert_eq!(name.en.as_deref(), Some("Deadlift"));
    assert!(name.custom.is_none());
}

// ---------------------------------------------------------------------------
// MeasurementsResponse::merge — the only hand-written logic in measurement.rs
// ---------------------------------------------------------------------------

#[test]
fn test_measurements_merge() {
    let a = response_from_fixture();
    let b = response_from_fixture();
    let original_count = a.embedded.measurements.len();
    let merged = a.merge(b);
    assert_eq!(merged.embedded.measurements.len(), original_count * 2);
}

// ---------------------------------------------------------------------------
// Deserialization with no next link (covers the None branch of Links.next)
// ---------------------------------------------------------------------------

#[test]
fn test_measurements_response_no_next_link() {
    let json = r#"{
        "_links": { "self": { "href": "/api/measurements?page=99" } },
        "total": 0,
        "_embedded": { "measurement": [] }
    }"#;
    let response: MeasurementsResponse = serde_json::from_str(json).unwrap();
    assert!(response.links.next.is_none());
    assert!(response.embedded.measurements.is_empty());
}
