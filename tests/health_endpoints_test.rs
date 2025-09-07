//! Tests for health check endpoints
//! Following TDD principles - tests written first

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::Value;
use synapse_mcp::{
    graph::Graph,
    mcp_server::create_server_with_auth,
    Config,
};
use tokio;

/// Helper function to create a test server with minimal dependencies
async fn create_test_server() -> TestServer {
    // Create a test Neo4j configuration
    let config = Config::default();
    
    // Create a graph connection (may fail in CI, but that's ok for testing error handling)
    let graph_result = Graph::new_direct(
        &config.neo4j.uri,
        &config.neo4j.user,
        &config.neo4j.password,
    ).await;
    
    let graph = match graph_result {
        Ok(graph) => graph,
        Err(_) => {
            // For tests, we'll create a mock or skip DB-dependent tests
            // In a real implementation, we'd use a test database
            panic!("Test database not available - configure test Neo4j instance");
        }
    };
    
    let app = create_server_with_auth(graph, None, None).await;
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_health_endpoint_returns_200_ok() {
    let server = create_test_server().await;
    
    let response = server.get("/health").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let text = response.text();
    assert_eq!(text, "OK");
}

#[tokio::test]
async fn test_health_endpoint_has_correct_content_type() {
    let server = create_test_server().await;
    
    let response = server.get("/health").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // Should be plain text for simple health check
    let content_type = response.headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok());
    
    assert!(content_type.map_or(false, |ct| ct.starts_with("text/plain")));
}

#[tokio::test]
async fn test_status_endpoint_returns_valid_json() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let json: Value = response.json();
    
    // Verify required top-level fields
    assert!(json["status"].is_string());
    assert!(json["version"].is_string());
    assert!(json["uptime_seconds"].is_number());
    assert!(json["dependencies"].is_object());
    assert!(json["system"].is_object());
}

#[tokio::test]
async fn test_status_endpoint_contains_neo4j_info() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    let json: Value = response.json();
    
    let neo4j_status = &json["dependencies"]["neo4j"];
    
    assert!(neo4j_status["status"].is_string());
    assert!(neo4j_status["latency_ms"].is_number());
    
    let pool_info = &neo4j_status["connection_pool"];
    assert!(pool_info["active"].is_number());
    assert!(pool_info["idle"].is_number());
    assert!(pool_info["max"].is_number());
}

#[tokio::test]
async fn test_status_endpoint_contains_system_info() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    let json: Value = response.json();
    
    let system_info = &json["system"];
    
    assert!(system_info["memory_used_mb"].is_number());
    assert!(system_info["memory_available_mb"].is_number());
    assert!(system_info["cpu_usage_percent"].is_number());
}

#[tokio::test]
async fn test_status_endpoint_when_healthy() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    let json: Value = response.json();
    
    // When all dependencies are healthy, overall status should be "healthy"
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["dependencies"]["neo4j"]["status"], "healthy");
}

#[tokio::test]
async fn test_status_endpoint_content_type_is_json() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    
    let content_type = response.headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok());
    
    assert!(content_type.map_or(false, |ct| ct.starts_with("application/json")));
}

#[tokio::test]
async fn test_status_endpoint_version_matches_cargo_toml() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    let json: Value = response.json();
    
    // Version should match the version in Cargo.toml
    let version = json["version"].as_str().unwrap();
    assert!(!version.is_empty());
    
    // Basic version format validation (semver-like)
    assert!(version.matches('.').count() >= 2 || version.contains("0."));
}

#[tokio::test]
async fn test_status_endpoint_uptime_is_positive() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    let json: Value = response.json();
    
    let uptime = json["uptime_seconds"].as_f64().unwrap();
    assert!(uptime >= 0.0);
}

// Test for cache status (when cache is enabled)
#[tokio::test]
async fn test_status_endpoint_contains_cache_info_when_enabled() {
    let server = create_test_server().await;
    
    let response = server.get("/status").await;
    let json: Value = response.json();
    
    // Cache should be reported if enabled in config
    if json["dependencies"]["cache"].is_object() {
        let cache_status = &json["dependencies"]["cache"];
        assert!(cache_status["status"].is_string());
        assert!(cache_status["hit_rate"].is_number());
        assert!(cache_status["entries"].is_number());
    }
}

// Test error handling when Neo4j is unavailable
#[tokio::test]
async fn test_status_endpoint_handles_database_unavailable() {
    // This test would require a way to mock or disable the database connection
    // For now, we'll implement the test structure and add implementation later
    
    // When implemented, this should:
    // 1. Create server with invalid database config
    // 2. Call /status endpoint
    // 3. Verify status is "unhealthy" or "degraded"
    // 4. Verify Neo4j status is "unhealthy"
    // 5. Verify HTTP status code is still 200 (for monitoring systems)
}

#[tokio::test] 
async fn test_health_and_status_endpoints_are_fast() {
    let server = create_test_server().await;
    
    use std::time::Instant;
    
    // Health check should be very fast (< 100ms)
    let start = Instant::now();
    let response = server.get("/health").await;
    let health_duration = start.elapsed();
    
    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(health_duration.as_millis() < 100, "Health check too slow: {}ms", health_duration.as_millis());
    
    // Status check should be reasonably fast (< 1000ms)
    let start = Instant::now();
    let response = server.get("/status").await;
    let status_duration = start.elapsed();
    
    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(status_duration.as_millis() < 1000, "Status check too slow: {}ms", status_duration.as_millis());
}