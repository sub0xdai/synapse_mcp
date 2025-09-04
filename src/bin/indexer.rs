use clap::{Arg, Command};
use synapse_mcp::{indexer, graph};
use std::path::PathBuf;
use std::process;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    let matches = Command::new("synapse-indexer")
        .version("0.1.0")
        .about("Synapse MCP indexer - parses markdown files and updates knowledge graph")
        .arg(
            Arg::new("files")
                .help("Markdown files to process")
                .required(true)
                .num_args(1..)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("neo4j-uri")
                .long("neo4j-uri")
                .help("Neo4j database URI")
                .default_value("bolt://localhost:7687")
        )
        .arg(
            Arg::new("neo4j-user")
                .long("neo4j-user")
                .help("Neo4j username")
                .default_value("neo4j")
        )
        .arg(
            Arg::new("neo4j-password")
                .long("neo4j-password")
                .help("Neo4j password")
                .default_value("password")
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Parse files but don't update database")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let files: Vec<PathBuf> = matches.get_many::<PathBuf>("files")
        .expect("files argument is required")
        .cloned()
        .collect();
        
    let neo4j_uri = matches.get_one::<String>("neo4j-uri").unwrap();
    let neo4j_user = matches.get_one::<String>("neo4j-user").unwrap();
    let neo4j_password = matches.get_one::<String>("neo4j-password").unwrap();
    let dry_run = matches.get_flag("dry-run");
    let verbose = matches.get_flag("verbose");

    if verbose {
        println!("Synapse MCP Indexer v0.1.0");
        println!("Processing {} files", files.len());
        if dry_run {
            println!("Running in dry-run mode");
        }
    }

    // Parse all markdown files
    let start_time = std::time::Instant::now();
    
    match indexer::parse_multiple_files(&files) {
        Ok((nodes, edges)) => {
            let parse_duration = start_time.elapsed();
            
            if verbose {
                println!("Parsed {} nodes and {} edges in {}ms", 
                    nodes.len(), edges.len(), parse_duration.as_millis());
            }
            
            // Check performance requirement (under 500ms)
            if parse_duration.as_millis() > 500 {
                eprintln!("Warning: Parsing took {}ms, exceeds 500ms target", parse_duration.as_millis());
            }
            
            if !dry_run {
                // Connect to Neo4j and update graph
                match graph::connect(neo4j_uri, neo4j_user, neo4j_password).await {
                    Ok(graph_conn) => {
                        match graph::batch_create(&graph_conn, &nodes, &edges).await {
                            Ok(_) => {
                                if verbose {
                                    println!("Successfully updated knowledge graph");
                                }
                                println!("Indexed {} files: {} nodes, {} edges", 
                                    files.len(), nodes.len(), edges.len());
                            }
                            Err(e) => {
                                eprintln!("Error updating graph: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error connecting to Neo4j: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                // Dry run - just show what would be done
                println!("Dry run results:");
                println!("  Nodes to create: {}", nodes.len());
                for node in &nodes {
                    println!("    - {:?} ({}): {}", node.node_type, node.label, 
                        truncate_content(&node.content, 50));
                }
                println!("  Edges to create: {}", edges.len());
                for edge in &edges {
                    println!("    - {} -> {} ({})", edge.source_id, edge.target_id, edge.label);
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing files: {}", e);
            process::exit(1);
        }
    }
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_truncate_content() {
        assert_eq!(truncate_content("short", 10), "short");
        assert_eq!(truncate_content("this is a very long content", 10), "this is a ...");
    }

    #[tokio::test]
    async fn test_indexer_with_sample_file() {
        let test_content = r#"---
mcp: synapse
type: rule
title: "Test Rule"
tags: ["test"]
---

# Test Rule

This is a test rule for the indexer.
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", test_content).unwrap();
        
        let files = vec![temp_file.path().to_path_buf()];
        let result = synapse_mcp::indexer::parse_multiple_files(&files);
        
        assert!(result.is_ok());
        let (nodes, _edges) = result.unwrap();
        assert!(!nodes.is_empty());
        assert_eq!(nodes[0].label, "Test Rule");
    }
}