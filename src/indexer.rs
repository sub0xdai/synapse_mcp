use crate::{Node, Edge, NodeType, EdgeType, Result, SynapseError};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;

#[derive(Debug, serde::Deserialize)]
struct FrontMatter {
    #[serde(rename = "type")]
    doc_type: Option<String>,
    title: Option<String>,
    tags: Option<Vec<String>>,
    #[serde(flatten)]
    metadata: HashMap<String, serde_yaml::Value>,
}

pub fn parse_markdown_file<P: AsRef<Path>>(path: P) -> Result<Node> {
    let content = fs::read_to_string(&path)?;
    let path_str = path.as_ref().to_string_lossy().to_string();
    
    if let Some((frontmatter_str, body)) = extract_frontmatter(&content) {
        parse_with_frontmatter(frontmatter_str, body, &path_str)
    } else {
        parse_without_frontmatter(&content, &path_str)
    }
}

pub fn parse_multiple_files(paths: &[std::path::PathBuf]) -> Result<(Vec<Node>, Vec<Edge>)> {
    let mut nodes = Vec::new();
    let mut all_edges = Vec::new();
    
    // Parse all files first
    for path in paths {
        match parse_markdown_file(path) {
            Ok(node) => nodes.push(node),
            Err(e) => eprintln!("Warning: Failed to parse {}: {}", path.display(), e),
        }
    }
    
    // Extract relationships between all documents
    for node in &nodes {
        let edges = extract_relationships(&node.content, &node.id);
        all_edges.extend(edges);
    }
    
    Ok((nodes, all_edges))
}

pub fn extract_relationships(content: &str, source_id: &str) -> Vec<Edge> {
    let mut edges = Vec::new();
    
    // Regex patterns for different types of references
    let markdown_link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+\.md)\)").unwrap();
    let rule_ref_re = Regex::new(r"\[([A-Z]+-\d+)\]").unwrap();
    let component_ref_re = Regex::new(r"\[Component ([A-Z])\]").unwrap();
    
    // Find markdown file references
    for cap in markdown_link_re.captures_iter(content) {
        let label = cap.get(1).unwrap().as_str();
        let target_path = cap.get(2).unwrap().as_str();
        
        edges.push(Edge::new(
            source_id.to_string(),
            format!("file:{}", target_path),
            EdgeType::References,
            format!("references {}", label),
        ));
    }
    
    // Find rule references
    for cap in rule_ref_re.captures_iter(content) {
        let rule_id = cap.get(1).unwrap().as_str();
        
        edges.push(Edge::new(
            source_id.to_string(),
            format!("rule:{}", rule_id),
            EdgeType::ImplementsRule,
            format!("implements {}", rule_id),
        ));
    }
    
    // Find component references
    for cap in component_ref_re.captures_iter(content) {
        let component_id = cap.get(1).unwrap().as_str();
        
        edges.push(Edge::new(
            source_id.to_string(),
            format!("component:{}", component_id),
            EdgeType::DependsOn,
            format!("depends on Component {}", component_id),
        ));
    }
    
    edges
}

fn extract_frontmatter(content: &str) -> Option<(&str, &str)> {
    if !content.starts_with("---\n") {
        return None;
    }
    
    let after_start = &content[4..]; // Skip initial "---\n"
    if let Some(end_pos) = after_start.find("\n---\n") {
        let frontmatter = &after_start[..end_pos];
        let body = &after_start[end_pos + 5..]; // Skip "\n---\n"
        Some((frontmatter, body))
    } else {
        None
    }
}

fn parse_with_frontmatter(frontmatter_str: &str, body: &str, file_path: &str) -> Result<Node> {
    let frontmatter: FrontMatter = serde_yaml::from_str(frontmatter_str)
        .map_err(|e| SynapseError::Parse(format!("Invalid YAML frontmatter: {}", e)))?;
    
    let node_type = match frontmatter.doc_type.as_deref() {
        Some("rule") => NodeType::Rule,
        Some("decision") => NodeType::Decision,
        Some("architecture") => NodeType::Architecture,
        Some("component") => NodeType::Component,
        Some("function") => NodeType::Function,
        _ => NodeType::File,
    };
    
    let label = frontmatter.title
        .unwrap_or_else(|| extract_first_heading(body).unwrap_or_else(|| file_path.to_string()));
    
    let tags = frontmatter.tags.unwrap_or_default();
    
    // Convert metadata values to strings
    let metadata = frontmatter.metadata
        .into_iter()
        .filter_map(|(k, v)| {
            // Skip the fields we've already handled
            if k == "type" || k == "title" || k == "tags" {
                return None;
            }
            
            let value_str = match v {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => format!("{:?}", v),
            };
            Some((k, value_str))
        })
        .collect();
    
    let node = Node::new(node_type, label, body.to_string())
        .with_tags(tags)
        .with_metadata(metadata);
    
    node.validate()?;
    Ok(node)
}

fn parse_without_frontmatter(content: &str, file_path: &str) -> Result<Node> {
    let label = extract_first_heading(content).unwrap_or_else(|| file_path.to_string());
    
    let node = Node::new(NodeType::File, label, content.to_string());
    node.validate()?;
    Ok(node)
}

fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}