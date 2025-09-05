use anyhow::Result;
use clap::ArgMatches;
use serde_json;
use std::path::PathBuf;

use synapse_mcp::{RuleGraph, RuleType};

/// Context information for AI assistant
#[derive(Debug, Clone, serde::Serialize)]
pub struct EnforceContextData {
    pub file_path: PathBuf,
    pub applicable_rules: Vec<RuleContextInfo>,
    pub inheritance_chain: Vec<PathBuf>,
    pub overridden_rules: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Rule information formatted for AI consumption
#[derive(Debug, Clone, serde::Serialize)]
pub struct RuleContextInfo {
    pub name: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub message: String,
    pub tags: Vec<String>,
    pub enforcement_level: String,
}

pub async fn handle_enforce_context(matches: &ArgMatches) -> Result<()> {
    let path: &PathBuf = matches.get_one::<PathBuf>("path")
        .ok_or_else(|| anyhow::anyhow!("Path is required"))?;
        
    let format = matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("markdown");
    let output = matches.get_one::<String>("output");
    let verbose = matches.get_flag("verbose");
    
    if verbose {
        println!("🤖 Generating enforcement context for: {}", path.display());
    }
    
    // Load RuleGraph from current directory
    let current_dir = std::env::current_dir()?;
    let rule_graph = match RuleGraph::from_project(&current_dir) {
        Ok(graph) => {
            if verbose {
                let stats = graph.stats();
                println!("📊 Loaded rule graph with {} rule files containing {} total rules", 
                    stats.rule_files, stats.total_rules);
            }
            graph
        }
        Err(e) => {
            if verbose {
                println!("⚠️  No rule graph found: {}", e);
            }
            println!("# No Enforcement Rules Found\n");
            println!("No .synapse.md rule files found in the project hierarchy.");
            println!("Consider creating rule files to guide development standards.");
            return Ok(());
        }
    };
    
    // Get applicable rules for the specified path
    let composite_rules = rule_graph.rules_for(path)?;
    
    if verbose {
        println!("📋 Found {} applicable rules for {}", 
            composite_rules.applicable_rules.len(), 
            path.display()
        );
        
        if !composite_rules.inheritance_chain.is_empty() {
            println!("🔗 Inheritance chain: {}", 
                composite_rules.inheritance_chain
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" → ")
            );
        }
    }
    
    // Convert to context data structure
    let context_data = EnforceContextData {
        file_path: path.clone(),
        applicable_rules: composite_rules.applicable_rules
            .into_iter()
            .map(|rule| RuleContextInfo {
                name: rule.name,
                rule_type: rule.rule_type.clone(),
                pattern: rule.pattern,
                message: rule.message,
                tags: rule.tags,
                enforcement_level: match rule.rule_type {
                    RuleType::Forbidden => "BLOCKING".to_string(),
                    RuleType::Required => "BLOCKING".to_string(),
                    RuleType::Standard => "SUGGESTION".to_string(),
                    RuleType::Convention => "STYLE".to_string(),
                },
            })
            .collect(),
        inheritance_chain: composite_rules.inheritance_chain,
        overridden_rules: composite_rules.overridden_rules,
        generated_at: chrono::Utc::now(),
    };
    
    // Format output
    let formatted_output = match format {
        "json" => format_as_json(&context_data)?,
        "plain" => format_as_plain(&context_data)?,
        "markdown" | _ => format_as_markdown(&context_data)?,
    };
    
    // Output to file or stdout
    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted_output)?;
        if verbose {
            println!("✅ Context written to: {}", output_path);
        }
    } else {
        print!("{}", formatted_output);
    }
    
    Ok(())
}

