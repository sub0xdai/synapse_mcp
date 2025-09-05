use crate::{RuleGraph, RuleType, CompositeRules, Result, SynapseError, Violation, CompiledRule, check_rules};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// PatternEnforcer integrates RuleGraph with MCP server for real-time rule enforcement
pub struct PatternEnforcer {
    rule_graph: RuleGraph,
}

/// Request to check files against rules (Write Hook)
#[derive(Debug, Deserialize, Serialize)]
pub struct EnforceCheckRequest {
    pub files: Vec<PathBuf>,
    pub dry_run: Option<bool>,
}

/// Response from rule checking
#[derive(Debug, Deserialize, Serialize)]
pub struct EnforceCheckResponse {
    pub success: bool,
    pub violations: Vec<RuleViolationDto>,
    pub files_checked: usize,
    pub rules_applied: usize,
    pub error: Option<String>,
}

/// DTO for rule violations (for serialization)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleViolationDto {
    pub file_path: PathBuf,
    pub rule_name: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub message: String,
    pub line_number: Option<usize>,
    pub line_content: Option<String>,
}

impl From<&Violation> for RuleViolationDto {
    fn from(violation: &Violation) -> Self {
        Self {
            file_path: violation.file_path.clone(),
            rule_name: violation.rule.name.clone(),
            rule_type: violation.rule.rule_type.clone(),
            pattern: violation.rule.pattern.clone(),
            message: violation.rule.message.clone(),
            line_number: violation.line_number,
            line_content: violation.line_content.clone(),
        }
    }
}

/// Request for rule context (Read Hook)
#[derive(Debug, Deserialize, Serialize)]
pub struct EnforceContextRequest {
    pub path: PathBuf,
    pub format: Option<String>,
}

/// Response with rule context for AI assistant
#[derive(Debug, Deserialize, Serialize)]
pub struct EnforceContextResponse {
    pub success: bool,
    pub context: Option<String>,
    pub applicable_rules: Vec<RuleContextInfo>,
    pub inheritance_chain: Vec<PathBuf>,
    pub overridden_rules: Vec<String>,
    pub error: Option<String>,
}


/// Rule information formatted for AI consumption
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleContextInfo {
    pub name: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub message: String,
    pub tags: Vec<String>,
    pub enforcement_level: String,
}

/// Request to get rules for a specific path
#[derive(Debug, Deserialize, Serialize)]
pub struct RulesForPathRequest {
    pub path: PathBuf,
}

/// Response with rules applicable to a path
#[derive(Debug, Deserialize, Serialize)]
pub struct RulesForPathResponse {
    pub success: bool,
    pub path: PathBuf,
    pub rules: Vec<RuleContextInfo>,
    pub inheritance_chain: Vec<PathBuf>,
    pub overridden_rules: Vec<String>,
    pub error: Option<String>,
}

impl PatternEnforcer {
    /// Create a new PatternEnforcer from a project directory
    pub fn from_project(project_root: &PathBuf) -> Result<Self> {
        let rule_graph = RuleGraph::from_project(project_root)?;
        Ok(Self { rule_graph })
    }
    
    /// Create a PatternEnforcer with a pre-built RuleGraph
    pub fn new(rule_graph: RuleGraph) -> Self {
        Self { rule_graph }
    }
    
    /// Get the underlying RuleGraph
    pub fn rule_graph(&self) -> &RuleGraph {
        &self.rule_graph
    }
    
    /// Check files against rules (implements Write Hook functionality)
    pub fn check_files(&self, request: EnforceCheckRequest) -> Result<EnforceCheckResponse> {
        let mut all_violations = Vec::new();
        let mut total_rules_applied = 0;
        let dry_run = request.dry_run.unwrap_or(false);
        
        for file_path in &request.files {
            if !file_path.exists() {
                continue;
            }
            
            // Get applicable rules for this file
            let composite_rules = self.rule_graph.rules_for(file_path)?;
            total_rules_applied += composite_rules.applicable_rules.len();
            
            // Read file content
            let content = std::fs::read_to_string(file_path)
                .map_err(|e| SynapseError::Io(e))?;
            
            // Convert rules to CompiledRule format for enforcement
            let compiled_rules: Vec<CompiledRule> = composite_rules.applicable_rules
                .iter()
                .map(|rule| CompiledRule::from_rule(rule.clone()))
                .collect();
            
            // Check file against rules using unified enforcement
            let violations = check_rules(file_path, &content, &compiled_rules)?;
            let violation_dtos: Vec<RuleViolationDto> = violations.iter().map(|v| v.into()).collect();
            all_violations.extend(violation_dtos);
        }
        
        let success = dry_run || all_violations.is_empty();
        
        Ok(EnforceCheckResponse {
            success,
            violations: all_violations,
            files_checked: request.files.len(),
            rules_applied: total_rules_applied,
            error: None,
        })
    }
    
