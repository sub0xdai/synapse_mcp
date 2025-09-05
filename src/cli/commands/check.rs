use anyhow::Result;
use clap::ArgMatches;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use synapse_mcp::{RuleGraph, RuleType, CompositeRules};

/// Violation found in a file
#[derive(Debug, Clone)]
pub struct Violation {
    pub file_path: PathBuf,
    pub rule_name: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub message: String,
    pub line_number: Option<usize>,
    pub line_content: Option<String>,
}

/// Result of checking files against rules
#[derive(Debug)]
pub struct CheckResult {
    pub violations: Vec<Violation>,
    pub files_checked: usize,
    pub rules_applied: usize,
}

pub async fn handle_check(matches: &ArgMatches) -> Result<()> {
    let files: Vec<&PathBuf> = matches
        .get_many::<PathBuf>("files")
        .map(|v| v.collect())
        .unwrap_or_default();
        
    let verbose = matches.get_flag("verbose");
    let dry_run = matches.get_flag("dry-run");
    
    if files.is_empty() {
        eprintln!("❌ No files provided to check");
        process::exit(1);
    }
    
    if verbose {
        println!("🔍 Synapse Rule Enforcement");
        println!("Files to check: {}", files.len());
        for file in &files {
            println!("  • {}", file.display());
        }
        println!();
    }
    
    // Load RuleGraph from current directory
    let current_dir = std::env::current_dir()?;
    let rule_graph = match RuleGraph::from_project(&current_dir) {
        Ok(graph) => {
            if verbose {
                let stats = graph.stats();
                println!("📊 Loaded rule graph with {} rule files containing {} total rules", 
                    stats.rule_files, stats.total_rules);
                println!();
            }
            graph
        }
        Err(e) => {
            if verbose {
                println!("⚠️  No rule graph found: {}", e);
                println!("Proceeding without rule enforcement");
            }
            return Ok(());
        }
    };
    
    // Check each file against applicable rules
    let mut all_violations = Vec::new();
    let mut total_rules_applied = 0;
    
    for file_path in &files {
        if !file_path.exists() {
            if verbose {
                println!("⚠️  File does not exist: {}", file_path.display());
            }
            continue;
        }
        
        // Get applicable rules for this file
        let composite_rules = rule_graph.rules_for(file_path)?;
        total_rules_applied += composite_rules.applicable_rules.len();
        
        if verbose {
            println!("🔎 Checking {} ({} rules apply)", 
                file_path.display(), 
                composite_rules.applicable_rules.len()
            );
            
            if !composite_rules.inheritance_chain.is_empty() {
                println!("   Inheritance: {}", 
                    composite_rules.inheritance_chain
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(" → ")
                );
            }
        }
        
        // Read file content
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Failed to read {}: {}", file_path.display(), e);
                continue;
            }
        };
        
        // Check file against rules
        let violations = check_file_against_rules(file_path, &content, &composite_rules)?;
        
        if verbose && !violations.is_empty() {
            println!("   ❌ Found {} violation(s)", violations.len());
        } else if verbose {
            println!("   ✅ No violations found");
        }
        
        all_violations.extend(violations);
    }
    
    let check_result = CheckResult {
        violations: all_violations,
        files_checked: files.len(),
        rules_applied: total_rules_applied,
    };
    
    // Display results
    display_check_results(&check_result, verbose);
    
    // Exit with appropriate code for pre-commit hook
    if dry_run {
        println!("\n🧪 Dry run complete - no enforcement applied");
        Ok(())
    } else if check_result.violations.is_empty() {
        if verbose {
            println!("\n✅ All files pass rule enforcement");
        }
        Ok(())
    } else {
        process::exit(1);
    }
}

pub fn check_file_against_rules(
    file_path: &Path, 
    content: &str, 
    composite_rules: &CompositeRules
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    for rule in &composite_rules.applicable_rules {
        match rule.rule_type {
            RuleType::Forbidden => {
                // Check if forbidden pattern exists
                if let Ok(regex) = Regex::new(&rule.pattern) {
                    for (line_num, line) in lines.iter().enumerate() {
                        if regex.is_match(line) {
                            violations.push(Violation {
                                file_path: file_path.to_path_buf(),
                                rule_name: rule.name.clone(),
                                rule_type: rule.rule_type.clone(),
                                pattern: rule.pattern.clone(),
                                message: rule.message.clone(),
                                line_number: Some(line_num + 1),
                                line_content: Some(line.to_string()),
                            });
                        }
                    }
                } else {
                    // Fall back to simple string matching if regex fails
                    for (line_num, line) in lines.iter().enumerate() {
                        if line.contains(&rule.pattern) {
                            violations.push(Violation {
                                file_path: file_path.to_path_buf(),
                                rule_name: rule.name.clone(),
                                rule_type: rule.rule_type.clone(),
                                pattern: rule.pattern.clone(),
                                message: rule.message.clone(),
                                line_number: Some(line_num + 1),
                                line_content: Some(line.to_string()),
                            });
                        }
                    }
                }
            }
            RuleType::Required => {
                // Check if required pattern is missing
                let pattern_found = if let Ok(regex) = Regex::new(&rule.pattern) {
                    content.lines().any(|line| regex.is_match(line))
                } else {
                    content.contains(&rule.pattern)
                };
                
                if !pattern_found {
                    violations.push(Violation {
                        file_path: file_path.to_path_buf(),
                        rule_name: rule.name.clone(),
                        rule_type: rule.rule_type.clone(),
                        pattern: rule.pattern.clone(),
                        message: rule.message.clone(),
                        line_number: None,
                        line_content: None,
                    });
                }
            }
            // Standard and Convention rules are suggestions, not enforced
            RuleType::Standard | RuleType::Convention => {
                // These could be implemented as warnings in the future
                continue;
            }
        }
    }
    
    Ok(violations)
}