fn format_as_markdown(context: &EnforceContextData) -> Result<String> {
    let mut output = String::new();
    
    output.push_str("# Synapse Rule Enforcement Context\n\n");
    output.push_str(&format!("**File:** `{}`  \n", context.file_path.display()));
    output.push_str(&format!("**Generated:** {}  \n", 
        context.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    
    if !context.inheritance_chain.is_empty() {
        output.push_str(&format!("**Rule Inheritance:** {}  \n\n", 
            context.inheritance_chain
                .iter()
                .map(|p| format!("`{}`", p.display()))
                .collect::<Vec<_>>()
                .join(" → ")
        ));
    } else {
        output.push_str("\n");
    }
    
    if context.applicable_rules.is_empty() {
        output.push_str("## No Rules Apply\n\n");
        output.push_str("No specific rules are configured for this file path.\n");
        return Ok(output);
    }
    
    // Group rules by enforcement level
    let blocking_rules: Vec<_> = context.applicable_rules
        .iter()
        .filter(|r| r.enforcement_level == "BLOCKING")
        .collect();
        
    let suggestion_rules: Vec<_> = context.applicable_rules
        .iter()
        .filter(|r| r.enforcement_level == "SUGGESTION")
        .collect();
        
    let style_rules: Vec<_> = context.applicable_rules
        .iter()
        .filter(|r| r.enforcement_level == "STYLE")
        .collect();
    
    if !blocking_rules.is_empty() {
        output.push_str("## 🚫 Blocking Rules (Enforced)\n\n");
        output.push_str("These rules are automatically enforced and will block commits if violated:\n\n");
        for rule in blocking_rules {
            output.push_str(&format!("### {} ({})\n", rule.name, rule.rule_type_display()));
            output.push_str(&format!("**Pattern:** `{}`  \n", rule.pattern));
            output.push_str(&format!("**Message:** {}  \n", rule.message));
            if !rule.tags.is_empty() {
                output.push_str(&format!("**Tags:** {}  \n", rule.tags.join(", ")));
            }
            output.push_str("\n");
        }
    }
    
    if !suggestion_rules.is_empty() {
        output.push_str("## 💡 Standards & Suggestions\n\n");
        output.push_str("These rules provide guidance and suggestions for better code:\n\n");
        for rule in suggestion_rules {
            output.push_str(&format!("### {} ({})\n", rule.name, rule.rule_type_display()));
            output.push_str(&format!("**Pattern:** `{}`  \n", rule.pattern));
            output.push_str(&format!("**Message:** {}  \n", rule.message));
            if !rule.tags.is_empty() {
                output.push_str(&format!("**Tags:** {}  \n", rule.tags.join(", ")));
            }
            output.push_str("\n");
        }
    }
    
    if !style_rules.is_empty() {
        output.push_str("## 🎨 Style Conventions\n\n");
        output.push_str("These rules define coding style and naming conventions:\n\n");
        for rule in style_rules {
            output.push_str(&format!("### {} ({})\n", rule.name, rule.rule_type_display()));
            output.push_str(&format!("**Pattern:** `{}`  \n", rule.pattern));
            output.push_str(&format!("**Message:** {}  \n", rule.message));
            if !rule.tags.is_empty() {
                output.push_str(&format!("**Tags:** {}  \n", rule.tags.join(", ")));
            }
            output.push_str("\n");
        }
    }
    
    if !context.overridden_rules.is_empty() {
        output.push_str("## ⚠️ Overridden Rules\n\n");
        output.push_str("The following rules were overridden for this file path:\n\n");
        for rule_id in &context.overridden_rules {
            output.push_str(&format!("- `{}`\n", rule_id));
        }
        output.push_str("\n");
    }
    
    output.push_str("---\n");
    output.push_str("*This context was generated by Synapse MCP for AI assistant guidance.*\n");
    
    Ok(output)
}

fn format_as_json(context: &EnforceContextData) -> Result<String> {
    serde_json::to_string_pretty(context)
        .map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {}", e))
}

fn format_as_plain(context: &EnforceContextData) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("File: {}\n", context.file_path.display()));
    output.push_str(&format!("Rules: {}\n", context.applicable_rules.len()));
    
    if !context.inheritance_chain.is_empty() {
        output.push_str(&format!("Inheritance: {}\n", 
            context.inheritance_chain
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ")
        ));
    }
    
    output.push_str("\n");
    
    for rule in &context.applicable_rules {
        output.push_str(&format!("{} ({}): {} - {}\n",
            rule.name,
            rule.rule_type_display(),
            rule.pattern,
            rule.message
        ));
    }
    
    Ok(output)
}

