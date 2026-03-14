use strong_api_lib::models::error::ApiErrorResponse;

#[test]
fn test_api_error_display() {
    let err = ApiErrorResponse {
        code: "AUTH_001".to_string(),
        description: "Invalid credentials".to_string(),
    };
    assert_eq!(err.to_string(), "AUTH_001: Invalid credentials");
}

#[test]
fn test_api_error_is_std_error() {
    let err = ApiErrorResponse {
        code: "ERR".to_string(),
        description: "something went wrong".to_string(),
    };
    // Verify it satisfies std::error::Error (compile-time + source() returns None)
    let boxed: Box<dyn std::error::Error> = Box::new(err);
    assert!(boxed.source().is_none());
}

#[test]
fn test_api_error_deserializes() {
    let json = r#"{"code":"NOT_FOUND","description":"Resource not found"}"#;
    let err: ApiErrorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(err.code, "NOT_FOUND");
    assert_eq!(err.description, "Resource not found");
}