fn display_check_results(result: &CheckResult, verbose: bool) {
    if verbose {
        println!("\n📊 Check Summary:");
        println!("  Files checked: {}", result.files_checked);
        println!("  Rules applied: {}", result.rules_applied);
        println!("  Violations found: {}", result.violations.len());
    }
    
    if result.violations.is_empty() {
        return;
    }
    
    // Group violations by file
    let mut violations_by_file = std::collections::HashMap::new();
    for violation in &result.violations {
        violations_by_file
            .entry(&violation.file_path)
            .or_insert_with(Vec::new)
            .push(violation);
    }
    
    println!("\n❌ Rule Violations Found:");
    for (file_path, violations) in violations_by_file {
        println!("\n📄 {}", file_path.display());
        
        for violation in violations {
            match violation.rule_type {
                RuleType::Forbidden => {
                    println!("  ❌ FORBIDDEN: {} ({})", violation.message, violation.rule_name);
                    if let (Some(line_num), Some(line_content)) = (&violation.line_number, &violation.line_content) {
                        println!("     Line {}: {}", line_num, line_content.trim());
                        println!("     Pattern: {}", violation.pattern);
                    }
                }
                RuleType::Required => {
                    println!("  ⚠️  MISSING REQUIRED: {} ({})", violation.message, violation.rule_name);
                    println!("     Required pattern: {}", violation.pattern);
                }
                _ => {}
            }
        }
    }
    
    println!("\n💡 Fix these violations before committing.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use synapse_mcp::{Rule, RuleSet, RuleSystem};
    use tempfile::TempDir;

    #[test]
    fn test_check_forbidden_pattern() {
        let rule = Rule::new(
            "no-println".to_string(),
            RuleType::Forbidden,
            r"println!\(".to_string(),
            "Use logging instead of println!".to_string(),
        );
        
        let rule_set = RuleSet::new(PathBuf::from("/test/.synapse.md"))
            .add_rule(rule);
            
        let composite_rules = CompositeRules::new()
            .add_rule(rule_set.rules[0].clone());
        
        let content = r#"
            fn main() {
                println!("Hello, world!");
                log::info("This is ok");
            }
        "#;
        
        let violations = check_file_against_rules(
            Path::new("test.rs"), 
            content, 
            &composite_rules
        ).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name, "no-println");
        assert_eq!(violations[0].rule_type, RuleType::Forbidden);
        assert!(violations[0].line_number.is_some());
    }
    
    #[test]
    fn test_check_required_pattern() {
        let rule = Rule::new(
            "must-have-license".to_string(),
            RuleType::Required,
            r"// SPDX-License-Identifier".to_string(),
            "All files must have SPDX license header".to_string(),
        );
        
        let rule_set = RuleSet::new(PathBuf::from("/test/.synapse.md"))
            .add_rule(rule);
            
        let composite_rules = CompositeRules::new()
            .add_rule(rule_set.rules[0].clone());
        
        let content_without_license = r#"
            fn main() {
                println!("Hello");
            }
        "#;
        
        let violations = check_file_against_rules(
            Path::new("test.rs"), 
            content_without_license, 
            &composite_rules
        ).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name, "must-have-license");
        assert_eq!(violations[0].rule_type, RuleType::Required);
        assert!(violations[0].line_number.is_none());
        
        let content_with_license = r#"
            // SPDX-License-Identifier: MIT
            fn main() {
                println!("Hello");
            }
        "#;
        
        let violations = check_file_against_rules(
            Path::new("test.rs"), 
            content_with_license, 
            &composite_rules
        ).unwrap();
        
        assert_eq!(violations.len(), 0);
    }
    
    #[test]
    fn test_no_violations_clean_file() {
        let rule = Rule::new(
            "no-todo".to_string(),
            RuleType::Forbidden,
            "TODO".to_string(),
            "Remove TODO comments before committing".to_string(),
        );
        
        let rule_set = RuleSet::new(PathBuf::from("/test/.synapse.md"))
            .add_rule(rule);
            
        let composite_rules = CompositeRules::new()
            .add_rule(rule_set.rules[0].clone());
        
        let clean_content = r#"
            fn main() {
                println!("Clean code");
                // This is a proper comment
            }
        "#;
        
        let violations = check_file_against_rules(
            Path::new("test.rs"), 
            clean_content, 
            &composite_rules
        ).unwrap();
        
        assert_eq!(violations.len(), 0);
    }
    
    #[test]
    fn test_standard_rules_not_enforced() {
        let rule = Rule::new(
            "prefer-iterators".to_string(),
            RuleType::Standard,
            "for.*in.*".to_string(),
            "Consider using iterator methods".to_string(),
        );
        
        let rule_set = RuleSet::new(PathBuf::from("/test/.synapse.md"))
            .add_rule(rule);
            
        let composite_rules = CompositeRules::new()
            .add_rule(rule_set.rules[0].clone());
        
        let content = r#"
            fn main() {
                for i in 0..10 {
                    println!("{}", i);
                }
            }
        "#;
        
        let violations = check_file_against_rules(
            Path::new("test.rs"), 
            content, 
            &composite_rules
        ).unwrap();
        
        // Standard rules should not create violations
        assert_eq!(violations.len(), 0);
    }
}