impl RuleContextInfo {
    fn rule_type_display(&self) -> &str {
        match self.rule_type {
            RuleType::Forbidden => "FORBIDDEN",
            RuleType::Required => "REQUIRED", 
            RuleType::Standard => "STANDARD",
            RuleType::Convention => "CONVENTION",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synapse_mcp::{Rule, RuleSet, RuleSystem};
    use tempfile::TempDir;

    fn create_test_context() -> EnforceContextData {
        let rules = vec![
            RuleContextInfo {
                name: "no-println".to_string(),
                rule_type: RuleType::Forbidden,
                pattern: r"println!\(".to_string(),
                message: "Use logging instead of println!".to_string(),
                tags: vec!["logging".to_string()],
                enforcement_level: "BLOCKING".to_string(),
            },
            RuleContextInfo {
                name: "must-have-docs".to_string(),
                rule_type: RuleType::Required,
                pattern: r"///".to_string(),
                message: "Public functions must have documentation".to_string(),
                tags: vec!["documentation".to_string()],
                enforcement_level: "BLOCKING".to_string(),
            },
            RuleContextInfo {
                name: "prefer-iterators".to_string(),
                rule_type: RuleType::Standard,
                pattern: r"for.*in.*".to_string(),
                message: "Consider using iterator methods".to_string(),
                tags: vec!["style".to_string()],
                enforcement_level: "SUGGESTION".to_string(),
            },
        ];
        
        EnforceContextData {
            file_path: PathBuf::from("/test/src/main.rs"),
            applicable_rules: rules,
            inheritance_chain: vec![
                PathBuf::from("/test/.synapse.md"),
                PathBuf::from("/test/src/.synapse.md"),
            ],
            overridden_rules: vec!["old-rule".to_string()],
            generated_at: chrono::Utc::now(),
        }
    }
    
    #[test]
    fn test_format_as_markdown() {
        let context = create_test_context();
        let result = format_as_markdown(&context).unwrap();
        
        assert!(result.contains("# Synapse Rule Enforcement Context"));
        assert!(result.contains("no-println"));
        assert!(result.contains("🚫 Blocking Rules"));
        assert!(result.contains("💡 Standards & Suggestions"));
        assert!(result.contains("⚠️ Overridden Rules"));
    }
    
    #[test]
    fn test_format_as_json() {
        let context = create_test_context();
        let result = format_as_json(&context).unwrap();
        
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());
        
        // Should contain expected fields
        assert!(result.contains("file_path"));
        assert!(result.contains("applicable_rules"));
        assert!(result.contains("inheritance_chain"));
    }
    
    #[test]
    fn test_format_as_plain() {
        let context = create_test_context();
        let result = format_as_plain(&context).unwrap();
        
        assert!(result.contains("File: /test/src/main.rs"));
        assert!(result.contains("Rules: 3"));
        assert!(result.contains("no-println (FORBIDDEN)"));
        assert!(result.contains("must-have-docs (REQUIRED)"));
        assert!(result.contains("prefer-iterators (STANDARD)"));
    }
    
    #[test]
    fn test_empty_rules_context() {
        let context = EnforceContextData {
            file_path: PathBuf::from("/test/empty.rs"),
            applicable_rules: vec![],
            inheritance_chain: vec![],
            overridden_rules: vec![],
            generated_at: chrono::Utc::now(),
        };
        
        let result = format_as_markdown(&context).unwrap();
        assert!(result.contains("No Rules Apply"));
        assert!(result.contains("No specific rules are configured"));
    }
    
    #[test]
    fn test_rule_type_display() {
        let rule = RuleContextInfo {
            name: "test".to_string(),
            rule_type: RuleType::Forbidden,
            pattern: "test".to_string(),
            message: "test".to_string(),
            tags: vec![],
            enforcement_level: "BLOCKING".to_string(),
        };
        
        assert_eq!(rule.rule_type_display(), "FORBIDDEN");
    }
}