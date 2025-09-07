use std::path::PathBuf;
use synapse_mcp::{PreWriteRequest, PreWriteData, RuleGraph, PatternEnforcer};
use tempfile::TempDir;
use std::fs;

/// Test that dangerous auto-fixes (unwrap, panic) are disabled
#[tokio::test]
async fn test_dangerous_auto_fixes_disabled() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Create rules that forbid unwrap() and panic!
    let rule_content = r#"---
mcp: synapse
type: rule
---

# Performance and Safety Rules
FORBIDDEN: `unwrap()` - Use proper error handling
FORBIDDEN: `panic!` - Use Result return types
"#;

    // Create .synapse directory and rule file
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("safety_rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    // Create PatternEnforcer
    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    
    let enforcer = PatternEnforcer::new(rule_graph);

    // Test content with dangerous patterns
    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/main.rs"),
        content: r#"
fn main() {
    let result = Some(42);
    let value = result.unwrap(); // Should detect but not auto-fix
    
    if value < 0 {
        panic!("Negative value!"); // Should detect but not auto-fix
    }
}
"#.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    // Should detect violations
    let data = response.data.expect("Response should have data");
    assert!(!data.valid, "Should detect unwrap() and panic! violations");
    assert_eq!(data.violations.len(), 2, "Should find both unwrap() and panic! violations");
    
    // Should NOT provide auto-fixes for dangerous patterns
    if let Some(auto_fixes) = &data.auto_fixes {
        for fix in auto_fixes {
            assert_ne!(fix.original_pattern, ".unwrap()", "Should not auto-fix unwrap()");
            assert_ne!(fix.original_pattern, "panic!", "Should not auto-fix panic!");
        }
    }
}

/// Test that safe auto-fixes still work (TODO, console.log)
#[tokio::test]
async fn test_safe_auto_fixes_still_work() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

# Safe Patterns We Can Fix
FORBIDDEN: `TODO` - Convert to GitHub issue
FORBIDDEN: `console.log` - Use proper logging
"#;

    // Create .synapse directory and rule file
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("safe_rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/debug.js"),
        content: "// TODO: Fix this later\nconsole.log('debug info');".to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    assert!(!data.valid, "Should detect TODO and console.log violations");
    assert_eq!(data.violations.len(), 2);
    
    // Should provide auto-fixes for safe patterns
    assert!(data.auto_fixes.is_some(), "Should provide auto-fixes for safe patterns");
    let auto_fixes = data.auto_fixes.unwrap();
    assert!(!auto_fixes.is_empty(), "Should have at least one auto-fix");
    
    // Verify we have fixes for safe patterns
    let patterns: Vec<&str> = auto_fixes.iter().map(|f| f.original_pattern.as_str()).collect();
    assert!(patterns.contains(&"TODO") || patterns.contains(&"console.log"), 
           "Should provide fixes for safe patterns");
}

/// Test mixed scenario: dangerous + safe patterns
#[tokio::test]
async fn test_mixed_dangerous_and_safe_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

# Mixed Rules
FORBIDDEN: `TODO` - Safe to auto-fix
FORBIDDEN: `unwrap()` - Dangerous to auto-fix
FORBIDDEN: `console.log` - Safe to auto-fix
FORBIDDEN: `panic!` - Dangerous to auto-fix
"#;

    // Create .synapse directory and rule file
    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("mixed_rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/mixed.rs"),
        content: r#"
// TODO: Refactor this function
fn process_data(data: Option<String>) {
    console.log("Processing data"); // JavaScript-style logging
    let value = data.unwrap(); // Dangerous unwrap
    if value.is_empty() {
        panic!("Empty data!"); // Dangerous panic
    }
}
"#.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    assert!(!data.valid, "Should detect all violations");
    assert_eq!(data.violations.len(), 4, "Should find all four violations");
    
    // Should provide auto-fixes only for safe patterns
    if let Some(auto_fixes) = &data.auto_fixes {
        let safe_patterns = ["TODO", "console.log"];
        let dangerous_patterns = [".unwrap()", "panic!"];
        
        for fix in auto_fixes {
            assert!(safe_patterns.contains(&fix.original_pattern.as_str()),
                   "Auto-fix should only be for safe patterns, got: {}", fix.original_pattern);
            assert!(!dangerous_patterns.contains(&fix.original_pattern.as_str()),
                   "Should not auto-fix dangerous pattern: {}", fix.original_pattern);
        }
        
        // Should have exactly 2 fixes (for TODO and console.log)
        assert_eq!(auto_fixes.len(), 2, "Should have exactly 2 safe auto-fixes");
    } else {
        panic!("Should provide auto-fixes for safe patterns");
    }
}

/// Test that auto-fix application respects confidence thresholds
#[tokio::test]
async fn test_auto_fix_confidence_thresholds() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

# High Confidence Fixes Only
FORBIDDEN: `TODO` - High confidence fix
FORBIDDEN: `console.log` - High confidence fix
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("confidence_rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/confidence.js"),
        content: "// TODO: Test confidence\nconsole.log('test');".to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    
    if let Some(auto_fixes) = &data.auto_fixes {
        for fix in auto_fixes {
            // All returned fixes should have confidence >= 0.8 (high confidence)
            assert!(fix.confidence >= 0.8, 
                   "Auto-fix confidence should be >= 0.8, got: {} for pattern: {}", 
                   fix.confidence, fix.original_pattern);
        }
    }
}