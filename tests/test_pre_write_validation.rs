use std::path::PathBuf;
use synapse_mcp::{PreWriteRequest, PreWriteResponse, PreWriteData, AutoFix, RuleGraph, PatternEnforcer};
use tempfile::TempDir;
use std::fs;

/// Test that pre-write validation catches forbidden patterns
#[tokio::test]
async fn test_pre_write_validation_blocks_forbidden() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Create rule that forbids TODO comments
    let rule_content = r#"---
mcp: synapse
type: rule
---

# No TODO Rule
FORBIDDEN: `TODO` - Convert TODOs to GitHub issues
"#;

    // Create .synapse directory and rule file
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    // Create PatternEnforcer
    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    
    let enforcer = PatternEnforcer::new(rule_graph);

    // Test content with forbidden pattern
    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/main.rs"),
        content: "// TODO: Fix this later\nfn main() {}".to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    // Should find violations
    let data = response.data.expect("Response should have data");
    assert!(!data.valid, "Should detect TODO violation");
    assert_eq!(data.violations.len(), 1);
    assert!(data.violations[0].rule_name.contains("TODO") || data.violations[0].pattern.contains("TODO"));
}

/// Test that pre-write validation passes clean content
#[tokio::test]
async fn test_pre_write_validation_passes_clean_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `TODO` - No TODOs allowed
"#;

    // Create .synapse directory and rule file
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    // Clean content without violations
    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/main.rs"),
        content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    assert!(data.valid, "Clean content should pass validation");
    assert_eq!(data.violations.len(), 0);
}

/// Test auto-fix suggestions for common violations
#[tokio::test]
async fn test_pre_write_auto_fix_suggestions() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `TODO` - Convert to GitHub issue
FORBIDDEN: `console.log` - Use proper logging
"#;

    // Create .synapse directory and rule file
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/debug.js"),
        content: "// TODO: Fix this\nconsole.log('debug');".to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    assert!(!data.valid);
    assert_eq!(data.violations.len(), 2);
    
    // Should provide auto-fix suggestions
    assert!(data.auto_fixes.is_some());
    let auto_fixes = data.auto_fixes.unwrap();
    assert!(!auto_fixes.is_empty());
    
    // Check that fixes are provided for both violations
    assert!(auto_fixes.iter().any(|fix| fix.original_pattern.contains("TODO")));
    assert!(auto_fixes.iter().any(|fix| fix.original_pattern.contains("console.log")));
}

/// Test that required patterns are enforced
#[tokio::test]
async fn test_pre_write_validates_required_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

REQUIRED: `#[test]` - All functions need tests
"#;

    // Create .synapse directory and rule file
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    // Content missing required pattern
    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/lib.rs"),
        content: "pub fn calculate(x: i32) -> i32 { x * 2 }".to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    assert!(!data.valid, "Should detect missing required pattern");
    assert_eq!(data.violations.len(), 1);
    assert!(data.violations[0].rule_type == synapse_mcp::RuleType::Required);
}

/// Test inheritance works in pre-write validation
#[tokio::test]
async fn test_pre_write_respects_rule_inheritance() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Root rule
    let root_rule = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `panic!` - No panics allowed
"#;
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    fs::write(synapse_dir.join("root.md"), root_rule).expect("Failed to write root rule");

    // Src directory rule
    let src_dir = project_root.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");
    
    let src_rule = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `unwrap()` - Prefer ? operator
"#;
    let src_synapse_dir = src_dir.join(".synapse");
    fs::create_dir(&src_synapse_dir).expect("Failed to create src .synapse dir");
    fs::write(src_synapse_dir.join("src_rules.md"), src_rule).expect("Failed to write src rule");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    // Content that violates both inherited rules
    let request = PreWriteRequest::new(PreWriteData {
        file_path: src_dir.join("main.rs"),
        content: "fn main() {\n    let x = get_value().unwrap();\n    panic!(\"error\");\n}".to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    assert!(!data.valid);
    assert_eq!(data.violations.len(), 2, "Should inherit both rules");
    
    // Verify both violations are caught
    let violation_patterns: Vec<&str> = data.violations
        .iter()
        .map(|v| v.pattern.as_str())
        .collect();
    
    assert!(violation_patterns.contains(&"panic!"));
    assert!(violation_patterns.contains(&"unwrap()"));
}