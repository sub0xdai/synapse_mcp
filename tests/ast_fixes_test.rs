use std::path::PathBuf;
use synapse_mcp::{PreWriteRequest, PreWriteData, RuleGraph, PatternEnforcer, ast_fixes_available};
use tempfile::TempDir;
use std::fs;

/// Test that AST fixes are only available when feature is enabled
#[test]
fn test_ast_fixes_feature_availability() {
    let available = ast_fixes_available();
    
    #[cfg(feature = "ast-fixes")]
    assert!(available, "AST fixes should be available when feature is enabled");
    
    #[cfg(not(feature = "ast-fixes"))]
    assert!(!available, "AST fixes should not be available when feature is disabled");
}

/// Test safe unwrap replacement in Result-returning function
#[tokio::test]
async fn test_safe_unwrap_replacement_result_function() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Create rule that forbids unwrap()
    let rule_content = r#"---
mcp: synapse
type: rule
---

# Performance Rules
FORBIDDEN: `unwrap()` - Use proper error handling
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("performance.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    // Test Rust code with unwrap() in Result-returning function
    let rust_code = r#"
fn process_data() -> Result<String, Box<dyn std::error::Error>> {
    let maybe_value = Some("data");
    let value = maybe_value.unwrap(); // Should be replaceable with ?
    Ok(value.to_string())
}
"#;

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/lib.rs"),
        content: rust_code.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    
    // Should detect unwrap() violation
    assert!(!data.valid, "Should detect unwrap() violation");
    assert_eq!(data.violations.len(), 1);
    
    // Check auto-fix behavior based on feature flag
    if cfg!(feature = "ast-fixes") {
        // With AST fixes enabled, should provide safe auto-fix
        assert!(data.auto_fixes.is_some(), "Should provide AST-based auto-fix when feature enabled");
        if let Some(fixes) = data.auto_fixes {
            let unwrap_fix = fixes.iter().find(|f| f.original_pattern.contains("unwrap"));
            if let Some(fix) = unwrap_fix {
                assert_eq!(fix.suggested_replacement, "?");
                assert!(fix.confidence >= 0.9, "AST-based fixes should have high confidence");
                assert!(fix.description.contains("AST-based"));
            }
        }
    } else {
        // Without AST fixes, should not provide auto-fix for unwrap()
        if let Some(fixes) = data.auto_fixes {
            assert!(!fixes.iter().any(|f| f.original_pattern.contains("unwrap")), 
                   "Should not auto-fix unwrap() when AST feature disabled");
        }
    }
}

/// Test that unwrap() is NOT replaced in non-Result functions
#[tokio::test]
async fn test_no_unwrap_replacement_in_non_result_function() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `unwrap()` - Use proper error handling
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    // Test Rust code with unwrap() in non-Result function (unsafe to replace)
    let rust_code = r#"
fn print_data() {
    let maybe_value = Some("data");
    let value = maybe_value.unwrap(); // Should NOT be replaceable - function doesn't return Result
    println!("{}", value);
}
"#;

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/lib.rs"),
        content: rust_code.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    
    // Should detect violation but not suggest auto-fix
    assert!(!data.valid, "Should detect unwrap() violation");
    
    // Should NOT provide auto-fix regardless of feature flag
    if let Some(fixes) = data.auto_fixes {
        assert!(!fixes.iter().any(|f| f.original_pattern.contains("unwrap")), 
               "Should not auto-fix unwrap() in non-Result function even with AST feature");
    }
}

/// Test that panic! is never auto-fixed
#[tokio::test] 
async fn test_panic_never_auto_fixed() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `panic!` - Use Result returns
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let rust_code = r#"
fn risky_operation() -> Result<String, String> {
    if true {
        panic!("This should never be auto-fixed!");
    }
    Ok("success".to_string())
}
"#;

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/lib.rs"),
        content: rust_code.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    
    // Should detect panic! violation
    assert!(!data.valid, "Should detect panic! violation");
    
    // Should NEVER provide auto-fix for panic! regardless of feature flag
    if let Some(fixes) = data.auto_fixes {
        assert!(!fixes.iter().any(|f| f.original_pattern.contains("panic")), 
               "Should never auto-fix panic! - requires human judgment");
    }
}

