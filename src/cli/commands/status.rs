use anyhow::Result;
use clap::ArgMatches;
use std::path::Path;

use synapse_mcp::graph;

pub async fn handle_status(
    matches: &ArgMatches,
    neo4j_uri: &str,
    neo4j_user: &str,
    neo4j_password: &str,
) -> Result<()> {
    let verbose = matches.get_flag("verbose");
    
    println!("🔍 Synapse System Status");
    println!("========================\n");
    
    // Check project structure
    check_project_structure(verbose).await?;
    
    // Check Neo4j connection
    check_neo4j_connection(neo4j_uri, neo4j_user, neo4j_password, verbose).await?;
    
    // Check git hooks
    check_git_hooks(verbose).await?;
    
    // Check dependencies
    if verbose {
        check_dependencies().await?;
    }
    
    println!("\n✅ System status check complete");
    
    Ok(())
}

async fn check_project_structure(verbose: bool) -> Result<()> {
    println!("📁 Project Structure");
    
    let synapse_dir = Path::new(".synapse");
    if synapse_dir.exists() {
        println!("  ✅ .synapse directory found");
        
        if verbose {
            let subdirs = ["rules", "architecture", "decisions", "components"];
            for subdir in &subdirs {
                let path = synapse_dir.join(subdir);
                if path.exists() {
                    let count = std::fs::read_dir(&path)
                        .map(|entries| entries.count())
                        .unwrap_or(0);
                    println!("    📂 {}: {} files", subdir, count);
                } else {
                    println!("    ⚠️  {}: missing", subdir);
                }
            }
        }
    } else {
        println!("  ❌ .synapse directory not found");
        println!("    💡 Run 'synapse init' to initialize workspace");
    }
    
    // Check for documentation files
    let mut doc_count = 0;
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("mcp: synapse") {
                        doc_count += 1;
                    }
                }
            }
        }
    }
    
    if synapse_dir.exists() {
        if let Ok(entries) = walkdir::WalkDir::new(synapse_dir).into_iter().collect::<Result<Vec<_>, _>>() {
            for entry in entries {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if content.contains("mcp: synapse") {
                            doc_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    println!("  📄 Synapse documents: {}", doc_count);
    
    Ok(())
}

async fn check_neo4j_connection(neo4j_uri: &str, neo4j_user: &str, neo4j_password: &str, verbose: bool) -> Result<()> {
    println!("\n🗄️  Neo4j Database");
    
    match graph::connect(neo4j_uri, neo4j_user, neo4j_password).await {
        Ok(conn) => {
            println!("  ✅ Connected to {}", neo4j_uri);
            
            if verbose {
                // Try to get some statistics
                match graph::get_node_count(&conn).await {
                    Ok(count) => println!("    📊 Total nodes: {}", count),
                    Err(e) => println!("    ⚠️  Could not get node count: {}", e),
                }
            }
        }
        Err(e) => {
            println!("  ❌ Connection failed: {}", e);
            println!("    💡 Ensure Neo4j is running on {}", neo4j_uri);
            println!("    💡 Check credentials and network connectivity");
        }
    }
    
    Ok(())
}

async fn check_git_hooks(verbose: bool) -> Result<()> {
    println!("\n🔧 Git Hooks");
    
    // Check if pre-commit is installed
    let pre_commit_check = tokio::process::Command::new("pre-commit")
        .arg("--version")
        .output()
        .await;
        
    match pre_commit_check {
        Ok(output) if output.status.success() => {
            println!("  ✅ pre-commit is installed");
            
            if verbose {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("    📋 Version: {}", version.trim());
            }
        }
        _ => {
            println!("  ❌ pre-commit not found");
            println!("    💡 Install with: uv tool install pre-commit");
        }
    }
    
    // Check if hooks are installed
    let hooks_path = Path::new(".git/hooks/pre-commit");
    if hooks_path.exists() {
        println!("  ✅ Git hooks installed");
    } else {
        println!("  ⚠️  Git hooks not installed");
        println!("    💡 Run 'pre-commit install' to install hooks");
    }
    
    // Check pre-commit config
    let config_path = Path::new(".pre-commit-config.yaml");
    if config_path.exists() {
        println!("  ✅ pre-commit configuration found");
    } else {
        println!("  ⚠️  .pre-commit-config.yaml not found");
    }
    
    Ok(())
}

async fn check_dependencies() -> Result<()> {
    println!("\n📦 Dependencies");
    
    // Check Rust toolchain
    let rustc_check = tokio::process::Command::new("rustc")
        .arg("--version")
        .output()
        .await;
        
    match rustc_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  ✅ Rust: {}", version.trim());
        }
        _ => {
            println!("  ❌ Rust compiler not found");
        }
    }
    
    // Check cargo
    let cargo_check = tokio::process::Command::new("cargo")
        .arg("--version")
        .output()
        .await;
        
    match cargo_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  ✅ Cargo: {}", version.trim());
        }
        _ => {
            println!("  ❌ Cargo not found");
        }
    }
    
    // Check uv
    let uv_check = tokio::process::Command::new("uv")
        .arg("--version")
        .output()
        .await;
        
    match uv_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  ✅ uv: {}", version.trim());
        }
        _ => {
            println!("  ⚠️  uv not found (recommended for Python tool management)");
        }
    }
    
    Ok(())
}