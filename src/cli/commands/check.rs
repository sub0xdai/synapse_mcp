use anyhow::Result;
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use synapse_mcp::{RuleGraph, RuleType, Violation, check_rules};

/// Result of checking files against rules
#[derive(Debug)]
pub struct CheckResult {
    pub violations: Vec<Violation>,
    pub files_checked: usize,
    pub rules_applied: usize,
}

pub async fn handle_check(matches: &ArgMatches, rule_graph_opt: Option<&RuleGraph>) -> Result<()> {
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
    
    // Use the pre-loaded RuleGraph or exit if none available
    let rule_graph = match rule_graph_opt {
        Some(graph) => {
            if verbose {
                let stats = graph.stats();
                println!("📊 Using rule graph with {} rule files containing {} total rules", 
                    stats.rule_files, stats.total_rules);
                println!();
            }
            graph
        }
        None => {
            if verbose {
                println!("⚠️  No rule graph available - proceeding without rule enforcement");
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
        
        // Convert rules to CompiledRule format for enforcement
        let compiled_rules: Vec<synapse_mcp::CompiledRule> = composite_rules.applicable_rules
            .iter()
            .map(|rule| synapse_mcp::CompiledRule::from_rule(rule.clone()))
            .collect();
        
        // Check file against rules using unified enforcement
        let violations = check_rules(file_path, &content, &compiled_rules)?;
        
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

// Legacy function removed - now using unified enforcement::check_rules

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
            match violation.rule.rule_type {
                RuleType::Forbidden => {
                    println!("  ❌ FORBIDDEN: {} ({})", violation.rule.message, violation.rule.name);
                    if let (Some(line_num), Some(line_content)) = (&violation.line_number, &violation.line_content) {
                        println!("     Line {}: {}", line_num, line_content.trim());
                        println!("     Pattern: {}", violation.rule.pattern);
                    }
                }
                RuleType::Required => {
                    println!("  ⚠️  MISSING REQUIRED: {} ({})", violation.rule.message, violation.rule.name);
                    println!("     Required pattern: {}", violation.rule.pattern);
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
    use synapse_mcp::{Rule, CompiledRule};

    #[test]
    fn test_check_forbidden_pattern() {
        let rule = Rule::new(
            "no-println".to_string(),
            RuleType::Forbidden,
            r"println!\(".to_string(),
            "Use logging instead of println!".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let content = r#"
            fn main() {
                println!("Hello, world!");
                log::info("This is ok");
            }
        "#;
        
        let violations = check_rules(file_path, content, &[compiled_rule]).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule.name, "no-println");
        assert_eq!(violations[0].rule.rule_type, RuleType::Forbidden);
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
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let content_without_license = r#"
            fn main() {
                println!("Hello");
            }
        "#;
        
        let violations = check_rules(file_path, content_without_license, &[compiled_rule.clone()]).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule.name, "must-have-license");
        assert_eq!(violations[0].rule.rule_type, RuleType::Required);
        assert!(violations[0].line_number.is_none());
        
        let content_with_license = r#"
            // SPDX-License-Identifier: MIT
            fn main() {
                println!("Hello");
            }
        "#;
        
        let violations = check_rules(file_path, content_with_license, &[compiled_rule]).unwrap();
        
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
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let clean_content = r#"
            fn main() {
                println!("Clean code");
                // This is a proper comment
            }
        "#;
        
        let violations = check_rules(file_path, clean_content, &[compiled_rule]).unwrap();
        
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
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let content = r#"
            fn main() {
                for i in 0..10 {
                    println!("{}", i);
                }
            }
        "#;
        
        let violations = check_rules(file_path, content, &[compiled_rule]).unwrap();
        
        // Standard rules should not create violations
        assert_eq!(violations.len(), 0);
    }
}