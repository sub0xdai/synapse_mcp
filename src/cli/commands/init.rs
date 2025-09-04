use anyhow::Result;
use clap::ArgMatches;
use std::fs;
use std::path::Path;

use crate::cli::templates;

pub async fn handle_init(matches: &ArgMatches) -> Result<()> {
    let project_name = matches.get_one::<String>("project-name")
        .map(|s| s.as_str())
        .unwrap_or("synapse-project");
    
    let template = matches.get_one::<String>("template").unwrap();
    let install_hooks = matches.get_flag("hooks");
    
    println!("🎯 Initializing Synapse workspace for '{}'", project_name);
    println!("📋 Template: {}", template);
    
    // Create .synapse directory structure
    create_synapse_directory()?;
    
    // Deploy templates based on project type
    deploy_templates(project_name, template).await?;
    
    // Update .gitignore
    update_gitignore()?;
    
    // Install hooks if requested
    if install_hooks {
        install_git_hooks().await?;
    }
    
    print_success_message(project_name, template, install_hooks)?;
    
    Ok(())
}

fn create_synapse_directory() -> Result<()> {
    let synapse_dir = Path::new(".synapse");
    
    // Create main directories
    fs::create_dir_all(synapse_dir)?;
    fs::create_dir_all(synapse_dir.join("templates"))?;
    fs::create_dir_all(synapse_dir.join("rules"))?;
    fs::create_dir_all(synapse_dir.join("architecture"))?;
    fs::create_dir_all(synapse_dir.join("decisions"))?;
    fs::create_dir_all(synapse_dir.join("components"))?;
    
    // Create README explaining the structure
    let readme_content = r#"# Synapse AI Memory Workspace

This directory contains AI-readable documentation that helps build context for AI coding assistants.

## Structure

- `rules/` - Development rules and coding standards
- `architecture/` - High-level architecture documentation
- `decisions/` - Architecture decision records (ADRs)
- `components/` - Component specifications and documentation
- `templates/` - Document templates for consistency

## Usage

1. Fill out the template files in each directory
2. Create new documents using the templates
3. Use `synapse context` to generate AI context
4. All documents are automatically indexed on git commit

## Document Format

All documents should include YAML frontmatter:

```yaml
---
mcp: synapse
type: rule|architecture|decision|component
title: "Document Title"
tags: ["tag1", "tag2"]
---
```

Only documents with `mcp: synapse` will be indexed.
"#;
    
    fs::write(synapse_dir.join("README.md"), readme_content)?;
    
    println!("✅ Created .synapse workspace structure");
    Ok(())
}

async fn deploy_templates(project_name: &str, template: &str) -> Result<()> {
    match template {
        "rust" => templates::rust::deploy_templates(project_name).await?,
        "python" => templates::python::deploy_templates(project_name).await?,
        "typescript" => templates::typescript::deploy_templates(project_name).await?,
        "generic" => templates::generic::deploy_templates(project_name).await?,
        _ => return Err(anyhow::anyhow!("Unknown template type: {}", template)),
    }
    
    println!("✅ Deployed {} templates", template);
    Ok(())
}

fn update_gitignore() -> Result<()> {
    let gitignore_path = Path::new(".gitignore");
    let mut content = String::new();
    
    if gitignore_path.exists() {
        content = fs::read_to_string(gitignore_path)?;
    }
    
    let synapse_entries = vec![
        "# Synapse MCP context files",
        ".synapse_context",
        ".synapse_context.*",
        "",
    ];
    
    let synapse_section = synapse_entries.join("\n");
    
    if !content.contains(".synapse_context") {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(&synapse_section);
        
        fs::write(gitignore_path, content)?;
        println!("✅ Updated .gitignore with Synapse entries");
    }
    
    Ok(())
}

async fn install_git_hooks() -> Result<()> {
    println!("🔧 Installing git hooks...");
    
    // Check if pre-commit is available
    let output = tokio::process::Command::new("pre-commit")
        .arg("--version")
        .output()
        .await;
        
    if output.is_err() {
        println!("⚠️  pre-commit not found. Installing with uv...");
        
        let install_output = tokio::process::Command::new("uv")
            .args(&["tool", "install", "pre-commit"])
            .output()
            .await?;
            
        if !install_output.status.success() {
            return Err(anyhow::anyhow!("Failed to install pre-commit"));
        }
    }
    
    // Install the hooks
    let hook_output = tokio::process::Command::new("pre-commit")
        .arg("install")
        .output()
        .await?;
        
    if !hook_output.status.success() {
        return Err(anyhow::anyhow!("Failed to install git hooks"));
    }
    
    println!("✅ Git hooks installed successfully");
    Ok(())
}

fn print_success_message(project_name: &str, template: &str, hooks_installed: bool) -> Result<()> {
    println!("\n🎉 Synapse workspace initialized successfully!");
    println!("\n📋 Project: {}", project_name);
    println!("📋 Template: {}", template);
    
    if hooks_installed {
        println!("🔧 Git hooks: Installed");
    } else {
        println!("🔧 Git hooks: Not installed (use --hooks to install)");
    }
    
    println!("\n🚀 Next steps:");
    println!("   1. Edit the templates in .synapse/ directories");
    println!("   2. Fill in your project-specific documentation");
    println!("   3. Run 'synapse context' to generate AI context");
    
    if !hooks_installed {
        println!("   4. Run 'synapse init --hooks' to install git automation");
    } else {
        println!("   4. Commit changes - documentation will auto-index!");
    }
    
    println!("\n💡 Quick commands:");
    println!("   synapse context              # Generate AI context");
    println!("   synapse context --scope=api  # Context for API development");
    println!("   synapse query \"performance\"   # Query knowledge graph");
    println!("   synapse status               # Check system health");
    
    Ok(())
}