use crate::models::{RuleSet, Rule, RuleType, CompiledRule};
use regex::Regex;
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize, Debug)]
struct RuleFrontmatter {
    inherits: Option<Vec<String>>,
    overrides: Option<Vec<String>>,
    project: Option<String>,
    module: Option<String>,
    #[serde(flatten)]
    metadata: HashMap<String, serde_yaml::Value>,
}

pub struct RuleParser {
    frontmatter_regex: Regex,
}

impl RuleParser {
    pub fn new() -> Self {
        Self {
            frontmatter_regex: Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n").unwrap(),
        }
    }

    /// Parse a .synapse.md rule file
    pub fn parse_rule_file(&self, file_path: &Path) -> crate::Result<RuleSet> {
        let content = fs::read_to_string(file_path)?;
        self.parse_content(&content, file_path.to_path_buf())
    }

    /// Parse rule content from string
    pub fn parse_content(&self, content: &str, file_path: PathBuf) -> crate::Result<RuleSet> {
        let (frontmatter_opt, markdown_content) = self.extract_frontmatter(content)?;
        
        let mut rule_set = RuleSet::new(file_path);

        // Parse frontmatter if present
        if let Some(frontmatter_yaml) = frontmatter_opt {
            let frontmatter: RuleFrontmatter = serde_yaml::from_str(&frontmatter_yaml)?;
            
            // Handle inheritance
            if let Some(inherits) = frontmatter.inherits {
                let inherit_paths: Vec<PathBuf> = inherits.iter()
                    .map(|p| PathBuf::from(p))
                    .collect();
                rule_set = rule_set.with_inherits(inherit_paths);
            }

            // Handle overrides
            if let Some(overrides) = frontmatter.overrides {
                rule_set = rule_set.with_overrides(overrides);
            }

            // Convert metadata
            let mut metadata = HashMap::new();
            if let Some(project) = frontmatter.project {
                metadata.insert("project".to_string(), project);
            }
            if let Some(module) = frontmatter.module {
                metadata.insert("module".to_string(), module);
            }
            
            for (key, value) in frontmatter.metadata {
                if !key.starts_with('@') {
                    let value_str = match value {
                        serde_yaml::Value::String(s) => s,
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        _ => serde_yaml::to_string(&value).unwrap_or_else(|_| "".to_string()),
                    };
                    metadata.insert(key, value_str);
                }
            }
            
            if !metadata.is_empty() {
                rule_set = rule_set.with_metadata(metadata);
            }
        }

        // Parse markdown content for rules
        let compiled_rules = self.extract_compiled_rules(&markdown_content)?;
        for compiled_rule in compiled_rules {
            rule_set = rule_set.add_rule((*compiled_rule.rule).clone());
        }

        rule_set.validate()?;
        Ok(rule_set)
    }

    /// Extract frontmatter from content
    fn extract_frontmatter(&self, content: &str) -> crate::Result<(Option<String>, String)> {
        if let Some(captures) = self.frontmatter_regex.captures(content) {
            let frontmatter = captures.get(1).unwrap().as_str().to_string();
            let remaining = self.frontmatter_regex.replace(content, "").to_string();
            Ok((Some(frontmatter), remaining))
        } else {
            Ok((None, content.to_string()))
        }
    }

    /// Extract compiled rules from markdown content
    fn extract_compiled_rules(&self, content: &str) -> crate::Result<Vec<CompiledRule>> {
        let mut compiled_rules = Vec::new();
        
        // Simple rule extraction - look for specific patterns
        // This is a basic implementation, could be enhanced with more sophisticated parsing
        
        // Look for "FORBIDDEN" patterns
        if let Some(forbidden_rules) = self.extract_forbidden_rules(content) {
            for rule in forbidden_rules {
                compiled_rules.push(CompiledRule::from_rule(rule));
            }
        }

        // Look for "REQUIRED" patterns  
        if let Some(required_rules) = self.extract_required_rules(content) {
            for rule in required_rules {
                compiled_rules.push(CompiledRule::from_rule(rule));
            }
        }

        // Look for "STANDARD" patterns
        if let Some(standard_rules) = self.extract_standard_rules(content) {
            for rule in standard_rules {
                compiled_rules.push(CompiledRule::from_rule(rule));
            }
        }

        Ok(compiled_rules)
    }

