use crate::{RuleGraph, RuleType, Result, SynapseError, CompiledRule, check_rules, CheckRequest, CheckResponse, ContextRequest, ContextResponse, RulesForPathRequest, RulesForPathResponse, PreWriteRequest, PreWriteResponse, PreWriteResultData, RuleViolationDto, RuleContextInfo, CheckResultData, ContextResultData, RulesForPathResultData, AutoFix, get_formatter, Violation};

#[cfg(feature = "ast-fixes")]
use crate::safely_replace_unwrap;

use std::path::PathBuf;

/// Generate AST-based auto-fixes when feature is enabled
#[cfg(feature = "ast-fixes")]
fn generate_ast_based_fixes(content: &str, violations: &[Violation]) -> Vec<AutoFix> {
    let mut fixes = Vec::new();
    
    for violation in violations {
        match violation.rule.pattern.as_str() {
            "unwrap()" => {
                // Use AST analysis for safe unwrap replacement
                match safely_replace_unwrap(content) {
                    Ok(fixed_content) if fixed_content != content => {
                        fixes.push(AutoFix {
                            original_pattern: ".unwrap()".to_string(),
                            suggested_replacement: "?".to_string(),
                            description: "Safe AST-based replacement of unwrap() with ? operator".to_string(),
                            confidence: 0.9, // High confidence from AST analysis
                        });
                    }
                    Ok(_) => {
                        // No safe replacement found, don't suggest fix
                    }
                    Err(_) => {
                        // AST analysis failed, don't suggest fix
                    }
                }
            }
            // panic! is never auto-fixed - requires human judgment
            "panic!" => {
                // Intentionally skip - no auto-fix for panic!
            }
            _ => {
                // Not handled by AST analysis
            }
        }
    }
    
    fixes
}

/// Stub for when AST fixes are not available
#[cfg(not(feature = "ast-fixes"))]
fn generate_ast_based_fixes(_content: &str, _violations: &[Violation]) -> Vec<AutoFix> {
    Vec::new() // No AST fixes available
}

/// Generate auto-fix suggestions for violations (legacy function for simple fixes)
fn generate_simple_auto_fixes(violations: &[Violation]) -> Vec<AutoFix> {
    let mut fixes = Vec::new();
    
    for violation in violations {
        let pattern = &violation.rule.pattern;
        let confidence = 0.8; // Default confidence
        
        // Pattern-specific auto-fixes (KISS principle)
        let auto_fix = match pattern.as_str() {
            "TODO" => AutoFix {
                original_pattern: "TODO".to_string(),
                suggested_replacement: "// Issue #XXX:".to_string(),
                description: "Convert TODO to GitHub issue reference".to_string(),
                confidence,
            },
            "console.log" => AutoFix {
                original_pattern: "console.log".to_string(),
                suggested_replacement: "log::info!".to_string(),
                description: "Replace console.log with proper logging".to_string(),
                confidence,
            },
            // DANGEROUS AUTO-FIXES REMOVED FOR SAFETY
            // unwrap() and panic! require AST analysis to fix safely
            // These will be handled by the AST-based system when enabled
            "unwrap()" => continue, // Skip - requires context analysis
            "panic!" => continue,   // Skip - requires human judgment
            _ => continue, // Skip patterns we don't have fixes for
        };
        
        fixes.push(auto_fix);
    }
    
    fixes
}

/// Generate comprehensive auto-fix suggestions combining simple and AST-based fixes
fn generate_auto_fixes(content: &str, violations: &[Violation]) -> Vec<AutoFix> {
    let mut all_fixes = Vec::new();
    
    // Get simple fixes (TODO, console.log, etc.)
    let mut simple_fixes = generate_simple_auto_fixes(violations);
    all_fixes.append(&mut simple_fixes);
    
    // Get AST-based fixes if available (unwrap, etc.)
    let mut ast_fixes = generate_ast_based_fixes(content, violations);
    all_fixes.append(&mut ast_fixes);
    
    all_fixes
}

/// Apply auto-fixes to content where confidence is high enough
fn apply_auto_fixes(content: &str, fixes: &[AutoFix]) -> Option<String> {
    let mut fixed_content = content.to_string();
    let mut applied_any = false;
    
    for fix in fixes {
        // Only apply fixes with high confidence (>= 0.8)
        if fix.confidence >= 0.8 {
            if fixed_content.contains(&fix.original_pattern) {
                fixed_content = fixed_content.replace(&fix.original_pattern, &fix.suggested_replacement);
                applied_any = true;
            }
        }
    }
    
    if applied_any {
        Some(fixed_content)
    } else {
        None
    }
}

