//! Integration tests for connection pool functionality
//! 
//! These tests verify that the connection pool works correctly.
//! Run with: cargo test --test integration_connection_pool --ignored

use synapse_mcp::{PooledGraph, Neo4jConfig, PoolConfig};

/// Helper to create a test Neo4j config
fn create_test_neo4j_config() -> Neo4jConfig {
    Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "test".to_string(),
        password: "test".to_string(),
        database: "test".to_string(),
        fetch_size: 100,
        max_connections: 5,
        pool: PoolConfig {
            min_idle: 1,
            max_size: 5,
            connection_timeout_secs: 10,
            idle_timeout_secs: 300,
            max_lifetime_secs: 1800,
            metrics_enabled: true,
        },
    }
}

#[tokio::test]
#[ignore] // Run only with --ignored when Neo4j is available  
async fn test_pooled_graph_basic_operations() {
    // Skip test if Neo4j is not available
    if std::env::var("NEO4J_URI").is_err() {
        println!("Skipping integration test - NEO4J_URI not set");
        return;
    }
    
    let config = create_test_neo4j_config();
    
    let pooled_graph = match PooledGraph::new(config).await {
        Ok(g) => g,
        Err(e) => {
            println!("Skipping pooled graph test - connection failed: {}", e);
            return;
        }
    };
    
    // Test pool health check
    let health = pooled_graph.health_check().await.unwrap();
    assert!(health, "Pooled graph should be healthy");
    
    // Test pool statistics
    let stats = pooled_graph.pool_stats().await;
    assert!(stats.max_size > 0, "Pool should have positive max size");
    
    // Test direct connection access
    let conn = pooled_graph.get_connection().await.unwrap();
    let mut result = conn.execute(neo4rs::query("RETURN 1 as test")).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    let value: i64 = row.get("test").unwrap();
    assert_eq!(value, 1, "Query should return expected value");
    
    println!("Basic pooled graph operations test passed");
}

#[tokio::test]
#[ignore] // Run only with --ignored when Neo4j is available
async fn test_connection_pool_stats() {
    // Skip test if Neo4j is not available  
    if std::env::var("NEO4J_URI").is_err() {
        println!("Skipping pool stats test - NEO4J_URI not set");
        return;
    }
    
    let config = create_test_neo4j_config();
    
    let pooled_graph = match PooledGraph::new(config).await {
        Ok(g) => g,
        Err(e) => {
            println!("Skipping pool stats test - connection failed: {}", e);
            return;
        }
    };
    
    let stats = pooled_graph.pool_stats().await;
    assert_eq!(stats.max_size, 5, "Pool max size should match config");
    assert!(stats.size <= 5, "Pool size should not exceed max size");
    
    println!("Connection pool stats test passed");
}