    /// Extract rules from markdown content (legacy method for tests)
    fn extract_rules(&self, content: &str) -> crate::Result<Vec<Rule>> {
        let compiled_rules = self.extract_compiled_rules(content)?;
        Ok(compiled_rules.into_iter()
            .map(|cr| (*cr.rule).clone())
            .collect())
    }

    fn extract_forbidden_rules(&self, content: &str) -> Option<Vec<Rule>> {
        let forbidden_regex = Regex::new(r"(?i)(?:forbidden|never|must not):\s*`([^`]+)`\s*-\s*(.+)").ok()?;
        let mut rules = Vec::new();

        for captures in forbidden_regex.captures_iter(content) {
            let pattern = captures.get(1)?.as_str();
            let message = captures.get(2)?.as_str();
            
            let rule = Rule::new(
                format!("forbidden-{}", rules.len()),
                RuleType::Forbidden,
                pattern.to_string(),
                message.to_string(),
            );
            rules.push(rule);
        }

        if rules.is_empty() { None } else { Some(rules) }
    }

    fn extract_required_rules(&self, content: &str) -> Option<Vec<Rule>> {
        let required_regex = Regex::new(r"(?i)(?:required|must|mandatory):\s*`([^`]+)`\s*-\s*(.+)").ok()?;
        let mut rules = Vec::new();

        for captures in required_regex.captures_iter(content) {
            let pattern = captures.get(1)?.as_str();
            let message = captures.get(2)?.as_str();
            
            let rule = Rule::new(
                format!("required-{}", rules.len()),
                RuleType::Required,
                pattern.to_string(),
                message.to_string(),
            );
            rules.push(rule);
        }

        if rules.is_empty() { None } else { Some(rules) }
    }

    fn extract_standard_rules(&self, content: &str) -> Option<Vec<Rule>> {
        let standard_regex = Regex::new(r"(?i)(?:use|prefer|should):\s*`([^`]+)`\s*-\s*(.+)").ok()?;
        let mut rules = Vec::new();

        for captures in standard_regex.captures_iter(content) {
            let pattern = captures.get(1)?.as_str();
            let message = captures.get(2)?.as_str();
            
            let rule = Rule::new(
                format!("standard-{}", rules.len()),
                RuleType::Standard,
                pattern.to_string(),
                message.to_string(),
            );
            rules.push(rule);
        }

        if rules.is_empty() { None } else { Some(rules) }
    }
}