    /// Generate rule context for AI assistant (implements Read Hook functionality)
    pub fn generate_context(&self, request: EnforceContextRequest) -> Result<EnforceContextResponse> {
        let composite_rules = self.rule_graph.rules_for(&request.path)?;
        let format = request.format.as_deref().unwrap_or("markdown");
        
        let applicable_rules: Vec<RuleContextInfo> = composite_rules.applicable_rules
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
            .collect();
        
        let context = match format {
            "json" => serde_json::to_string_pretty(&applicable_rules)
                .map_err(|e| SynapseError::Serde(e))?,
            "plain" => self.format_context_plain(&request.path, &applicable_rules, &composite_rules.inheritance_chain),
            "markdown" | _ => self.format_context_markdown(&request.path, &applicable_rules, &composite_rules.inheritance_chain, &composite_rules.overridden_rules),
        };
        
        Ok(EnforceContextResponse {
            success: true,
            context: Some(context),
            applicable_rules,
            inheritance_chain: composite_rules.inheritance_chain,
            overridden_rules: composite_rules.overridden_rules,
            error: None,
        })
    }
    
    /// Get rules for a specific path
    pub fn get_rules_for_path(&self, request: RulesForPathRequest) -> Result<RulesForPathResponse> {
        let composite_rules = self.rule_graph.rules_for(&request.path)?;
        
        let rules: Vec<RuleContextInfo> = composite_rules.applicable_rules
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
            .collect();
        
        Ok(RulesForPathResponse {
            success: true,
            path: request.path,
            rules,
            inheritance_chain: composite_rules.inheritance_chain,
            overridden_rules: composite_rules.overridden_rules,
            error: None,
        })
    }
    
    // Legacy check_file_against_rules function removed - now using unified enforcement::check_rules
    
    /// Format context as markdown for AI consumption
    fn format_context_markdown(
        &self,
        path: &PathBuf,
        rules: &[RuleContextInfo],
        inheritance_chain: &[PathBuf],
        overridden_rules: &[String],
    ) -> String {
        let mut output = String::new();
        
        output.push_str("# Synapse Rule Enforcement Context\n\n");
        output.push_str(&format!("**File:** `{}`\n\n", path.display()));
        
        if !inheritance_chain.is_empty() {
            output.push_str(&format!("**Rule Inheritance:** {}\n\n", 
                inheritance_chain
                    .iter()
                    .map(|p| format!("`{}`", p.display()))
                    .collect::<Vec<_>>()
                    .join(" → ")
            ));
        }
        
        if rules.is_empty() {
            output.push_str("## No Rules Apply\n\nNo specific rules are configured for this file path.\n");
            return output;
        }
        
        // Group rules by enforcement level
        let blocking_rules: Vec<_> = rules.iter().filter(|r| r.enforcement_level == "BLOCKING").collect();
        let suggestion_rules: Vec<_> = rules.iter().filter(|r| r.enforcement_level == "SUGGESTION").collect();
        let style_rules: Vec<_> = rules.iter().filter(|r| r.enforcement_level == "STYLE").collect();
        
        if !blocking_rules.is_empty() {
            output.push_str("## 🚫 Blocking Rules (Enforced)\n\n");
            for rule in blocking_rules {
                output.push_str(&format!("### {} ({})\n", rule.name, rule.rule_type_display()));
                output.push_str(&format!("**Pattern:** `{}`\n", rule.pattern));
                output.push_str(&format!("**Message:** {}\n\n", rule.message));
            }
        }
        
        if !suggestion_rules.is_empty() {
            output.push_str("## 💡 Standards & Suggestions\n\n");
            for rule in suggestion_rules {
                output.push_str(&format!("### {} ({})\n", rule.name, rule.rule_type_display()));
                output.push_str(&format!("**Pattern:** `{}`\n", rule.pattern));
                output.push_str(&format!("**Message:** {}\n\n", rule.message));
            }
        }
        
        if !style_rules.is_empty() {
            output.push_str("## 🎨 Style Conventions\n\n");
            for rule in style_rules {
                output.push_str(&format!("### {} ({})\n", rule.name, rule.rule_type_display()));
                output.push_str(&format!("**Pattern:** `{}`\n", rule.pattern));
                output.push_str(&format!("**Message:** {}\n\n", rule.message));
            }
        }
        
        if !overridden_rules.is_empty() {
            output.push_str("## ⚠️ Overridden Rules\n\n");
            for rule_id in overridden_rules {
                output.push_str(&format!("- `{}`\n", rule_id));
            }
            output.push_str("\n");
        }
        
        output.push_str("---\n");
        output.push_str("*Generated by Synapse MCP PatternEnforcer*\n");
        
        output
    }
    
