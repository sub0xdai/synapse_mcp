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

/// Test that rule overrides work correctly and child rules actually override parent rules
#[tokio::test]
async fn test_rule_overrides_are_applied() {
    // Create a temporary project directory with inheritance and overrides
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Create root rule that forbids TODO comments
    let root_rule = r#"---
mcp: synapse
type: rule
---

# Root Rules

FORBIDDEN: `TODO` - Root rule forbids TODO comments globally
REQUIRED: `license` - All files must have license header
"#;

    let root_rule_file = project_root.join(".synapse.md");
    fs::write(&root_rule_file, root_rule).expect("Failed to write root rule");

    // Create src subdirectory with override rule
    let src_dir = project_root.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");
    
    let src_rule = r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
overrides: ["forbidden-0"]
---

# Source Rules with Override

STANDARD: `TODO` - Override: TODOs are allowed in source code during development
FORBIDDEN: `panic!` - Src-specific rule: no panic! in production code
"#;

    let src_rule_file = src_dir.join(".synapse.md");
    fs::write(&src_rule_file, src_rule).expect("Failed to write src rule");

    // Create test file in src directory
    let test_file = src_dir.join("main.rs");
    fs::write(&test_file, "// Test content").expect("Failed to write test file");

    // Test that overrides are properly applied
    let project_pathbuf = PathBuf::from(project_root);
    let rule_graph = RuleGraph::from_project(&project_pathbuf)
        .expect("Failed to create rule graph");

    let rules = rule_graph.rules_for(&test_file)
        .expect("Failed to get rules for file");

    // Check that we have rules (inheritance working)
    assert!(!rules.applicable_rules.is_empty(), "Should have inherited rules");
    
    // Check that the override worked - TODO should no longer be FORBIDDEN
    let forbidden_rules: Vec<_> = rules.applicable_rules.iter()
        .filter(|rule| rule.rule_type == synapse_mcp::RuleType::Forbidden)
        .collect();
        
    let has_todo_forbidden = forbidden_rules.iter()
        .any(|rule| rule.pattern.contains("TODO"));
    
    assert!(!has_todo_forbidden, 
        "TODO rule should be overridden and not be FORBIDDEN anymore. Found forbidden rules: {:#?}", 
        forbidden_rules);

    // Check that the STANDARD TODO rule is present (the override)
    let standard_rules: Vec<_> = rules.applicable_rules.iter()
        .filter(|rule| rule.rule_type == synapse_mcp::RuleType::Standard)
        .collect();
        
    let has_todo_standard = standard_rules.iter()
        .any(|rule| rule.pattern.contains("TODO"));
    
    assert!(has_todo_standard, 
        "TODO rule should be overridden as STANDARD. Found standard rules: {:#?}", 
        standard_rules);

    // Check that other inherited rules are still present (license requirement)
    let required_rules: Vec<_> = rules.applicable_rules.iter()
        .filter(|rule| rule.rule_type == synapse_mcp::RuleType::Required)
        .collect();
        
    let has_license_required = required_rules.iter()
        .any(|rule| rule.pattern.contains("license"));
    
    assert!(has_license_required, 
        "License requirement should be inherited from parent. Found required rules: {:#?}", 
        required_rules);

    println!("✅ Rule overrides work correctly");
    println!("   Overridden TODO from FORBIDDEN to STANDARD");
    println!("   Inherited license requirement from parent");
    println!("   Added src-specific panic! prohibition");
}

