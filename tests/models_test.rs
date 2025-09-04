use synapse_mcp::{Node, Edge, NodeType, EdgeType};
use std::collections::HashMap;

#[test]
fn test_node_creation() {
    let node = Node::new(
        NodeType::Rule,
        "Test Rule".to_string(),
        "This is a test rule".to_string(),
    );
    
    assert_eq!(node.node_type, NodeType::Rule);
    assert_eq!(node.label, "Test Rule");
    assert_eq!(node.content, "This is a test rule");
    assert!(!node.id.is_empty());
    assert!(node.tags.is_empty());
    assert!(node.metadata.is_empty());
}

#[test]
fn test_node_with_tags_and_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("priority".to_string(), "high".to_string());
    
    let node = Node::new(
        NodeType::Decision,
        "Architecture Decision".to_string(),
        "We decided to use Rust".to_string(),
    )
    .with_tags(vec!["architecture".to_string(), "rust".to_string()])
    .with_metadata(metadata.clone());
    
    assert_eq!(node.tags.len(), 2);
    assert!(node.tags.contains(&"architecture".to_string()));
    assert_eq!(node.metadata.get("priority"), Some(&"high".to_string()));
}

#[test]
fn test_node_validation_success() {
    let node = Node::new(
        NodeType::File,
        "main.rs".to_string(),
        "fn main() {}".to_string(),
    );
    
    assert!(node.validate().is_ok());
}

#[test]
fn test_node_validation_empty_label() {
    let node = Node::new(
        NodeType::File,
        "".to_string(),
        "some content".to_string(),
    );
    
    assert!(node.validate().is_err());
}

#[test]
fn test_node_validation_empty_content() {
    let node = Node::new(
        NodeType::Rule,
        "Some rule".to_string(),
        "".to_string(),
    );
    
    assert!(node.validate().is_err());
}

#[test]
fn test_edge_creation() {
    let edge = Edge::new(
        "node1".to_string(),
        "node2".to_string(),
        EdgeType::RelatesTo,
        "relates to".to_string(),
    );
    
    assert_eq!(edge.source_id, "node1");
    assert_eq!(edge.target_id, "node2");
    assert_eq!(edge.edge_type, EdgeType::RelatesTo);
    assert_eq!(edge.label, "relates to");
    assert!(edge.metadata.is_empty());
}

#[test]
fn test_edge_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("strength".to_string(), "strong".to_string());
    
    let edge = Edge::new(
        "rule1".to_string(),
        "impl1".to_string(),
        EdgeType::ImplementsRule,
        "implements".to_string(),
    ).with_metadata(metadata.clone());
    
    assert_eq!(edge.metadata.get("strength"), Some(&"strong".to_string()));
}

#[test]
fn test_edge_validation_success() {
    let edge = Edge::new(
        "source".to_string(),
        "target".to_string(),
        EdgeType::DependsOn,
        "depends on".to_string(),
    );
    
    assert!(edge.validate().is_ok());
}

#[test]
fn test_edge_validation_empty_source() {
    let edge = Edge::new(
        "".to_string(),
        "target".to_string(),
        EdgeType::Contains,
        "contains".to_string(),
    );
    
    assert!(edge.validate().is_err());
}

#[test]
fn test_edge_validation_empty_target() {
    let edge = Edge::new(
        "source".to_string(),
        "".to_string(),
        EdgeType::References,
        "references".to_string(),
    );
    
    assert!(edge.validate().is_err());
}

#[test]
fn test_edge_validation_same_source_and_target() {
    let edge = Edge::new(
        "same".to_string(),
        "same".to_string(),
        EdgeType::RelatesTo,
        "self reference".to_string(),
    );
    
    assert!(edge.validate().is_err());
}

#[test]
fn test_node_serialization() {
    let node = Node::new(
        NodeType::Component,
        "Parser".to_string(),
        "Markdown parser component".to_string(),
    );
    
    let json = serde_json::to_string(&node).unwrap();
    let deserialized: Node = serde_json::from_str(&json).unwrap();
    
    assert_eq!(node.id, deserialized.id);
    assert_eq!(node.node_type, deserialized.node_type);
    assert_eq!(node.label, deserialized.label);
    assert_eq!(node.content, deserialized.content);
}

#[test]
fn test_edge_serialization() {
    let edge = Edge::new(
        "comp1".to_string(),
        "comp2".to_string(),
        EdgeType::DependsOn,
        "dependency".to_string(),
    );
    
    let json = serde_json::to_string(&edge).unwrap();
    let deserialized: Edge = serde_json::from_str(&json).unwrap();
    
    assert_eq!(edge.source_id, deserialized.source_id);
    assert_eq!(edge.target_id, deserialized.target_id);
    assert_eq!(edge.edge_type, deserialized.edge_type);
    assert_eq!(edge.label, deserialized.label);
}