/// PatternEnforcer integrates RuleGraph with MCP server for real-time rule enforcement
#[derive(Debug)]
pub struct PatternEnforcer {
    rule_graph: RuleGraph,
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
    pub fn check_files(&self, request: CheckRequest) -> Result<CheckResponse> {
        let mut all_violations = Vec::new();
        let mut total_rules_applied = 0;
        let dry_run = request.data.dry_run.unwrap_or(false);
        
        for file_path in &request.data.files {
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
        let data = CheckResultData {
            violations: all_violations,
            files_checked: request.data.files.len(),
            rules_applied: total_rules_applied,
        };
        
        Ok(if success {
            CheckResponse::success(data)
        } else {
            // Create a response that indicates failure but still has data
            let mut response = CheckResponse::success(data);
            response.success = false;
            response
        })
    }
    
    /// Generate rule context for AI assistant (implements Read Hook functionality)
    pub fn generate_context(&self, request: ContextRequest) -> Result<ContextResponse> {
        let composite_rules = self.rule_graph.rules_for(&request.data.path)?;
        let format = request.data.format.as_deref().unwrap_or("markdown");
        
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
        
        let formatter = get_formatter(format);
        let context = formatter.format_context(
            &request.data.path,
            &applicable_rules,
            &composite_rules.inheritance_chain,
            &composite_rules.overridden_rules,
        );
        
        Ok(ContextResponse::success(ContextResultData {
            context: Some(context),
            applicable_rules,
            inheritance_chain: composite_rules.inheritance_chain,
            overridden_rules: composite_rules.overridden_rules,
        }))
    }
    
    /// Get rules for a specific path
    pub fn get_rules_for_path(&self, request: RulesForPathRequest) -> Result<RulesForPathResponse> {
        let composite_rules = self.rule_graph.rules_for(&request.data.path)?;
        
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
        
        Ok(RulesForPathResponse::success(RulesForPathResultData {
            path: request.data.path,
            rules,
            inheritance_chain: composite_rules.inheritance_chain,
            overridden_rules: composite_rules.overridden_rules,
        }))
    }
    
    /// Validate content before writing (implements Pre-Write Hook functionality)
    pub fn validate_pre_write(&self, request: PreWriteRequest) -> Result<PreWriteResponse> {
        let file_path = &request.data.file_path;
        let content = &request.data.content;
        
        // Get applicable rules for this file path
        let composite_rules = self.rule_graph.rules_for(file_path)?;
        
        // Convert rules to CompiledRule format for enforcement
        let compiled_rules: Vec<CompiledRule> = composite_rules.applicable_rules
            .iter()
            .map(|rule| CompiledRule::from_rule(rule.clone()))
            .collect();
        
        // Check content against rules
        let violations = check_rules(file_path, content, &compiled_rules)?;
        
        // Generate auto-fix suggestions for violations
        let auto_fixes = if !violations.is_empty() {
            Some(generate_auto_fixes(content, &violations))
        } else {
            None
        };
        
        // Apply auto-fixes if possible
        let fixed_content = if let Some(ref fixes) = auto_fixes {
            apply_auto_fixes(content, fixes)
        } else {
            None
        };
        
        let is_valid = violations.is_empty();
        let violation_dtos = violations.iter().map(RuleViolationDto::from).collect();
        
        Ok(PreWriteResponse::success(PreWriteResultData {
            valid: is_valid,
            violations: violation_dtos,
            auto_fixes,
            fixed_content,
        }))
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rule, RuleSet, CheckData, ContextData, RulesForPathData};
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
        
        let request = CheckRequest::new(CheckData {
            files: vec![test_file.clone()],
            dry_run: Some(false),
        });
        
        let response = enforcer.check_files(request).unwrap();
        
        assert!(!response.success);
        let data = response.data.as_ref().unwrap();
        assert_eq!(data.violations.len(), 2); // forbidden println + missing docs
        assert_eq!(data.files_checked, 1);
        assert!(data.rules_applied > 0);
        
        // Check forbidden violation
        let println_violation = data.violations.iter()
            .find(|v| v.rule_name == "no-println")
            .expect("Should find println violation");
        assert_eq!(println_violation.rule_type, RuleType::Forbidden);
        assert!(println_violation.line_number.is_some());
        assert!(println_violation.line_content.is_some());
        
        // Check required violation
        let docs_violation = data.violations.iter()
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
        
        let request = CheckRequest::new(CheckData {
            files: vec![test_file.clone()],
            dry_run: Some(false),
        });
        
        let response = enforcer.check_files(request).unwrap();
        
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert_eq!(data.violations.len(), 0);
        assert_eq!(data.files_checked, 1);
        assert!(data.rules_applied > 0);
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
        
        let request = CheckRequest::new(CheckData {
            files: vec![test_file.clone()],
            dry_run: Some(true),
        });
        
        let response = enforcer.check_files(request).unwrap();
        
        // Dry run should always return success
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert!(data.violations.len() > 0); // But still report violations
        assert_eq!(data.files_checked, 1);
    }
    