    /// Format context as plain text
    fn format_context_plain(
        &self,
        path: &PathBuf,
        rules: &[RuleContextInfo],
        inheritance_chain: &[PathBuf],
    ) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("File: {}\n", path.display()));
        output.push_str(&format!("Rules: {}\n", rules.len()));
        
        if !inheritance_chain.is_empty() {
            output.push_str(&format!("Inheritance: {}\n", 
                inheritance_chain
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ));
        }
        
        output.push_str("\n");
        
        for rule in rules {
            output.push_str(&format!("{} ({}): {} - {}\n",
                rule.name,
                rule.rule_type_display(),
                rule.pattern,
                rule.message
            ));
        }
        
        output
    }
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
    use crate::{Rule, RuleSet};
    use tempfile::TempDir;
    use std::fs;

    fn create_test_rule_graph() -> RuleGraph {
        let mut graph = RuleGraph::new();
        
        let rule1 = Rule::new(
            "no-println".to_string(),
            RuleType::Forbidden,
            "println!(".to_string(),
            "Use logging instead of println!".to_string(),
        );
        
        let rule2 = Rule::new(
            "must-have-docs".to_string(),
            RuleType::Required,
            "///".to_string(),
            "Public functions must have documentation".to_string(),
        );
        
        let rule3 = Rule::new(
            "prefer-iterators".to_string(),
            RuleType::Standard,
            "for.*in".to_string(),
            "Consider using iterator methods".to_string(),
        );
        
        // Create root rule set that will apply to all files under /
        let rule_set = RuleSet::new(PathBuf::from("/.synapse.md"))
            .add_rule(rule1)
            .add_rule(rule2)
            .add_rule(rule3);
            
        graph.add_rule_set(rule_set);
        graph
    }
    
    #[test]
    fn test_pattern_enforcer_creation() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        assert_eq!(enforcer.rule_graph().node_count(), 1);
    }
    
    #[test]
    fn test_check_files_with_violations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        
        fs::write(&test_file, r#"
            fn main() {
                println!("Hello, world!");  // This should be flagged
            }
        "#).unwrap();
        
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = EnforceCheckRequest {
            files: vec![test_file.clone()],
            dry_run: Some(false),
        };
        
        let response = enforcer.check_files(request).unwrap();
        
        assert!(!response.success);
        assert_eq!(response.violations.len(), 2); // forbidden println + missing docs
        assert_eq!(response.files_checked, 1);
        assert!(response.rules_applied > 0);
        
        // Check forbidden violation
        let println_violation = response.violations.iter()
            .find(|v| v.rule_name == "no-println")
            .expect("Should find println violation");
        assert_eq!(println_violation.rule_type, RuleType::Forbidden);
        assert!(println_violation.line_number.is_some());
        assert!(println_violation.line_content.is_some());
        
        // Check required violation
        let docs_violation = response.violations.iter()
            .find(|v| v.rule_name == "must-have-docs")
            .expect("Should find docs violation");
        assert_eq!(docs_violation.rule_type, RuleType::Required);
        assert!(docs_violation.line_number.is_none());
    }
    
    #[test]
    fn test_check_files_clean() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        
        fs::write(&test_file, r#"
            /// Main function with proper documentation
            fn main() {
                log::info!("Hello, world!");  // Using proper logging
            }
        "#).unwrap();
        
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = EnforceCheckRequest {
            files: vec![test_file.clone()],
            dry_run: Some(false),
        };
        
        let response = enforcer.check_files(request).unwrap();
        
        assert!(response.success);
        assert_eq!(response.violations.len(), 0);
        assert_eq!(response.files_checked, 1);
        assert!(response.rules_applied > 0);
    }
    
    #[test]
    fn test_check_files_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        
        fs::write(&test_file, r#"
            fn main() {
                println!("This should be flagged in dry run");
            }
        "#).unwrap();
        
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = EnforceCheckRequest {
            files: vec![test_file.clone()],
            dry_run: Some(true),
        };
        
        let response = enforcer.check_files(request).unwrap();
        
        // Dry run should always return success
        assert!(response.success);
        assert!(response.violations.len() > 0); // But still report violations
        assert_eq!(response.files_checked, 1);
    }
    
    #[test]
    fn test_generate_context_markdown() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = EnforceContextRequest {
            path: PathBuf::from("/test/src/main.rs"),
            format: Some("markdown".to_string()),
        };
        
        let response = enforcer.generate_context(request).unwrap();
        
        assert!(response.success);
        assert!(response.context.is_some());
        assert_eq!(response.applicable_rules.len(), 3);
        
        let context = response.context.unwrap();
        assert!(context.contains("# Synapse Rule Enforcement Context"));
        assert!(context.contains("no-println"));
        assert!(context.contains("🚫 Blocking Rules"));
        assert!(context.contains("💡 Standards & Suggestions"));
    }
    
    #[test]
    fn test_generate_context_json() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = EnforceContextRequest {
            path: PathBuf::from("/test/src/main.rs"),
            format: Some("json".to_string()),
        };
        
        let response = enforcer.generate_context(request).unwrap();
        
        assert!(response.success);
        assert!(response.context.is_some());
        
        let context = response.context.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&context).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
    
    #[test]
    fn test_generate_context_plain() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = EnforceContextRequest {
            path: PathBuf::from("/test/src/main.rs"),
            format: Some("plain".to_string()),
        };
        
        let response = enforcer.generate_context(request).unwrap();
        
        assert!(response.success);
        assert!(response.context.is_some());
        
        let context = response.context.unwrap();
        assert!(context.contains("File: /test/src/main.rs"));
        assert!(context.contains("Rules: 3"));
        assert!(context.contains("no-println (FORBIDDEN)"));
    }
    
    #[test]
    fn test_get_rules_for_path() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = RulesForPathRequest {
            path: PathBuf::from("/test/src/main.rs"),
        };
        
        let response = enforcer.get_rules_for_path(request).unwrap();
        
        assert!(response.success);
        assert_eq!(response.rules.len(), 3);
        assert_eq!(response.path, PathBuf::from("/test/src/main.rs"));
        
        // Check enforcement levels
        let blocking_rules: Vec<_> = response.rules.iter()
            .filter(|r| r.enforcement_level == "BLOCKING")
            .collect();
        assert_eq!(blocking_rules.len(), 2); // Forbidden + Required
        
        let suggestion_rules: Vec<_> = response.rules.iter()
            .filter(|r| r.enforcement_level == "SUGGESTION")
            .collect();
        assert_eq!(suggestion_rules.len(), 1); // Standard
    }
    
    #[test]
    fn test_from_project_with_no_rules() {
        let temp_dir = TempDir::new().unwrap();
        let result = PatternEnforcer::from_project(&temp_dir.path().to_path_buf());
        
        // Should succeed even with no rule files
        assert!(result.is_ok());
        let enforcer = result.unwrap();
        assert_eq!(enforcer.rule_graph().node_count(), 0);
    }
    
    #[test]
    fn test_from_project_with_rule_files() {
        let temp_dir = TempDir::new().unwrap();
        let rule_file = temp_dir.path().join(".synapse.md");
        
        fs::write(&rule_file, r#"---
mcp: synapse
type: rule
---

# Test Rules

FORBIDDEN: `panic!` - Don't use panic in production code.
REQUIRED: `#[test]` - All test functions must have test attribute.
"#).unwrap();

        let result = PatternEnforcer::from_project(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());
        
        let enforcer = result.unwrap();
        assert_eq!(enforcer.rule_graph().node_count(), 1);
        
        // Test that rules are loaded correctly
        let request = RulesForPathRequest {
            path: temp_dir.path().join("src/main.rs"),
        };
        
        let response = enforcer.get_rules_for_path(request).unwrap();
        assert!(response.success);
        assert_eq!(response.rules.len(), 2);
    }
    
    #[test]
    fn test_enforcement_levels() {
        let rule_info = RuleContextInfo {
            name: "test-rule".to_string(),
            rule_type: RuleType::Forbidden,
            pattern: "test".to_string(),
            message: "Test message".to_string(),
            tags: vec![],
            enforcement_level: "BLOCKING".to_string(),
        };
        
        assert_eq!(rule_info.rule_type_display(), "FORBIDDEN");
    }
    
    #[test]
    fn test_check_nonexistent_file() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = EnforceCheckRequest {
            files: vec![PathBuf::from("/nonexistent/file.rs")],
            dry_run: Some(false),
        };
        
        let response = enforcer.check_files(request).unwrap();
        
        // Should succeed but skip nonexistent files
        assert!(response.success);
        assert_eq!(response.violations.len(), 0);
        assert_eq!(response.files_checked, 1);
        assert_eq!(response.rules_applied, 0);
    }
}