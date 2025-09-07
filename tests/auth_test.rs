use std::env;
use std::time::Instant;
use synapse_mcp::Config;

/// Test loading auth token from environment variable
#[tokio::test]
async fn test_config_loads_auth_token_from_env() {
    let test_token = "test_bearer_token_123";
    unsafe {
        env::set_var("SYNAPSE_AUTH_TOKEN", test_token);
    }
    
    let config = Config::load().expect("Config should load");
    
    assert_eq!(
        config.server.auth_token.as_deref(), 
        Some(test_token),
        "Auth token should be loaded from environment"
    );
    
    unsafe {
        env::remove_var("SYNAPSE_AUTH_TOKEN");
    }
}

/// Test config handles missing auth token gracefully
#[tokio::test] 
async fn test_config_handles_missing_auth_token() {
    unsafe {
        env::remove_var("SYNAPSE_AUTH_TOKEN");
    }
    
    let config = Config::load().expect("Config should load even without auth token");
    
    assert!(
        config.server.auth_token.is_none(),
        "Auth token should be None when not set in environment"
    );
}

/// Test config serialization includes auth token
#[tokio::test]
async fn test_config_serialization_with_auth_token() {
    let test_token = "serialization_test_token";
    unsafe {
        env::set_var("SYNAPSE_AUTH_TOKEN", test_token);
    }
    
    let config = Config::load().expect("Config should load");
    let serialized = serde_json::to_string(&config)
        .expect("Config should serialize to JSON");
    
    // Auth token should not be serialized for security
    assert!(
        !serialized.contains(test_token),
        "Auth token should not appear in serialized config for security"
    );
    
    unsafe {
        env::remove_var("SYNAPSE_AUTH_TOKEN");
    }
}

#[cfg(test)]
mod middleware_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, HeaderMap, HeaderValue},
        middleware::Next,
        response::Response,
    };
    use std::collections::HashMap;

    /// Test bearer token extraction from Authorization header
    #[tokio::test]
    async fn test_extract_bearer_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization", 
            HeaderValue::from_static("Bearer valid_token_123")
        );
        
        let result = synapse_mcp::auth::extract_bearer_token(&headers);
        assert_eq!(result, Some("valid_token_123"));
    }

    /// Test bearer token extraction handles missing header
    #[tokio::test]
    async fn test_extract_bearer_token_missing_header() {
        let headers = HeaderMap::new();
        let result = synapse_mcp::auth::extract_bearer_token(&headers);
        assert_eq!(result, None);
    }

    /// Test bearer token extraction handles malformed header
    #[tokio::test]
    async fn test_extract_bearer_token_malformed() {
        let mut headers = HeaderMap::new();
        
        // Test various malformed headers
        headers.insert("authorization", HeaderValue::from_static("Basic username:password"));
        assert_eq!(synapse_mcp::auth::extract_bearer_token(&headers), None);
        
        headers.insert("authorization", HeaderValue::from_static("Bearer"));
        assert_eq!(synapse_mcp::auth::extract_bearer_token(&headers), None);
        
        headers.insert("authorization", HeaderValue::from_static("bearer lowercase_not_accepted"));
        assert_eq!(synapse_mcp::auth::extract_bearer_token(&headers), None);
    }

    /// Test creating auth middleware with token
    #[tokio::test]
    async fn test_auth_middleware_creation() {
        let middleware = synapse_mcp::auth::AuthMiddleware::new(Some("test_token".to_string()));
        // If it compiles and runs without panic, the creation works
        assert!(true);
        
        let middleware_no_auth = synapse_mcp::auth::AuthMiddleware::new(None);
        assert!(true);
    }
}