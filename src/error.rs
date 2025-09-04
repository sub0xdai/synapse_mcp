use thiserror::Error;

pub type Result<T> = std::result::Result<T, SynapseError>;

#[derive(Error, Debug)]
pub enum SynapseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Neo4j error: {0}")]
    Neo4j(#[from] neo4rs::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
}