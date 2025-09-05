use crate::{Node, Edge, NodeType, EdgeType, Result, SynapseError};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;
use rayon::prelude::*;

#[derive(Debug, serde::Deserialize)]
struct FrontMatter {
    mcp: Option<String>,
    #[serde(rename = "type")]
    doc_type: Option<String>,
    title: Option<String>,
    tags: Option<Vec<String>>,
    #[serde(flatten)]
    metadata: HashMap<String, serde_yaml::Value>,
}

/// Parse a single markdown file into a knowledge graph node
/// 
/// Only processes files with YAML frontmatter containing `mcp: synapse` marker.
/// This allows multiple MCP servers to coexist without conflicts.
/// 
/// # Arguments
/// 
/// * `path` - Path to the markdown file to parse
/// 
/// # Returns
/// 
/// * `Ok(Some(node))` - Successfully parsed file with synapse marker
/// * `Ok(None)` - File was skipped (no frontmatter, wrong MCP marker, etc.)
/// * `Err(error)` - File system error or YAML parsing error
/// 
/// # Performance
/// 
/// * File I/O: O(n) where n = file size
/// * YAML parsing: O(m) where m = frontmatter size  
/// * Content processing: O(n) for relationship extraction
/// 
/// # Supported frontmatter fields
/// 
/// * `mcp: synapse` - Required marker for processing
/// * `type` - Node type (rule, decision, architecture, component, function)
/// * `title` - Display name (falls back to filename)
/// * `tags` - Array of categorization tags
/// * Additional metadata fields are preserved
/// 
/// # Examples
/// 
/// ```no_run
/// use synapse_mcp::parse_markdown_file;
/// 
/// // Returns Some(node) for files with synapse marker
/// let node = parse_markdown_file("docs/rules.md").unwrap();
/// 
/// // Returns None for files without marker  
/// let skipped = parse_markdown_file("README.md").unwrap();
/// assert!(skipped.is_none());
/// ```
pub fn parse_markdown_file<P: AsRef<Path>>(path: P) -> Result<Option<Node>> {
    let content = fs::read_to_string(&path)?;
    let path_str = path.as_ref().to_string_lossy().to_string();
    
    // Only process files with frontmatter
    if let Some((frontmatter_str, body)) = extract_frontmatter(&content) {
        // Parse frontmatter to check MCP marker
        let frontmatter: FrontMatter = serde_yaml::from_str(frontmatter_str)
            .map_err(|e| SynapseError::Parse(format!("Invalid YAML frontmatter: {}", e)))?;
        
        // Only process files marked for Synapse MCP
        if frontmatter.mcp.as_deref() == Some("synapse") {
            Ok(Some(parse_with_frontmatter_validated(frontmatter, body, &path_str)?))
        } else {
            Ok(None) // Skip files not marked for Synapse
        }
    } else {
        Ok(None) // Skip files without frontmatter
    }
}

pub fn parse_multiple_files(paths: &[std::path::PathBuf]) -> Result<(Vec<Node>, Vec<Edge>)> {
    parse_multiple_files_sequential(paths)
}

pub fn parse_multiple_files_parallel(paths: &[std::path::PathBuf]) -> Result<(Vec<Node>, Vec<Edge>)> {
    let verbose = std::env::var("SYNAPSE_VERBOSE").is_ok();
    
    // Parse files in parallel
    let results: Vec<_> = paths
        .par_iter()
        .map(|path| {
            match parse_markdown_file(path) {
                Ok(Some(node)) => Ok(Some(node)),
                Ok(None) => {
                    if verbose {
                        eprintln!("Skipped {} (no MCP marker or not for Synapse)", path.display());
                    }
                    Ok(None)
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    Err(e)
                }
            }
        })
        .collect();
    
    // Collect successful results
    let mut nodes = Vec::new();
    let mut skipped_count = 0;
    
    for result in results {
        match result {
            Ok(Some(node)) => nodes.push(node),
            Ok(None) => skipped_count += 1,
            Err(_) => {} // Already logged error above
        }
    }
    
    if verbose && skipped_count > 0 {
        eprintln!("Processed {} files, skipped {} files without 'mcp: synapse' marker", 
                  nodes.len(), skipped_count);
    }
    
    // Extract relationships between all documents (sequential for now)
    let mut all_edges = Vec::new();
    for node in &nodes {
        let edges = extract_relationships(&node.content, &node.id);
        all_edges.extend(edges);
    }
    
    Ok((nodes, all_edges))
}

pub fn parse_multiple_files_sequential(paths: &[std::path::PathBuf]) -> Result<(Vec<Node>, Vec<Edge>)> {
    let mut nodes = Vec::new();
    let mut all_edges = Vec::new();
    let mut skipped_count = 0;
    
    // Parse all files first, filtering for Synapse MCP documents
    for path in paths {
        match parse_markdown_file(path) {
            Ok(Some(node)) => nodes.push(node),
            Ok(None) => {
                skipped_count += 1;
                if std::env::var("SYNAPSE_VERBOSE").is_ok() {
                    eprintln!("Skipped {} (no MCP marker or not for Synapse)", path.display());
                }
            }
            Err(e) => eprintln!("Warning: Failed to parse {}: {}", path.display(), e),
        }
    }
    
    if std::env::var("SYNAPSE_VERBOSE").is_ok() && skipped_count > 0 {
        eprintln!("Processed {} files, skipped {} files without 'mcp: synapse' marker", 
                  nodes.len(), skipped_count);
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

fn parse_with_frontmatter_validated(frontmatter: FrontMatter, body: &str, file_path: &str) -> Result<Node> {
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
            if k == "mcp" || k == "type" || k == "title" || k == "tags" {
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


fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}