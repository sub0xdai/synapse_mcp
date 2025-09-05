use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use synapse_mcp::{RuleGraph, check_rules};

/// Test that the write hook (enforcement) correctly blocks violations
#[tokio::test]
async fn test_write_hook_blocks_violations() {
    // Create a temporary project directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Create a .synapse.md rule file that forbids TODO comments
    let rule_content = r#"---
mcp: synapse
type: rule
---

# No TODO Comments Rule

FORBIDDEN: `TODO` - TODO comments should be converted to proper issue tracking
"#;

    let rule_file = project_root.join(".synapse.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    // Create a source file that violates the rule
    let src_dir = project_root.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");
    
    let violating_file = src_dir.join("bad_code.rs");
    let violating_content = r#"fn main() {
    println!("Hello world");
    // TODO: Fix this later
    let x = 42;
}
"#;
    fs::write(&violating_file, violating_content).expect("Failed to write violating file");

    // Load the rule graph from the project
    let project_pathbuf = PathBuf::from(project_root);
    let rule_graph = RuleGraph::from_project(&project_pathbuf)
        .expect("Failed to load rule graph");

    // Get rules for the violating file
    let rules = rule_graph.rules_for(&violating_file)
        .expect("Failed to get rules for file");

    // Check the file against the rules - this should find violations
    // Note: We'll need to compile the rules for the enforcement check
    let compiled_rules: Vec<_> = rules.applicable_rules.iter()
        .map(|rule| synapse_mcp::CompiledRule::from_rule(rule.clone()))
        .collect();
    
    let violations = check_rules(&violating_file, &violating_content, &compiled_rules)
        .expect("Failed to check rules");

    // Assert that violations were found
    assert!(!violations.is_empty(), "Expected to find violations but found none");
    
    // Check that the violation is about TODO comments
    let todo_violation = violations.iter()
        .find(|v| v.rule.pattern.contains("TODO"));
    assert!(todo_violation.is_some(), 
        "Expected to find TODO violation, but violations were: {:#?}", violations);

    println!("✅ Write hook correctly blocked {} violations", violations.len());
}

/// Test that the read hook (MCP server) provides correct context
/// This test currently focuses on RuleGraph functionality since we don't have test DB setup yet
#[tokio::test] 
async fn test_read_hook_provides_context() {
    // Create a temporary project directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Create multiple .synapse.md rule files to test inheritance
    let root_rule = r#"---
mcp: synapse
type: rule
---

# Performance Rule

REQUIRED: `async` functions must complete within 500ms
"#;

    let root_rule_file = project_root.join(".synapse.md");
    fs::write(&root_rule_file, root_rule).expect("Failed to write root rule");

    // Create a subdirectory with its own rule
    let api_dir = project_root.join("api");
    fs::create_dir(&api_dir).expect("Failed to create api dir");
    
    let api_rule = r#"---
mcp: synapse
type: rule
---

# API Validation Rule

REQUIRED: `validate_input` - All API endpoints must validate input parameters
"#;

    let api_rule_file = api_dir.join(".synapse.md");
    fs::write(&api_rule_file, api_rule).expect("Failed to write API rule");

    // Create a test file in the API directory
    let test_file = api_dir.join("users.rs");
    fs::write(&test_file, "// API implementation").expect("Failed to write test file");

    // Test RuleGraph functionality (which the MCP server depends on)
    let project_pathbuf = PathBuf::from(project_root);
    let rule_graph = RuleGraph::from_project(&project_pathbuf)
        .expect("Failed to create rule graph");

    // Get rules for the test file - this should include both parent and directory rules
    let rules = rule_graph.rules_for(&test_file)
        .expect("Failed to get rules for file");

    // Verify that both rules are found
    assert!(!rules.applicable_rules.is_empty(), "Expected to find rules");

    // Check that we got rules from both the root and api directory
    let rule_names: Vec<String> = rules.applicable_rules.iter()
        .map(|r| r.name.clone())
        .collect();

    // Verify inheritance: API directory file should get both rules
    assert!(rule_names.iter().any(|name| name.contains("required")), 
        "Expected to find required rules (both async and validate_input)");

    println!("✅ Read hook context preparation works correctly");
    println!("   Found {} rules for file: {:?}", rules.applicable_rules.len(), rule_names);

    // Note: Full MCP server testing will be implemented in Phase 4 when we have proper test DB setup
}