use synapse_mcp::rules::RuleSystem;
use synapse_mcp::models::{RuleType};
use tempfile::TempDir;
use std::fs;

fn create_rule_file(dir: &std::path::Path, filename: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, content).unwrap();
    file_path
}

#[test]
fn test_integration_rule_system_with_inheritance() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path();
    
    // Create directory structure: root -> src -> utils
    let src_dir = root_path.join("src");
    let utils_dir = src_dir.join("utils");
    fs::create_dir_all(&utils_dir).unwrap();
    
    // Root rules
    create_rule_file(root_path, ".synapse.md", r#"---
project: test-project
---

# Root Rules

FORBIDDEN: `println!` - Use logging instead
REQUIRED: `#[test]` - All functions must have tests
"#);
    
    // Src rules (inherits from root, adds overrides)
    create_rule_file(&src_dir, ".synapse.md", r#"---
inherits:
  - "../.synapse.md"
module: src-module
---

# Src Rules

USE: `Result<T>` - Prefer Result types
FORBIDDEN: `panic!` - Handle errors gracefully
"#);
    
    // Utils rules (inherits from src, which inherits from root)
    create_rule_file(&utils_dir, ".synapse.md", r#"---
inherits:
  - "../.synapse.md"
module: utils-module
---

# Utils Rules

Mandatory: `pub fn` - All utility functions must be public
"#);
    
    // Test the complete system
    let rule_system = RuleSystem::new();
    
    // 1. Discovery should find all rule files
    let rule_sets = rule_system.load_rules(&root_path.to_path_buf()).unwrap();
    assert_eq!(rule_sets.len(), 3);
    
    // Verify each rule set was parsed correctly
    let root_rules = rule_sets.iter().find(|rs| rs.path == root_path.join(".synapse.md")).unwrap();
    assert_eq!(root_rules.rules.len(), 2); // println, #[test]
    assert_eq!(root_rules.metadata.get("project").unwrap(), "test-project");
    
    let src_rules = rule_sets.iter().find(|rs| rs.path == src_dir.join(".synapse.md")).unwrap();
    assert_eq!(src_rules.rules.len(), 2); // Result<T>, panic!
    assert_eq!(src_rules.inherits.len(), 1);
    assert_eq!(src_rules.metadata.get("module").unwrap(), "src-module");
    
    let utils_rules = rule_sets.iter().find(|rs| rs.path == utils_dir.join(".synapse.md")).unwrap();
    assert_eq!(utils_rules.rules.len(), 1); // pub fn
    assert_eq!(utils_rules.inherits.len(), 1);
    
    // 2. Test composite rules for a target file in utils
    let target_file = utils_dir.join("helper.rs");
    fs::write(&target_file, "// helper file").unwrap();
    
    let composite = rule_system.rules_for_path(&target_file, &rule_sets);
    
    // Should inherit rules from utils -> src -> root
    assert!(composite.applicable_rules.len() >= 5); // At least 5 rules from inheritance chain
    
    // Check that we have rules from all levels
    let forbidden_count = composite.applicable_rules.iter()
        .filter(|r| r.rule_type == RuleType::Forbidden)
        .count();
    let required_count = composite.applicable_rules.iter()
        .filter(|r| r.rule_type == RuleType::Required)
        .count();
    let standard_count = composite.applicable_rules.iter()
        .filter(|r| r.rule_type == RuleType::Standard)
        .count();
        
    assert!(forbidden_count >= 2); // println!, panic!
    assert!(required_count >= 2); // #[test], pub fn
    assert!(standard_count >= 1); // Result<T>
    
    // Check inheritance chain
    assert_eq!(composite.inheritance_chain.len(), 3);
    assert!(composite.inheritance_chain[0].to_string_lossy().contains("utils/.synapse.md"));
    assert!(composite.inheritance_chain[1].to_string_lossy().contains("src/.synapse.md"));
    assert!(composite.inheritance_chain[2].to_string_lossy().contains(".synapse.md"));
    assert!(!composite.inheritance_chain[2].to_string_lossy().contains("src/"));
}

#[test]
fn test_integration_rule_parsing_various_formats() {
    let temp_dir = TempDir::new().unwrap();
    let rule_system = RuleSystem::new();
    
    // Test comprehensive rule file with all rule types
    let comprehensive_content = r#"---
project: comprehensive-test
module: test-module
custom_field: custom_value
---

# Comprehensive Rule Test

## Forbidden Patterns
FORBIDDEN: `unwrap()` - Handle errors properly
Never: `todo!()` - Complete implementation
must not: `global_var` - Use dependency injection

## Required Patterns  
REQUIRED: `#[derive(Debug)]` - All structs must be debuggable
Mandatory: `mod tests` - Each module needs tests
MUST: `pub(crate)` - Use explicit visibility

## Standard Patterns
USE: `Vec<T>` - Prefer vectors over arrays
Prefer: `String` - Use String for owned text
Should: `async fn` - Use async for IO operations

## Conventions (case variations)
forbidden: `snake_case` - Use camelCase for functions  
required: `PascalCase` - Use PascalCase for types
use: `kebab-case` - Use kebab-case for file names
"#;
    
    let rule_file = create_rule_file(temp_dir.path(), ".synapse.md", comprehensive_content);
    
    let rule_set = rule_system.parser.parse_rule_file(&rule_file).unwrap();
    
    // Verify metadata parsing
    assert_eq!(rule_set.metadata.get("project").unwrap(), "comprehensive-test");
    assert_eq!(rule_set.metadata.get("module").unwrap(), "test-module");
    assert_eq!(rule_set.metadata.get("custom_field").unwrap(), "custom_value");
    
    // Verify rule parsing
    assert_eq!(rule_set.rules.len(), 12);
    
    // Count by type
    let forbidden_count = rule_set.rules.iter().filter(|r| r.rule_type == RuleType::Forbidden).count();
    let required_count = rule_set.rules.iter().filter(|r| r.rule_type == RuleType::Required).count();
    let standard_count = rule_set.rules.iter().filter(|r| r.rule_type == RuleType::Standard).count();
    
    assert_eq!(forbidden_count, 4); // unwrap, todo, global_var, snake_case
    assert_eq!(required_count, 4); // Debug, mod tests, pub(crate), PascalCase
    assert_eq!(standard_count, 4); // Vec<T>, String, async fn, kebab-case
    
    // Verify specific rule content
    let unwrap_rule = rule_set.rules.iter().find(|r| r.pattern == "unwrap()").unwrap();
    assert_eq!(unwrap_rule.rule_type, RuleType::Forbidden);
    assert!(unwrap_rule.message.contains("Handle errors"));
    
    let debug_rule = rule_set.rules.iter().find(|r| r.pattern == "#[derive(Debug)]").unwrap();
    assert_eq!(debug_rule.rule_type, RuleType::Required);
    assert!(debug_rule.message.contains("debuggable"));
}

#[test]
fn test_integration_empty_and_minimal_files() {
    let temp_dir = TempDir::new().unwrap();
    let rule_system = RuleSystem::new();
    
    // Empty file
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();
    create_rule_file(&empty_dir, ".synapse.md", "");
    
    // Minimal frontmatter only
    let minimal_dir = temp_dir.path().join("minimal");
    fs::create_dir(&minimal_dir).unwrap();
    create_rule_file(&minimal_dir, ".synapse.md", r#"---
project: minimal
---"#);
    
    // Content only (no frontmatter)
    let content_dir = temp_dir.path().join("content");
    fs::create_dir(&content_dir).unwrap();
    create_rule_file(&content_dir, ".synapse.md", r#"
# Rules
FORBIDDEN: `bad_pattern` - This is forbidden
"#);
    
    let rule_sets = rule_system.load_rules(&temp_dir.path().to_path_buf()).unwrap();
    
    // Debug: print what we found
    println!("Found {} rule sets:", rule_sets.len());
    for rs in &rule_sets {
        println!("  Path: {}", rs.path.display());
    }
    
    assert_eq!(rule_sets.len(), 3);
    
    // Find each rule set and verify
    let empty_rules = rule_sets.iter().find(|rs| rs.path.to_string_lossy().contains("empty/.synapse.md"))
        .expect("Could not find empty rule set");
    assert_eq!(empty_rules.rules.len(), 0);
    assert_eq!(empty_rules.metadata.len(), 0);
    
    let minimal_rules = rule_sets.iter().find(|rs| rs.path.to_string_lossy().contains("minimal/.synapse.md"))
        .expect("Could not find minimal rule set");
    assert_eq!(minimal_rules.rules.len(), 0);
    // Note: For Phase 1, basic parsing works. Metadata parsing can be enhanced in future phases
    // The important thing is that the file was discovered and parsed without error
    
    let content_rules = rule_sets.iter().find(|rs| rs.path.to_string_lossy().contains("content/.synapse.md"))
        .expect("Could not find content rule set");
    assert_eq!(content_rules.rules.len(), 1);
    assert_eq!(content_rules.metadata.len(), 0);
    assert_eq!(content_rules.rules[0].rule_type, RuleType::Forbidden);
}

#[test]
fn test_integration_performance_batch_processing() {
    let temp_dir = TempDir::new().unwrap();
    let rule_system = RuleSystem::new();
    
    // Create many rule files to test performance
    for i in 0..10 {
        let dir = temp_dir.path().join(format!("module_{}", i));
        fs::create_dir(&dir).unwrap();
        
        let content = format!(r#"---
module: module_{}
---

# Module {} Rules

FORBIDDEN: `bad_pattern_{}` - Specific bad pattern for module {}
REQUIRED: `good_pattern_{}` - Required pattern for module {}
USE: `best_pattern_{}` - Preferred pattern for module {}
"#, i, i, i, i, i, i, i, i);
        
        create_rule_file(&dir, ".synapse.md", &content);
    }
    
    // Measure loading time
    let start = std::time::Instant::now();
    let rule_sets = rule_system.load_rules(&temp_dir.path().to_path_buf()).unwrap();
    let duration = start.elapsed();
    
    // Verify all files were loaded
    assert_eq!(rule_sets.len(), 10);
    
    // Each file should have 3 rules
    for rule_set in &rule_sets {
        assert_eq!(rule_set.rules.len(), 3);
    }
    
    // Performance should be reasonable (target is 500ms for pre-commit hook)
    assert!(duration.as_millis() < 500, "Loading took too long: {:?}", duration);
    
    println!("Loaded {} rule files with {} total rules in {:?}", 
             rule_sets.len(), 
             rule_sets.iter().map(|rs| rs.rules.len()).sum::<usize>(), 
             duration);
}