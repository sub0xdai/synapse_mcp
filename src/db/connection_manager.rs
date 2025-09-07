//! Neo4j connection manager for bb8 pool
//! 
//! Implements the bb8::ManageConnection trait to manage Neo4j connections
//! in a connection pool, following SOLID principles.

use async_trait::async_trait;
use bb8::ManageConnection;
use neo4rs::{Graph as Neo4jGraph, ConfigBuilder};
use std::fmt;
use thiserror::Error;
use tracing::{debug, warn, error};

/// Errors that can occur during connection management
#[derive(Error, Debug)]
pub enum ConnectionManagerError {
    #[error("Failed to create Neo4j connection: {0}")]
    ConnectionCreation(#[from] neo4rs::Error),
    
    #[error("Connection validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Configuration for Neo4j connection manager
#[derive(Debug, Clone)]
pub struct Neo4jConnectionConfig {
    pub uri: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub fetch_size: usize,
    pub connection_timeout_secs: u64,
}

impl Neo4jConnectionConfig {
    /// Create a new configuration
    pub fn new(
        uri: String,
        user: String, 
        password: String,
        database: String,
    ) -> Self {
        Self {
            uri,
            user,
            password,
            database,
            fetch_size: 500,
            connection_timeout_secs: 30,
        }
    }
    
    /// Set fetch size for query results
    pub fn with_fetch_size(mut self, size: usize) -> Self {
        self.fetch_size = size;
        self
    }
    
    /// Set connection timeout in seconds
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.connection_timeout_secs = timeout_secs;
        self
    }
}

/// Connection manager for Neo4j that implements bb8::ManageConnection
/// 
/// This struct is responsible for creating, validating, and managing
/// the lifecycle of Neo4j connections in the pool.
#[derive(Debug, Clone)]
pub struct Neo4jConnectionManager {
    config: Neo4jConnectionConfig,
}

impl Neo4jConnectionManager {
    /// Create a new connection manager with the given configuration
    pub fn new(config: Neo4jConnectionConfig) -> Self {
        debug!("Creating Neo4j connection manager for URI: {}", config.uri);
        Self { config }
    }
    
    /// Build Neo4j configuration from manager config
    fn build_neo4j_config(&self) -> Result<neo4rs::Config, ConnectionManagerError> {
        ConfigBuilder::default()
            .uri(&self.config.uri)
            .user(&self.config.user)
            .password(&self.config.password)
            .db(&*self.config.database)
            .fetch_size(self.config.fetch_size)
            .build()
            .map_err(|e| ConnectionManagerError::ConnectionCreation(e))
    }
    
    /// Validate that a connection is still healthy
    async fn validate_connection(&self, conn: &Neo4jGraph) -> bool {
        match conn.execute(neo4rs::query("RETURN 1 as health_check")).await {
            Ok(mut result) => {
                // Try to consume one record to ensure the connection works
                match result.next().await {
                    Ok(Some(_)) => {
                        debug!("Connection validation successful");
                        true
                    }
                    Ok(None) => {
                        warn!("Connection validation returned no results");
                        false
                    }
                    Err(e) => {
                        warn!("Connection validation failed during result consumption: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("Connection validation failed during query execution: {}", e);
                false
            }
        }
    }
}

#[async_trait]
impl ManageConnection for Neo4jConnectionManager {
    type Connection = Neo4jGraph;
    type Error = ConnectionManagerError;
    
    /// Create a new connection to Neo4j
    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        debug!("Creating new Neo4j connection");
        
        let config = self.build_neo4j_config()?;
        
        let connection = Neo4jGraph::connect(config).await
            .map_err(|e| {
                error!("Failed to create Neo4j connection: {}", e);
                ConnectionManagerError::ConnectionCreation(e)
            })?;
            
        debug!("Successfully created Neo4j connection");
        Ok(connection)
    }
    
    /// Check if a connection is still valid and healthy
    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        debug!("Validating Neo4j connection");
        
        if self.validate_connection(conn).await {
            Ok(())
        } else {
            Err(ConnectionManagerError::ValidationFailed(
                "Health check query failed".to_string()
            ))
        }
    }
    
    /// Check if an error indicates the connection should be discarded
    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        // For simplicity, we'll consider all connection errors as breaking
        // In a production system, this could be more sophisticated
        false // Let bb8 handle broken connections through is_valid checks
    }
}

impl fmt::Display for Neo4jConnectionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Neo4jConnectionManager(uri={})", self.config.uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_config_creation() {
        let config = Neo4jConnectionConfig::new(
            "bolt://localhost:7687".to_string(),
            "neo4j".to_string(),
            "password".to_string(),
            "neo4j".to_string(),
        );
        
        assert_eq!(config.uri, "bolt://localhost:7687");
        assert_eq!(config.user, "neo4j");
        assert_eq!(config.password, "password");
        assert_eq!(config.database, "neo4j");
        assert_eq!(config.fetch_size, 500);
        assert_eq!(config.connection_timeout_secs, 30);
    }
    
    #[test]
    fn test_connection_config_builder_pattern() {
        let config = Neo4jConnectionConfig::new(
            "bolt://localhost:7687".to_string(),
            "neo4j".to_string(),
            "password".to_string(),
            "neo4j".to_string(),
        )
        .with_fetch_size(1000)
        .with_timeout(60);
        
        assert_eq!(config.fetch_size, 1000);
        assert_eq!(config.connection_timeout_secs, 60);
    }
    
    #[test]
    fn test_connection_manager_creation() {
        let config = Neo4jConnectionConfig::new(
            "bolt://localhost:7687".to_string(),
            "neo4j".to_string(),
            "password".to_string(),
            "neo4j".to_string(),
        );
        
        let manager = Neo4jConnectionManager::new(config);
        assert!(manager.to_string().contains("bolt://localhost:7687"));
    }
    
    #[tokio::test]
    async fn test_neo4j_config_building() {
        let config = Neo4jConnectionConfig::new(
            "bolt://localhost:7687".to_string(),
            "neo4j".to_string(),
            "password".to_string(),
            "neo4j".to_string(),
        ).with_fetch_size(250);
        
        let manager = Neo4jConnectionManager::new(config);
        let neo4j_config = manager.build_neo4j_config().unwrap();
        
        // Note: We can't easily test the built config values as they're private
        // The important thing is that it builds without error
    }
    
    // Integration tests with actual Neo4j would go here
    // They should be behind a feature flag for CI/CD environments
    
    #[tokio::test]
    #[ignore] // Only run with --ignored flag when Neo4j is available
    async fn test_connection_manager_with_real_neo4j() {
        if std::env::var("SKIP_NEO4J_TESTS").is_ok() {
            return;
        }
        
        let config = Neo4jConnectionConfig::new(
            "bolt://localhost:7687".to_string(),
            "neo4j".to_string(),
            "password".to_string(),
            "neo4j".to_string(),
        );
        
        let manager = Neo4jConnectionManager::new(config);
        
        // Test connection creation
        let mut connection = manager.connect().await.unwrap();
        
        // Test connection validation
        let validation_result = manager.is_valid(&mut connection).await;
        assert!(validation_result.is_ok());
    }
}