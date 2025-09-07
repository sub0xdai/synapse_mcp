use std::env;

/// Test that authentication token configuration works
#[tokio::test]
async fn test_auth_token_environment_integration() {
    // Test that the auth token is properly loaded from environment
    unsafe {
        env::set_var("SYNAPSE_AUTH_TOKEN", "integration_test_token");
    }
    
    let config = synapse_mcp::Config::load().expect("Config should load");
    
    assert_eq!(
        config.server.auth_token.as_deref(),
        Some("integration_test_token"),
        "Auth token should be loaded from environment variable"
    );
    
    // Test serialization security - token should not appear in serialized output
    let serialized = serde_json::to_string(&config)
        .expect("Config should serialize");
        
    assert!(
        !serialized.contains("integration_test_token"),
        "Auth token should not appear in serialized config"
    );
    
    unsafe {
        env::remove_var("SYNAPSE_AUTH_TOKEN");
    }
}

/// Test that authentication middleware can be created
#[tokio::test]
async fn test_auth_middleware_creation() {
    // Test creating middleware with token
    let _middleware_with_auth = synapse_mcp::auth::AuthMiddleware::new(Some("test_token".to_string()));
    
    // Test creating middleware without token (auth disabled)
    let _middleware_no_auth = synapse_mcp::auth::AuthMiddleware::new(None);
    
    // If we get here without panicking, middleware creation works
    assert!(true);
}

/// Test bearer token extraction functionality
#[tokio::test]
async fn test_bearer_token_extraction() {
    use axum::http::{HeaderMap, HeaderValue};
    
    // Test valid bearer token
    let mut headers = HeaderMap::new();
    headers.insert("authorization", HeaderValue::from_static("Bearer valid_token_123"));
    
    let result = synapse_mcp::auth::extract_bearer_token(&headers);
    assert_eq!(result, Some("valid_token_123"));
    
    // Test missing header
    let empty_headers = HeaderMap::new();
    let result = synapse_mcp::auth::extract_bearer_token(&empty_headers);
    assert_eq!(result, None);
    
    // Test malformed header
    let mut malformed_headers = HeaderMap::new();
    malformed_headers.insert("authorization", HeaderValue::from_static("Basic user:pass"));
    let result = synapse_mcp::auth::extract_bearer_token(&malformed_headers);
    assert_eq!(result, None);
}