    #[test]
    fn test_generate_context_markdown() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = ContextRequest::new(ContextData {
            path: PathBuf::from("/test/src/main.rs"),
            format: Some("markdown".to_string()),
        });
        
        let response = enforcer.generate_context(request).unwrap();
        
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert!(data.context.is_some());
        assert_eq!(data.applicable_rules.len(), 3);
        
        let context = data.context.as_ref().unwrap();
        assert!(context.contains("# Synapse Rule Enforcement Context"));
        assert!(context.contains("no-println"));
        assert!(context.contains("🚫 Blocking Rules"));
        assert!(context.contains("💡 Standards & Suggestions"));
    }
    
    #[test]
    fn test_generate_context_json() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = ContextRequest::new(ContextData {
            path: PathBuf::from("/test/src/main.rs"),
            format: Some("json".to_string()),
        });
        
        let response = enforcer.generate_context(request).unwrap();
        
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert!(data.context.is_some());
        
        let context = data.context.as_ref().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(context).unwrap();
        assert!(parsed.is_object());
        assert_eq!(parsed["rules"].as_array().unwrap().len(), 3);
        assert_eq!(parsed["rule_count"], 3);
    }
    
    #[test]
    fn test_generate_context_plain() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = ContextRequest::new(ContextData {
            path: PathBuf::from("/test/src/main.rs"),
            format: Some("plain".to_string()),
        });
        
        let response = enforcer.generate_context(request).unwrap();
        
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert!(data.context.is_some());
        
        let context = data.context.as_ref().unwrap();
        assert!(context.contains("File: /test/src/main.rs"));
        assert!(context.contains("Rules: 3"));
        assert!(context.contains("no-println (FORBIDDEN)"));
    }
    
    #[test]
    fn test_get_rules_for_path() {
        let graph = create_test_rule_graph();
        let enforcer = PatternEnforcer::new(graph);
        
        let request = RulesForPathRequest::new(RulesForPathData {
            path: PathBuf::from("/test/src/main.rs"),
        });
        
        let response = enforcer.get_rules_for_path(request).unwrap();
        
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert_eq!(data.rules.len(), 3);
        assert_eq!(data.path, PathBuf::from("/test/src/main.rs"));
        
        // Check enforcement levels
        let blocking_rules: Vec<_> = data.rules.iter()
            .filter(|r| r.enforcement_level == "BLOCKING")
            .collect();
        assert_eq!(blocking_rules.len(), 2); // Forbidden + Required
        
        let suggestion_rules: Vec<_> = data.rules.iter()
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
        let request = RulesForPathRequest::new(RulesForPathData {
            path: temp_dir.path().join("src/main.rs"),
        });
        
        let response = enforcer.get_rules_for_path(request).unwrap();
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert_eq!(data.rules.len(), 2);
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
        
        let request = CheckRequest::new(CheckData {
            files: vec![PathBuf::from("/nonexistent/file.rs")],
            dry_run: Some(false),
        });
        
        let response = enforcer.check_files(request).unwrap();
        
        // Should succeed but skip nonexistent files
        assert!(response.success);
        let data = response.data.as_ref().unwrap();
        assert_eq!(data.violations.len(), 0);
        assert_eq!(data.files_checked, 1);
        assert_eq!(data.rules_applied, 0);
    }
}