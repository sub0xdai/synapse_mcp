use thiserror::Error;

pub type Result<T> = std::result::Result<T, SynapseError>;

#[derive(Error, Debug)]
pub enum SynapseError {
    // Standard library errors with automatic conversion
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Neo4j error: {0}")]
    Neo4j(#[from] neo4rs::Error),
    
    #[error("Database error: {0}")]
    Database(String),
    
    // HTTP API specific errors
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    // Rule enforcement errors
    #[error("Rule violation: {0}")]
    RuleViolation(String),
    
    // Application logic errors
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

// Additional From implementations for common error types
impl From<&str> for SynapseError {
    fn from(msg: &str) -> Self {
        SynapseError::Internal(msg.to_string())
    }
}

impl From<String> for SynapseError {
    fn from(msg: String) -> Self {
        SynapseError::Internal(msg)
    }
}

// Convert from anyhow::Error for CLI integration
impl From<anyhow::Error> for SynapseError {
    fn from(err: anyhow::Error) -> Self {
        SynapseError::Internal(err.to_string())
    }
}