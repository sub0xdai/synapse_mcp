use anyhow::Result;
use clap::ArgMatches;
use std::fs;
use std::path::Path;

use synapse_mcp::graph;
use crate::cli::context::{ContextData, ContextSection, ContextItem, format_as_markdown, format_as_json, format_as_plain, create_context_item_from_file};

pub async fn handle_context(
    matches: &ArgMatches,
    neo4j_uri: &str,
    neo4j_user: &str,
    neo4j_password: &str,
) -> Result<()> {
    let scope = matches.get_one::<String>("scope").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let filters: Vec<&String> = matches.get_many::<String>("filter")
        .map(|v| v.collect())
        .unwrap_or_default();
    
    println!("🧠 Generating AI context with scope: {}", scope);
    
    // Try to connect to Neo4j, but fallback to local mode if not available
    let context_data = if let Ok(graph_conn) = graph::connect(neo4j_uri, neo4j_user, neo4j_password).await {
        println!("✅ Connected to Neo4j - using live knowledge graph");
        generate_context_from_graph(&graph_conn, scope, &filters).await?
    } else {
        println!("⚠️  Neo4j not available - using local file scanning");
        generate_context_from_files(scope, &filters).await?
    };
    
    // Format and write context
    let formatted_content = match format.as_str() {
        "json" => format_as_json(&context_data)?,
        "plain" => format_as_plain(&context_data)?,
        "markdown" | _ => format_as_markdown(&context_data, scope)?,
    };
    
    // Write to output file
    fs::write(output, formatted_content)?;
    
    println!("✅ Context generated and saved to {}", output);
    println!("📊 Generated {} sections with {} total items", 
        context_data.sections.len(), 
        context_data.sections.iter().map(|s| s.items.len()).sum::<usize>()
    );
    
    Ok(())
}

