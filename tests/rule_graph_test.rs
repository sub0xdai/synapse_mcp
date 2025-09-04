use synapse_mcp::{RuleGraph, Rule, RuleSet, RuleType, CompositeRules};
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_rule_graph_creation() {
    let temp_dir = TempDir::new().unwrap();
    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    
    // Empty directory should create empty graph
    assert_eq!(rule_graph.node_count(), 0);
}

#[test]
fn test_single_rule_file() {
    let temp_dir = TempDir::new().unwrap();
    let rule_file = temp_dir.path().join(".synapse.md");
    
    // Create a simple rule file
    fs::write(&rule_file, r#"---
mcp: synapse
type: rule
---

# Project Rules

FORBIDDEN: `println!` - Use logging instead of direct println! calls.
"#).unwrap();

    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    assert_eq!(rule_graph.node_count(), 1);
    
    // Test rules for a file in the same directory
    let target_file = temp_dir.path().join("main.rs");
    let composite_rules = rule_graph.rules_for(&target_file).unwrap();
    
    assert_eq!(composite_rules.applicable_rules.len(), 1);
    assert_eq!(composite_rules.applicable_rules[0].name, "forbidden-0");
    assert_eq!(composite_rules.applicable_rules[0].rule_type, RuleType::Forbidden);
    assert_eq!(composite_rules.applicable_rules[0].pattern, "println!");
}

#[test]
fn test_nested_directory_inheritance() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create root rule file
    let root_rule = temp_dir.path().join(".synapse.md");
    fs::write(&root_rule, r#"---
mcp: synapse
type: rule
---

# Root Rules

FORBIDDEN: `database` - All database access must go through the service layer.
REQUIRED: `Result<` - All functions must handle errors properly.
"#).unwrap();

    // Create nested directory and rule file
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let src_rule = src_dir.join(".synapse.md");
    fs::write(&src_rule, r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
---

# Source Code Rules

FORBIDDEN: `unwrap()` - Use proper error handling instead of unwrap().
"#).unwrap();

    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    assert_eq!(rule_graph.node_count(), 2);
    
    // Test rules for file in nested directory
    let target_file = src_dir.join("database.rs");
    let composite_rules = rule_graph.rules_for(&target_file).unwrap();
    
    // Should have 3 rules total (2 from root + 1 from src)
    assert_eq!(composite_rules.applicable_rules.len(), 3);
    
    // Check inheritance chain
    assert_eq!(composite_rules.inheritance_chain.len(), 2);
    assert!(composite_rules.inheritance_chain.contains(&root_rule));
    assert!(composite_rules.inheritance_chain.contains(&src_rule));
}

#[test]
fn test_rule_overrides() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create root rule file with a rule that will be overridden
    let root_rule = temp_dir.path().join(".synapse.md");
    fs::write(&root_rule, r#"---
mcp: synapse
type: rule
---

# Root Rules

FORBIDDEN: `println!` - Use logging framework instead.
REQUIRED: `documentation` - All public functions must be documented.
"#).unwrap();

    // Create nested rule file that overrides the println! rule
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let src_rule = src_dir.join(".synapse.md");
    fs::write(&src_rule, r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
overrides: ["forbidden-0"]
---

# Source Rules

USE: `println!` - In development mode, println! is acceptable for debugging.
"#).unwrap();

    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    
    // Test rules for file in nested directory
    let target_file = src_dir.join("main.rs");
    let composite_rules = rule_graph.rules_for(&target_file).unwrap();
    
    // Should have 2 rules: new println! rule + documentation rule
    assert_eq!(composite_rules.applicable_rules.len(), 2);
    
    // Verify the println! rule is now "USE" instead of "FORBIDDEN"
    let println_rule = composite_rules.applicable_rules.iter()
        .find(|r| r.pattern.contains("println!"))
        .expect("Should find println! rule");
    assert_eq!(println_rule.rule_type, RuleType::Standard);
    
    // Verify override tracking
    assert_eq!(composite_rules.overridden_rules.len(), 1);
    assert_eq!(composite_rules.overridden_rules[0], "forbidden-0");
}

#[test]
fn test_deep_nested_inheritance() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create root/.synapse.md
    let root_rule = temp_dir.path().join(".synapse.md");
    fs::write(&root_rule, r#"---
mcp: synapse
type: rule
---

REQUIRED: `copyright` - All files must have copyright header.
"#).unwrap();

    // Create src/.synapse.md
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let src_rule = src_dir.join(".synapse.md");
    fs::write(&src_rule, r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
---

FORBIDDEN: `unsafe` - Avoid unsafe blocks unless absolutely necessary.
"#).unwrap();

    // Create src/utils/.synapse.md
    let utils_dir = src_dir.join("utils");
    fs::create_dir(&utils_dir).unwrap();
    let utils_rule = utils_dir.join(".synapse.md");
    fs::write(&utils_rule, r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
---

REQUIRED: `#[test]` - All utility functions must have unit tests.
"#).unwrap();

    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    assert_eq!(rule_graph.node_count(), 3);
    
    // Test rules for deeply nested file
    let target_file = utils_dir.join("helpers.rs");
    let composite_rules = rule_graph.rules_for(&target_file).unwrap();
    
    // Should inherit all 3 rules
    assert_eq!(composite_rules.applicable_rules.len(), 3);
    
    // Should have 3-level inheritance chain
    assert_eq!(composite_rules.inheritance_chain.len(), 3);
}

#[test]
fn test_rules_for_nonexistent_path() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create one rule file
    let rule_file = temp_dir.path().join(".synapse.md");
    fs::write(&rule_file, r#"---
mcp: synapse
type: rule
---

## REQUIRED tests
Write tests.
"#).unwrap();

    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    
    // Ask for rules for a path that doesn't exist
    let nonexistent_path = PathBuf::from("/completely/nonexistent/path/file.rs");
    let composite_rules = rule_graph.rules_for(&nonexistent_path).unwrap();
    
    // Should return empty rules since no applicable directory structure
    assert_eq!(composite_rules.applicable_rules.len(), 0);
    assert_eq!(composite_rules.inheritance_chain.len(), 0);
}

#[test]
fn test_rules_for_file_with_no_applicable_rules() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create rule file in subdirectory
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let src_rule = src_dir.join(".synapse.md");
    fs::write(&src_rule, r#"---
mcp: synapse
type: rule
---

REQUIRED: `fmt` - Use cargo fmt.
"#).unwrap();

    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    
    // Ask for rules for a file in a different directory
    let docs_file = temp_dir.path().join("docs").join("readme.md");
    let composite_rules = rule_graph.rules_for(&docs_file).unwrap();
    
    // Should have no applicable rules since docs/ doesn't have rules
    // and there's no root .synapse.md
    assert_eq!(composite_rules.applicable_rules.len(), 0);
}

#[test]
fn test_multiple_inheritance_sources() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple rule files that could be inherited
    let common_dir = temp_dir.path().join("common");
    fs::create_dir(&common_dir).unwrap();
    let common_rule = common_dir.join(".synapse.md");
    fs::write(&common_rule, r#"---
mcp: synapse
type: rule
---

REQUIRED: `log::` - Use structured logging.
"#).unwrap();

    let root_rule = temp_dir.path().join(".synapse.md");
    fs::write(&root_rule, r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `hardcoded` - Use configuration.
"#).unwrap();

    // Create child that inherits from both
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let src_rule = src_dir.join(".synapse.md");
    fs::write(&src_rule, r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md", "../common/.synapse.md"]
---

USE: `inject` - Prefer dependency injection patterns.
"#).unwrap();

    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    assert_eq!(rule_graph.node_count(), 3);
    
    let target_file = src_dir.join("service.rs");
    let composite_rules = rule_graph.rules_for(&target_file).unwrap();
    
    // Should have rules from all three sources
    // 1 from root + 1 from common + 1 from src = 3 total
    assert_eq!(composite_rules.applicable_rules.len(), 3);
    
    // Should track inheritance from explicit inherits
    let rule_names: Vec<&str> = composite_rules.applicable_rules
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    
    assert!(rule_names.contains(&"forbidden-0"));
    assert!(rule_names.contains(&"required-0"));
    assert!(rule_names.contains(&"standard-0"));
}

// Performance test to ensure we meet the <500ms target
#[test]
fn test_performance_large_project() {
    use std::time::Instant;
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create a moderately complex project structure
    for i in 0..10 {
        let dir = temp_dir.path().join(format!("module{}", i));
        fs::create_dir(&dir).unwrap();
        
        let rule_file = dir.join(".synapse.md");
        fs::write(&rule_file, format!(r#"---
mcp: synapse
type: rule
inherits: ["../.synapse.md"]
---

REQUIRED: `module{}` - Follow module {} specific standards.
FORBIDDEN: `anti{}` - Avoid these specific anti-patterns for module {}.
"#, i, i, i, i)).unwrap();
        
        // Create nested subdirectories
        for j in 0..3 {
            let subdir = dir.join(format!("sub{}", j));
            fs::create_dir(&subdir).unwrap();
        }
    }
    
    // Create root rule file
    let root_rule = temp_dir.path().join(".synapse.md");
    fs::write(&root_rule, r#"---
mcp: synapse
type: rule
---

REQUIRED: `project` - Follow project-wide standards.
FORBIDDEN: `dangerous` - Avoid these dangerous patterns.
"#).unwrap();

    let start = Instant::now();
    let rule_graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
    let construction_time = start.elapsed();
    
    // Test several rule lookups
    for i in 0..10 {
        let target_file = temp_dir.path()
            .join(format!("module{}", i))
            .join("sub0")
            .join("test.rs");
        let _rules = rule_graph.rules_for(&target_file).unwrap();
    }
    
    let total_time = start.elapsed();
    
    // Should be well under 500ms even for this moderately complex structure
    println!("Graph construction: {:?}, Total time: {:?}", construction_time, total_time);
    assert!(total_time.as_millis() < 500, "Performance target not met: {}ms", total_time.as_millis());
}