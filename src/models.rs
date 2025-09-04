use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NodeType {
    File,
    Rule,
    Decision,
    Function,
    Architecture,
    Component,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub content: String,
    pub tags: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EdgeType {
    RelatesTo,
    ImplementsRule,
    DefinedIn,
    DependsOn,
    Contains,
    References,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub label: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Node {
    pub fn new(node_type: NodeType, label: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            node_type,
            label,
            content,
            tags: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.label.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Label cannot be empty".to_string()));
        }
        if self.content.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Content cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl Edge {
    pub fn new(source_id: String, target_id: String, edge_type: EdgeType, label: String) -> Self {
        Self {
            source_id,
            target_id,
            edge_type,
            label,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.source_id.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Source ID cannot be empty".to_string()));
        }
        if self.target_id.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Target ID cannot be empty".to_string()));
        }
        if self.source_id == self.target_id {
            return Err(crate::SynapseError::Validation("Source and target cannot be the same".to_string()));
        }
        Ok(())
    }
}