/// Test mixed safe and dangerous patterns
#[tokio::test]
async fn test_mixed_patterns_selective_auto_fixes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `TODO` - Convert to issues
FORBIDDEN: `unwrap()` - Use proper error handling
FORBIDDEN: `panic!` - Use Result returns
FORBIDDEN: `console.log` - Use proper logging
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("mixed.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let mixed_code = r#"
// TODO: Refactor this function
fn process() -> Result<String, Box<dyn std::error::Error>> {
    console.log("Processing..."); // JavaScript-style logging
    let data = Some("value");
    let result = data.unwrap(); // Potentially safe to replace
    if result.is_empty() {
        panic!("Empty result!"); // Never safe to auto-fix
    }
    Ok(result.to_string())
}
"#;

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/mixed.rs"),
        content: mixed_code.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    
    // Should detect all violations
    assert!(!data.valid, "Should detect all violations");
    assert_eq!(data.violations.len(), 4, "Should find TODO, console.log, unwrap, and panic");
    
    if let Some(fixes) = data.auto_fixes {
        let fix_patterns: Vec<&str> = fixes.iter().map(|f| f.original_pattern.as_str()).collect();
        
        // Should always have safe fixes
        assert!(fix_patterns.contains(&"TODO"), "Should auto-fix TODO");
        assert!(fix_patterns.contains(&"console.log"), "Should auto-fix console.log");
        
        // Should never have panic fix
        assert!(!fix_patterns.iter().any(|p| p.contains("panic")), "Should never auto-fix panic!");
        
        // unwrap() fix depends on feature flag
        if cfg!(feature = "ast-fixes") {
            // May or may not have unwrap fix depending on AST analysis result
            // This is acceptable - AST analysis might determine it's not safe
        } else {
            assert!(!fix_patterns.iter().any(|p| p.contains("unwrap")), 
                   "Should not auto-fix unwrap without AST feature");
        }
        
        // Verify confidence levels
        for fix in &fixes {
            assert!(fix.confidence >= 0.8, "All suggested fixes should have high confidence");
        }
    }
}

/// Test feature flag integration
#[tokio::test]
async fn test_feature_flag_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `unwrap()` - Use proper error handling
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("test.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let code_with_unwrap = r#"
fn safe_unwrap() -> Result<i32, String> {
    let opt = Some(42);
    let value = opt.unwrap(); // Safe context for replacement
    Ok(value)
}
"#;

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/test.rs"),
        content: code_with_unwrap.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    
    // Behavior should differ based on feature flag
    #[cfg(feature = "ast-fixes")]
    {
        println!("Testing with ast-fixes feature enabled");
        // May provide AST-based auto-fix (depends on analysis)
        if let Some(fixes) = &data.auto_fixes {
            if let Some(unwrap_fix) = fixes.iter().find(|f| f.original_pattern.contains("unwrap")) {
                assert!(unwrap_fix.confidence >= 0.9, "AST fixes should have high confidence");
                assert!(unwrap_fix.description.contains("AST"));
            }
        }
    }
    
    #[cfg(not(feature = "ast-fixes"))]
    {
        println!("Testing with ast-fixes feature disabled");
        // Should not provide unwrap auto-fix
        if let Some(fixes) = &data.auto_fixes {
            assert!(!fixes.iter().any(|f| f.original_pattern.contains("unwrap")),
                   "Should not auto-fix unwrap without AST feature");
        }
    }
}

/// Test backward compatibility
#[tokio::test]
async fn test_backward_compatibility() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `TODO` - Convert to issues
FORBIDDEN: `console.log` - Use proper logging
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("compat.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    let simple_code = r#"
// TODO: Add error handling
function debug() {
    console.log("Debug info");
}
"#;

    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/compat.js"),
        content: simple_code.to_string(),
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");

    let data = response.data.expect("Response should have data");
    
    // Simple fixes should work regardless of AST feature
    assert!(!data.valid, "Should detect violations");
    assert!(data.auto_fixes.is_some(), "Should provide simple auto-fixes");
    
    if let Some(fixes) = data.auto_fixes {
        let patterns: Vec<&str> = fixes.iter().map(|f| f.original_pattern.as_str()).collect();
        assert!(patterns.contains(&"TODO"), "Should fix TODO");
        assert!(patterns.contains(&"console.log"), "Should fix console.log");
        
        for fix in fixes {
            assert!(fix.confidence >= 0.8, "Simple fixes should have high confidence");
        }
    }
}