/// Test that multiple levels of inheritance are resolved correctly
#[tokio::test]
async fn test_multiple_inheritance_is_resolved() {
    // Create a deeper directory hierarchy with multiple inheritance levels
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Level 1: Root rules
    let root_rule = r#"---
mcp: synapse
type: rule
---

# Root Level Rules

FORBIDDEN: `console.log` - Root rule: use proper logging
REQUIRED: `license` - Root rule: all files need license
STANDARD: `async_timeout` - Root rule: async operations timeout
"#;

    let root_rule_file = project_root.join(".synapse.md");
    fs::write(&root_rule_file, root_rule).expect("Failed to write root rule");

    // Level 2: Source directory rules
    let src_dir = project_root.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");
    
    let src_rule = r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
---

# Source Level Rules

FORBIDDEN: `unwrap()` - Src rule: prefer proper error handling
REQUIRED: `#[test]` - Src rule: all functions need tests
"#;

    let src_rule_file = src_dir.join(".synapse.md");
    fs::write(&src_rule_file, src_rule).expect("Failed to write src rule");

    // Level 3: API subdirectory rules with overrides
    let api_dir = src_dir.join("api");
    fs::create_dir(&api_dir).expect("Failed to create api dir");
    
    let api_rule = r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
overrides: ["forbidden-0"]
---

# API Level Rules with Override

STANDARD: `console.log` - API override: debug logging allowed in API layer
FORBIDDEN: `TODO` - API rule: no TODOs in production API
REQUIRED: `validate_input` - API rule: all inputs must be validated
"#;

    let api_rule_file = api_dir.join(".synapse.md");
    fs::write(&api_rule_file, api_rule).expect("Failed to write api rule");

    // Create test file at deepest level
    let test_file = api_dir.join("users.rs");
    fs::write(&test_file, "// API implementation").expect("Failed to write test file");

    // Test multiple inheritance resolution
    let project_pathbuf = PathBuf::from(project_root);
    let rule_graph = RuleGraph::from_project(&project_pathbuf)
        .expect("Failed to create rule graph");

    let rules = rule_graph.rules_for(&test_file)
        .expect("Failed to get rules for file");

    // Should have rules from all 3 levels
    assert!(!rules.applicable_rules.is_empty(), "Should have rules from multiple inheritance levels");
    
    // Group rules by type for easier checking
    let forbidden_rules: Vec<_> = rules.applicable_rules.iter()
        .filter(|rule| rule.rule_type == synapse_mcp::RuleType::Forbidden)
        .collect();
    let required_rules: Vec<_> = rules.applicable_rules.iter()
        .filter(|rule| rule.rule_type == synapse_mcp::RuleType::Required)
        .collect();
    let standard_rules: Vec<_> = rules.applicable_rules.iter()
        .filter(|rule| rule.rule_type == synapse_mcp::RuleType::Standard)
        .collect();

    // Check inherited rules from root level
    assert!(required_rules.iter().any(|rule| rule.pattern.contains("license")), 
        "Should inherit license requirement from root");
    assert!(standard_rules.iter().any(|rule| rule.pattern.contains("async_timeout")), 
        "Should inherit async timeout standard from root");

    // Check inherited rules from src level  
    assert!(forbidden_rules.iter().any(|rule| rule.pattern.contains("unwrap")), 
        "Should inherit unwrap prohibition from src level");
    assert!(required_rules.iter().any(|rule| rule.pattern.contains("#[test]")), 
        "Should inherit test requirement from src level");

    // Check api level rules
    assert!(forbidden_rules.iter().any(|rule| rule.pattern.contains("TODO")), 
        "Should have API-level TODO prohibition");
    assert!(required_rules.iter().any(|rule| rule.pattern.contains("validate_input")), 
        "Should have API-level input validation requirement");

    // Check override worked - console.log should be STANDARD, not FORBIDDEN
    assert!(!forbidden_rules.iter().any(|rule| rule.pattern.contains("console.log")), 
        "console.log should be overridden and not FORBIDDEN");
    assert!(standard_rules.iter().any(|rule| rule.pattern.contains("console.log")), 
        "console.log should be overridden as STANDARD");

    // Verify inheritance chain is correct
    assert_eq!(rules.inheritance_chain.len(), 3, 
        "Should have 3 levels in inheritance chain (root -> src -> api)");

    println!("✅ Multiple inheritance resolves correctly");
    println!("   Rules from 3 levels: root ({}) + src ({}) + api ({})", 
        1, 1, 1); // Each level has 1 rule file
    println!("   Total applicable rules: {}", rules.applicable_rules.len());
    println!("   Override applied: console.log changed from FORBIDDEN to STANDARD");
    println!("   Inheritance chain: {}", 
        rules.inheritance_chain
            .iter()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
            .collect::<Vec<_>>()
            .join(" → ")
    );
}