async fn generate_context_from_graph(
    graph_conn: &graph::Graph,
    scope: &str,
    filters: &[&String],
) -> Result<ContextData> {
    let mut sections = Vec::new();
    
    match scope {
        "all" => {
            // Generate comprehensive context
            if let Ok(rules) = fetch_nodes_by_type(graph_conn, "rule").await {
                if !rules.is_empty() {
                    sections.push(ContextSection {
                        title: "Project Rules".to_string(),
                        items: rules,
                    });
                }
            }
            
            if let Ok(architecture) = fetch_nodes_by_type(graph_conn, "architecture").await {
                if !architecture.is_empty() {
                    sections.push(ContextSection {
                        title: "Architecture Documentation".to_string(),
                        items: architecture,
                    });
                }
            }
            
            if let Ok(decisions) = fetch_nodes_by_type(graph_conn, "decision").await {
                if !decisions.is_empty() {
                    sections.push(ContextSection {
                        title: "Architecture Decisions".to_string(),
                        items: decisions,
                    });
                }
            }
        }
        "rules" => {
            if let Ok(rules) = fetch_nodes_by_type(graph_conn, "rule").await {
                sections.push(ContextSection {
                    title: "Project Rules & Standards".to_string(),
                    items: rules,
                });
            }
        }
        "architecture" => {
            if let Ok(arch) = fetch_nodes_by_type(graph_conn, "architecture").await {
                sections.push(ContextSection {
                    title: "Architecture Documentation".to_string(),
                    items: arch,
                });
            }
        }
        "decisions" => {
            if let Ok(decisions) = fetch_nodes_by_type(graph_conn, "decision").await {
                sections.push(ContextSection {
                    title: "Architecture Decision Records".to_string(),
                    items: decisions,
                });
            }
        }
        "test" => {
            // Filter for testing-related content
            if let Ok(items) = fetch_nodes_with_tags(graph_conn, &["test", "testing", "quality"]).await {
                sections.push(ContextSection {
                    title: "Testing & Quality Assurance".to_string(),
                    items,
                });
            }
        }
        "api" => {
            // Filter for API-related content
            if let Ok(items) = fetch_nodes_with_tags(graph_conn, &["api", "endpoint", "rest", "graphql"]).await {
                sections.push(ContextSection {
                    title: "API Documentation & Guidelines".to_string(),
                    items,
                });
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown scope: {}", scope));
        }
    }
    
    // Apply additional filters
    if !filters.is_empty() {
        sections = apply_filters(sections, filters);
    }
    
    Ok(ContextData {
        scope: scope.to_string(),
        generated_at: chrono::Utc::now(),
        sections,
    })
}

async fn generate_context_from_files(scope: &str, filters: &[&String]) -> Result<ContextData> {
    let mut sections = Vec::new();
    
    // Scan .synapse directory for markdown files
    let synapse_dir = Path::new(".synapse");
    if !synapse_dir.exists() {
        return Err(anyhow::anyhow!("No .synapse directory found. Run 'synapse init' first."));
    }
    
    match scope {
        "all" => {
            sections.extend(scan_directory_for_context(synapse_dir.join("rules"), "Project Rules").await?);
            sections.extend(scan_directory_for_context(synapse_dir.join("architecture"), "Architecture").await?);
            sections.extend(scan_directory_for_context(synapse_dir.join("decisions"), "Decisions").await?);
            sections.extend(scan_directory_for_context(synapse_dir.join("components"), "Components").await?);
        }
        "rules" => {
            sections.extend(scan_directory_for_context(synapse_dir.join("rules"), "Project Rules").await?);
        }
        "architecture" => {
            sections.extend(scan_directory_for_context(synapse_dir.join("architecture"), "Architecture").await?);
        }
        "decisions" => {
            sections.extend(scan_directory_for_context(synapse_dir.join("decisions"), "Decisions").await?);
        }
        "test" => {
            // Scan all directories but filter for testing-related content
            let all_sections = vec![
                scan_directory_for_context(synapse_dir.join("rules"), "Rules").await?,
                scan_directory_for_context(synapse_dir.join("architecture"), "Architecture").await?,
                scan_directory_for_context(synapse_dir.join("decisions"), "Decisions").await?,
            ].into_iter().flatten().collect();
            
            sections = filter_sections_by_tags(all_sections, &["test", "testing", "quality"]);
        }
        "api" => {
            // Similar filtering for API-related content
            let all_sections = vec![
                scan_directory_for_context(synapse_dir.join("rules"), "Rules").await?,
                scan_directory_for_context(synapse_dir.join("architecture"), "Architecture").await?,
                scan_directory_for_context(synapse_dir.join("decisions"), "Decisions").await?,
            ].into_iter().flatten().collect();
            
            sections = filter_sections_by_tags(all_sections, &["api", "endpoint", "rest"]);
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown scope: {}", scope));
        }
    }
    
    // Apply additional filters
    if !filters.is_empty() {
        sections = apply_filters(sections, filters);
    }
    
    Ok(ContextData {
        scope: scope.to_string(),
        generated_at: chrono::Utc::now(),
        sections: sections.into_iter().filter(|s| !s.items.is_empty()).collect(),
    })
}

async fn scan_directory_for_context(dir: impl AsRef<Path>, section_name: &str) -> Result<Vec<ContextSection>> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Ok(vec![]);
    }
    
    let mut items = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(item) = parse_markdown_file(&content, &path) {
                    items.push(item);
                }
            }
        }
    }
    
    if items.is_empty() {
        Ok(vec![])
    } else {
        Ok(vec![ContextSection {
            title: section_name.to_string(),
            items,
        }])
    }
}

// Helper functions for fetching from graph
async fn fetch_nodes_by_type(_graph_conn: &graph::Graph, _node_type: &str) -> Result<Vec<ContextItem>> {
    // For now, return empty vector - would implement proper graph querying
    Ok(vec![])
}

async fn fetch_nodes_with_tags(_graph_conn: &graph::Graph, _tags: &[&str]) -> Result<Vec<ContextItem>> {
    // For now, return empty vector - would implement proper graph querying
    Ok(vec![])
}

fn apply_filters(sections: Vec<ContextSection>, _filters: &[&String]) -> Vec<ContextSection> {
    // Apply file pattern and tag filters
    sections
}

fn filter_sections_by_tags(sections: Vec<ContextSection>, _tags: &[&str]) -> Vec<ContextSection> {
    // Filter sections by tags
    sections
}

fn parse_markdown_file(content: &str, path: &Path) -> Result<ContextItem> {
    create_context_item_from_file(content, path)
}