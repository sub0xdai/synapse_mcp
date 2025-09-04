use anyhow::Result;
use clap::ArgMatches;
use std::path::PathBuf;
use std::time::Instant;

use synapse_mcp::{graph, indexer, Node, Edge};

pub async fn handle_index(
    matches: &ArgMatches,
    neo4j_uri: &str,
    neo4j_user: &str,
    neo4j_password: &str,
) -> Result<()> {
    let files: Vec<PathBuf> = matches.get_many::<PathBuf>("files")
        .expect("files argument is required")
        .cloned()
        .collect();
        
    let dry_run = matches.get_flag("dry-run");
    let parallel_workers = *matches.get_one::<usize>("parallel").unwrap();
    let verbose = matches.get_flag("verbose");
    
    if verbose {
        println!("📂 Processing {} files with {} parallel workers", files.len(), parallel_workers);
        if dry_run {
            println!("🔍 Running in dry-run mode");
        }
    }
    
    let start_time = Instant::now();
    
    // Parse files in parallel batches
    let (nodes, edges) = if parallel_workers > 1 {
        parse_files_parallel(&files, parallel_workers, verbose).await?
    } else {
        parse_files_sequential(&files, verbose).await?
    };
    
    let parse_duration = start_time.elapsed();
    
    println!("✅ Parsed {} files: {} nodes, {} edges in {}ms", 
        files.len(), 
        nodes.len(), 
        edges.len(), 
        parse_duration.as_millis()
    );
    
    // Performance warning
    if parse_duration.as_millis() > 500 {
        println!("⚠️  Parsing took {}ms, exceeds 500ms target", parse_duration.as_millis());
    }
    
    if !dry_run {
        // Connect to Neo4j and update graph
        println!("🔗 Connecting to Neo4j at {}", neo4j_uri);
        let graph_conn = graph::connect(neo4j_uri, neo4j_user, neo4j_password).await?;
        
        let update_start = Instant::now();
        graph::batch_create(&graph_conn, &nodes, &edges).await?;
        let update_duration = update_start.elapsed();
        
        println!("✅ Updated knowledge graph in {}ms", update_duration.as_millis());
        
        if verbose {
            println!("📊 Total time: {}ms", (parse_duration + update_duration).as_millis());
        }
    } else {
        // Dry run - show what would be done
        println!("\n🔍 Dry run results:");
        println!("  📄 Files processed: {}", files.len());
        println!("  🔵 Nodes to create: {}", nodes.len());
        
        if verbose {
            for node in &nodes {
                let content_preview = truncate_content(&node.content, 60);
                println!("    - {} ({:?}): {}", node.label, node.node_type, content_preview);
            }
        }
        
        println!("  🔗 Edges to create: {}", edges.len());
        
        if verbose {
            for edge in &edges {
                println!("    - {} -> {} ({})", edge.source_id, edge.target_id, edge.label);
            }
        }
    }
    
    Ok(())
}

async fn parse_files_parallel(
    files: &[PathBuf],
    _parallel_workers: usize,
    verbose: bool,
) -> Result<(Vec<Node>, Vec<Edge>)> {
    if verbose {
        println!("🔄 Using Rayon parallel processing...");
    }
    
    // Use the parallel indexer function which uses Rayon internally
    Ok(indexer::parse_multiple_files_parallel(files)?)
}

async fn parse_files_sequential(
    files: &[PathBuf],
    verbose: bool,
) -> Result<(Vec<Node>, Vec<Edge>)> {
    if verbose {
        println!("🔄 Sequential processing...");
    }
    
    Ok(indexer::parse_multiple_files_sequential(files)?)
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len])
    }
}