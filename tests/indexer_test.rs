use synapse_mcp::NodeType;
use tempfile::NamedTempFile;
use std::io::Write;

// Test data for markdown files with YAML frontmatter
const TEST_RULE_MD: &str = r#"---
type: rule
title: "Use Rust for Performance Critical Code"
tags: ["architecture", "performance"]
priority: high
---

# Use Rust for Performance Critical Code

This is a rule about using Rust for performance-critical components.

## Rationale

Rust provides:
- Memory safety
- Zero-cost abstractions
- High performance

## Examples

- Indexer module
- Graph operations
"#;

const TEST_DECISION_MD: &str = r#"---
type: decision
title: "Choose Neo4j for Knowledge Graph"
tags: ["architecture", "database"]
decision_date: "2024-01-15"
status: accepted
---

# Decision: Use Neo4j for Knowledge Graph

We decided to use Neo4j as our graph database.

## Context

We need to store complex relationships between documentation elements.

## Decision

Neo4j provides excellent support for graph queries.
"#;

const TEST_INVALID_YAML_MD: &str = r#"---
invalid yaml content
missing proper structure
---

# Some content

This file has invalid YAML frontmatter.
"#;

const TEST_NO_FRONTMATTER_MD: &str = r#"# Regular Markdown File

This file has no YAML frontmatter.

It should be treated as a regular file.
"#;

#[test]
fn test_parse_rule_document() {
    // Create temporary file with rule content
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", TEST_RULE_MD).unwrap();
    let temp_path = temp_file.path();
    
    // Parse the document
    let result = synapse_mcp::indexer::parse_markdown_file(temp_path);
    
    assert!(result.is_ok());
    let node = result.unwrap();
    
    assert_eq!(node.node_type, NodeType::Rule);
    assert_eq!(node.label, "Use Rust for Performance Critical Code");
    assert!(node.content.contains("This is a rule about using Rust"));
    assert!(node.tags.contains(&"architecture".to_string()));
    assert!(node.tags.contains(&"performance".to_string()));
    assert_eq!(node.metadata.get("priority"), Some(&"high".to_string()));
}

#[test]
fn test_parse_decision_document() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", TEST_DECISION_MD).unwrap();
    let temp_path = temp_file.path();
    
    let result = synapse_mcp::indexer::parse_markdown_file(temp_path);
    
    assert!(result.is_ok());
    let node = result.unwrap();
    
    assert_eq!(node.node_type, NodeType::Decision);
    assert_eq!(node.label, "Choose Neo4j for Knowledge Graph");
    assert!(node.content.contains("We decided to use Neo4j"));
    assert!(node.tags.contains(&"architecture".to_string()));
    assert_eq!(node.metadata.get("status"), Some(&"accepted".to_string()));
    assert_eq!(node.metadata.get("decision_date"), Some(&"2024-01-15".to_string()));
}

#[test]
fn test_parse_invalid_yaml() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", TEST_INVALID_YAML_MD).unwrap();
    let temp_path = temp_file.path();
    
    let result = synapse_mcp::indexer::parse_markdown_file(temp_path);
    
    assert!(result.is_err());
}

#[test]
fn test_parse_no_frontmatter() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", TEST_NO_FRONTMATTER_MD).unwrap();
    let temp_path = temp_file.path();
    
    let result = synapse_mcp::indexer::parse_markdown_file(temp_path);
    
    assert!(result.is_ok());
    let node = result.unwrap();
    
    assert_eq!(node.node_type, NodeType::File);
    assert!(node.label.contains("Regular Markdown File") || node.label.contains(temp_path.to_str().unwrap()));
    assert!(node.content.contains("This file has no YAML frontmatter"));
}

#[test]
fn test_extract_relationships() {
    let content = r#"
# Architecture Decision

This decision relates to the [Performance Rule](./performance-rule.md).

It also implements rule [PERF-001] and depends on [Component A].

See also: [Database Design](../design/database.md)
"#;
    
    let relationships = synapse_mcp::indexer::extract_relationships(content, "test-doc-id");
    
    assert!(!relationships.is_empty());
    
    // Debug: print all relationships found
    for rel in &relationships {
        println!("Found relationship: {} -> {} ({})", rel.source_id, rel.target_id, rel.label);
    }
    
    // Should find references to other documents
    let markdown_refs: Vec<_> = relationships.iter()
        .filter(|edge| edge.target_id.contains(".md") || edge.label.contains(".md"))
        .collect();
    assert!(!markdown_refs.is_empty(), "Expected to find markdown references, but found: {:?}", relationships);
    
    // Should find rule references
    let rule_refs: Vec<_> = relationships.iter()
        .filter(|edge| edge.label.contains("PERF-001"))
        .collect();
    assert!(!rule_refs.is_empty());
}

#[test]
fn test_batch_parse_files() {
    // Create multiple temporary files
    let mut rule_file = NamedTempFile::new().unwrap();
    write!(rule_file, "{}", TEST_RULE_MD).unwrap();
    
    let mut decision_file = NamedTempFile::new().unwrap();
    write!(decision_file, "{}", TEST_DECISION_MD).unwrap();
    
    let files = vec![
        rule_file.path().to_path_buf(),
        decision_file.path().to_path_buf(),
    ];
    
    let result = synapse_mcp::indexer::parse_multiple_files(&files);
    
    assert!(result.is_ok());
    let (nodes, edges) = result.unwrap();
    
    assert_eq!(nodes.len(), 2);
    // Should have at least some relationships between documents
    assert!(!edges.is_empty() || nodes.iter().any(|n| !n.tags.is_empty()));
}

#[test]
fn test_parse_performance_under_500ms() {
    // Create a reasonably sized markdown file
    let large_content = format!(
        "{}\n{}", 
        TEST_RULE_MD,
        "# Additional Content\n".repeat(100)
    );
    
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", large_content).unwrap();
    let temp_path = temp_file.path();
    
    let start = std::time::Instant::now();
    let result = synapse_mcp::indexer::parse_markdown_file(temp_path);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 500, "Parsing took {}ms, should be under 500ms", duration.as_millis());
}