impl Default for RuleParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_parse_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let parser = RuleParser::new();
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", "");
        
        let result = parser.parse_rule_file(&file_path).unwrap();
        assert_eq!(result.rules.len(), 0);
        assert_eq!(result.inherits.len(), 0);
        assert_eq!(result.overrides.len(), 0);
    }

    #[test]
    fn test_parse_frontmatter_only() {
        let temp_dir = TempDir::new().unwrap();
        let parser = RuleParser::new();
        
        let content = r#"---
inherits:
  - "../.synapse.md"
overrides:
  - "old-rule-1"
project: test-project
module: test-module
---
"#;
        
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", content);
        let result = parser.parse_rule_file(&file_path).unwrap();
        
        assert_eq!(result.inherits.len(), 1);
        assert_eq!(result.inherits[0], PathBuf::from("../.synapse.md"));
        assert_eq!(result.overrides.len(), 1);
        assert_eq!(result.overrides[0], "old-rule-1");
        assert_eq!(result.metadata.get("project").unwrap(), "test-project");
        assert_eq!(result.metadata.get("module").unwrap(), "test-module");
    }

    #[test]
    fn test_parse_forbidden_rules() {
        let temp_dir = TempDir::new().unwrap();
        let parser = RuleParser::new();
        
        let content = r#"
# Forbidden Patterns

FORBIDDEN: `println!` - Use logging instead
Never: `unwrap()` - Handle errors properly
MUST NOT: `todo!()` - Complete implementation
"#;
        
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", content);
        let result = parser.parse_rule_file(&file_path).unwrap();
        
        assert_eq!(result.rules.len(), 3);
        assert_eq!(result.rules[0].rule_type, RuleType::Forbidden);
        assert_eq!(result.rules[0].pattern, "println!");
        assert!(result.rules[0].message.contains("logging"));
    }

    #[test]
    fn test_parse_required_rules() {
        let temp_dir = TempDir::new().unwrap();
        let parser = RuleParser::new();
        
        let content = r#"
# Required Patterns

REQUIRED: `#[test]` - All functions must have tests
Mandatory: `use crate::` - Use explicit crate imports
Must: `Result<T>` - Functions must return Results
"#;
        
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", content);
        let result = parser.parse_rule_file(&file_path).unwrap();
        
        assert_eq!(result.rules.len(), 3);
        assert_eq!(result.rules[0].rule_type, RuleType::Required);
        assert_eq!(result.rules[0].pattern, "#[test]");
        assert!(result.rules[0].message.contains("tests"));
    }

    #[test]
    fn test_parse_standard_rules() {
        let temp_dir = TempDir::new().unwrap();
        let parser = RuleParser::new();
        
        let content = r#"
# Standard Patterns

USE: `Vec<T>` - Prefer Vec over arrays
Prefer: `String` - Use String over &str for owned data
Should: `async fn` - Use async functions for IO
"#;
        
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", content);
        let result = parser.parse_rule_file(&file_path).unwrap();
        
        assert_eq!(result.rules.len(), 3);
        assert_eq!(result.rules[0].rule_type, RuleType::Standard);
        assert_eq!(result.rules[0].pattern, "Vec<T>");
        assert!(result.rules[0].message.contains("arrays"));
    }

    #[test]
    fn test_parse_complete_file() {
        let temp_dir = TempDir::new().unwrap();
        let parser = RuleParser::new();
        
        let content = r#"---
inherits:
  - "../.synapse.md"
project: test-project
custom_field: custom_value
---

# Test Rules

## Coding Standards

FORBIDDEN: `println!` - Use logging macros instead
REQUIRED: `#[test]` - All public functions need tests
USE: `Result<T>` - Prefer Result for error handling

## Architecture Rules

Never: `global variables` - Use dependency injection
Mandatory: `mod tests` - Each module must have a test module
"#;
        
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", content);
        let result = parser.parse_rule_file(&file_path).unwrap();
        
        // Check frontmatter parsing
        assert_eq!(result.inherits.len(), 1);
        assert_eq!(result.metadata.get("project").unwrap(), "test-project");
        assert_eq!(result.metadata.get("custom_field").unwrap(), "custom_value");
        
        // Check rule parsing  
        assert_eq!(result.rules.len(), 5);
        
        let forbidden_count = result.rules.iter().filter(|r| r.rule_type == RuleType::Forbidden).count();
        let required_count = result.rules.iter().filter(|r| r.rule_type == RuleType::Required).count();
        let standard_count = result.rules.iter().filter(|r| r.rule_type == RuleType::Standard).count();
        
        assert_eq!(forbidden_count, 2); // println!, global variables
        assert_eq!(required_count, 2); // #[test], mod tests  
        assert_eq!(standard_count, 1); // Result<T>
    }

    #[test]
    fn test_extract_frontmatter() {
        let parser = RuleParser::new();
        
        let content_with_frontmatter = r#"---
key: value
---
# Content here"#;
        
        let (frontmatter, remaining) = parser.extract_frontmatter(content_with_frontmatter).unwrap();
        assert!(frontmatter.is_some());
        assert!(frontmatter.unwrap().contains("key: value"));
        assert_eq!(remaining, "# Content here");
        
        let content_without_frontmatter = "# Just content";
        let (frontmatter, remaining) = parser.extract_frontmatter(content_without_frontmatter).unwrap();
        assert!(frontmatter.is_none());
        assert_eq!(remaining, "# Just content");
    }

    #[test] 
    fn test_invalid_yaml_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let parser = RuleParser::new();
        
        let content = r#"---
invalid: [unclosed array
  nested:
    badly: {unclosed map
---
# Content
"#;
        
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", content);
        let result = parser.parse_rule_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_rule_parsing() {
        let temp_dir = TempDir::new().unwrap();  
        let parser = RuleParser::new();
        
        let content = r#"
forbidden: `bad_pattern` - This is forbidden
REQUIRED: `good_pattern` - This is required
use: `best_pattern` - This is preferred
"#;
        
        let file_path = create_test_file(temp_dir.path(), ".synapse.md", content);
        let result = parser.parse_rule_file(&file_path).unwrap();
        
        assert_eq!(result.rules.len(), 3);
    }
}