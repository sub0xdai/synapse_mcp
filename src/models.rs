use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use regex::Regex;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NodeType {
    File,
    Rule,
    Decision,
    Function,
    Architecture,
    Component,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub content: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EdgeType {
    RelatesTo,
    ImplementsRule,
    DefinedIn,
    DependsOn,
    Contains,
    References,
    Inherits,
    Overrides,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub label: String,
    pub metadata: HashMap<String, String>,
}

// Phase 1: Rule-specific data structures

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RuleType {
    Forbidden,    // Pattern that must not exist
    Required,     // Pattern that must exist
    Standard,     // Preferred pattern with suggestions
    Convention,   // Style/naming convention
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub message: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuleSet {
    pub path: PathBuf,
    pub inherits: Vec<PathBuf>,
    pub overrides: Vec<String>,  // Rule IDs to override
    pub rules: Vec<Rule>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuleNode {
    pub path: PathBuf,
    pub rule_set: RuleSet,
    pub parent: Option<PathBuf>,
    pub children: Vec<PathBuf>,
}

// Phase 1: Performance-optimized structures

#[derive(Debug, Clone)]
pub enum PatternMatcher {
    Regex(Regex),
    Literal(String),
}

#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub rule: Arc<Rule>,
    pub matcher: PatternMatcher,
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub file_path: PathBuf,
    pub rule: Arc<Rule>,
    pub line_number: Option<usize>,
    pub line_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompositeRules {
    pub applicable_rules: Vec<Rule>,
    pub inheritance_chain: Vec<PathBuf>,
    pub overridden_rules: Vec<String>,
}

impl Node {
    pub fn new(node_type: NodeType, label: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            node_type,
            label,
            content,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.label.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Label cannot be empty".to_string()));
        }
        if self.content.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Content cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl Edge {
    pub fn new(source_id: String, target_id: String, edge_type: EdgeType, label: String) -> Self {
        Self {
            source_id,
            target_id,
            edge_type,
            label,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.source_id.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Source ID cannot be empty".to_string()));
        }
        if self.target_id.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Target ID cannot be empty".to_string()));
        }
        if self.source_id == self.target_id {
            return Err(crate::SynapseError::Validation("Source and target cannot be the same".to_string()));
        }
        Ok(())
    }
}

impl Rule {
    pub fn new(name: String, rule_type: RuleType, pattern: String, message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            rule_type,
            pattern,
            message,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Rule name cannot be empty".to_string()));
        }
        if self.pattern.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Rule pattern cannot be empty".to_string()));
        }
        if self.message.trim().is_empty() {
            return Err(crate::SynapseError::Validation("Rule message cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl RuleSet {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            inherits: Vec::new(),
            overrides: Vec::new(),
            rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_inherits(mut self, inherits: Vec<PathBuf>) -> Self {
        self.inherits = inherits;
        self
    }

    pub fn with_overrides(mut self, overrides: Vec<String>) -> Self {
        self.overrides = overrides;
        self
    }

    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> crate::Result<()> {
        for rule in &self.rules {
            rule.validate()?;
        }
        Ok(())
    }
}

impl RuleNode {
    pub fn new(path: PathBuf, rule_set: RuleSet) -> Self {
        Self {
            path,
            rule_set,
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent: PathBuf) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn add_child(mut self, child: PathBuf) -> Self {
        self.children.push(child);
        self
    }
}

impl CompositeRules {
    pub fn new() -> Self {
        Self {
            applicable_rules: Vec::new(),
            inheritance_chain: Vec::new(),
            overridden_rules: Vec::new(),
        }
    }

    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.applicable_rules.push(rule);
        self
    }

    pub fn with_inheritance_chain(mut self, chain: Vec<PathBuf>) -> Self {
        self.inheritance_chain = chain;
        self
    }

    pub fn add_override(mut self, rule_id: String) -> Self {
        self.overridden_rules.push(rule_id);
        self
    }
}

impl Default for CompositeRules {
    fn default() -> Self {
        Self::new()
    }
}

// Phase 1: New implementations

impl CompiledRule {
    pub fn new(rule: Rule, matcher: PatternMatcher) -> Self {
        Self {
            rule: Arc::new(rule),
            matcher,
        }
    }

    pub fn from_rule(rule: Rule) -> Self {
        let pattern = rule.pattern.clone(); // Clone once upfront
        let matcher = match Regex::new(&pattern) {
            Ok(regex) => PatternMatcher::Regex(regex),
            Err(_) => PatternMatcher::Literal(pattern), // Move instead of clone
        };
        Self::new(rule, matcher)
    }
}

impl Violation {
    pub fn new(
        file_path: PathBuf,
        rule: Arc<Rule>,
        line_number: Option<usize>,
        line_content: Option<String>,
    ) -> Self {
        Self {
            file_path,
            rule,
            line_number,
            line_content,
        }
    }

    pub fn from_compiled_rule(
        file_path: PathBuf,
        compiled_rule: &CompiledRule,
        line_number: Option<usize>,
        line_content: Option<String>,
    ) -> Self {
        Self::new(
            file_path,
            compiled_rule.rule.clone(),
            line_number,
            line_content,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation_and_validation() {
        let rule = Rule::new(
            "no-println".to_string(),
            RuleType::Forbidden,
            r"println!\(".to_string(),
            "Use logging instead of println!".to_string(),
        );

        assert_eq!(rule.name, "no-println");
        assert_eq!(rule.rule_type, RuleType::Forbidden);
        assert!(rule.validate().is_ok());
    }

    #[test]
    fn test_rule_validation_empty_fields() {
        let rule = Rule::new(
            "".to_string(),
            RuleType::Required,
            "test".to_string(),
            "message".to_string(),
        );
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_rule_with_tags_and_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("severity".to_string(), "high".to_string());

        let rule = Rule::new(
            "test-rule".to_string(),
            RuleType::Standard,
            "pattern".to_string(),
            "message".to_string(),
        )
        .with_tags(vec!["rust".to_string(), "style".to_string()])
        .with_metadata(metadata);

        assert_eq!(rule.tags.len(), 2);
        assert_eq!(rule.metadata.get("severity").unwrap(), "high");
    }

    #[test]
    fn test_rule_set_creation() {
        let path = PathBuf::from("/project/.synapse.md");
        let rule_set = RuleSet::new(path.clone());

        assert_eq!(rule_set.path, path);
        assert!(rule_set.rules.is_empty());
        assert!(rule_set.inherits.is_empty());
    }

    #[test]
    fn test_rule_set_with_inheritance() {
        let path = PathBuf::from("/project/src/.synapse.md");
        let parent_path = PathBuf::from("/project/.synapse.md");
        
        let rule_set = RuleSet::new(path.clone())
            .with_inherits(vec![parent_path.clone()]);

        assert_eq!(rule_set.inherits.len(), 1);
        assert_eq!(rule_set.inherits[0], parent_path);
    }

    #[test]
    fn test_rule_node_hierarchy() {
        let path = PathBuf::from("/project/src/.synapse.md");
        let parent_path = PathBuf::from("/project/.synapse.md");
        let child_path = PathBuf::from("/project/src/utils/.synapse.md");
        
        let rule_set = RuleSet::new(path.clone());
        let rule_node = RuleNode::new(path.clone(), rule_set)
            .with_parent(parent_path.clone())
            .add_child(child_path.clone());

        assert_eq!(rule_node.parent, Some(parent_path));
        assert_eq!(rule_node.children.len(), 1);
        assert_eq!(rule_node.children[0], child_path);
    }

    #[test]
    fn test_composite_rules() {
        let rule1 = Rule::new(
            "rule1".to_string(),
            RuleType::Required,
            "pattern1".to_string(),
            "message1".to_string(),
        );

        let rule2 = Rule::new(
            "rule2".to_string(),
            RuleType::Forbidden,
            "pattern2".to_string(),
            "message2".to_string(),
        );

        let composite = CompositeRules::new()
            .add_rule(rule1)
            .add_rule(rule2)
            .with_inheritance_chain(vec![
                PathBuf::from("/project/.synapse.md"),
                PathBuf::from("/project/src/.synapse.md"),
            ])
            .add_override("old-rule-id".to_string());

        assert_eq!(composite.applicable_rules.len(), 2);
        assert_eq!(composite.inheritance_chain.len(), 2);
        assert_eq!(composite.overridden_rules.len(), 1);
    }

    #[test]
    fn test_rule_types() {
        let forbidden = RuleType::Forbidden;
        let required = RuleType::Required;
        let standard = RuleType::Standard;
        let convention = RuleType::Convention;

        // Test that they're all different
        assert_ne!(forbidden, required);
        assert_ne!(standard, convention);
    }

    #[test]
    fn test_edge_types_include_rule_specific() {
        let inherits = EdgeType::Inherits;
        let overrides = EdgeType::Overrides;
        
        assert_ne!(inherits, overrides);
        assert_ne!(inherits, EdgeType::RelatesTo);
    }

    // Phase 1: New tests
    
    #[test]
    fn test_compiled_rule_with_valid_regex() {
        let rule = Rule::new(
            "no-println".to_string(),
            RuleType::Forbidden,
            r"println!\(".to_string(),
            "Use logging instead of println!".to_string(),
        );

        let compiled_rule = CompiledRule::from_rule(rule);
        
        match compiled_rule.matcher {
            PatternMatcher::Regex(_) => {}, // Success
            PatternMatcher::Literal(_) => panic!("Expected regex, got literal"),
        }
    }
    
    #[test]
    fn test_compiled_rule_with_invalid_regex() {
        let rule = Rule::new(
            "bad-pattern".to_string(),
            RuleType::Forbidden,
            "[invalid regex".to_string(), // Invalid regex
            "This has a bad pattern".to_string(),
        );

        let compiled_rule = CompiledRule::from_rule(rule);
        
        match compiled_rule.matcher {
            PatternMatcher::Literal(pattern) => {
                assert_eq!(pattern, "[invalid regex");
            },
            PatternMatcher::Regex(_) => panic!("Expected literal fallback, got regex"),
        }
    }
    
    #[test]
    fn test_violation_creation() {
        let rule = Rule::new(
            "test-rule".to_string(),
            RuleType::Required,
            "test-pattern".to_string(),
            "test message".to_string(),
        );
        
        let file_path = PathBuf::from("test.rs");
        let violation = Violation::new(
            file_path.clone(),
            Arc::new(rule.clone()),
            Some(42),
            Some("test line".to_string()),
        );
        
        assert_eq!(violation.file_path, file_path);
        assert_eq!(violation.rule.name, "test-rule");
        assert_eq!(violation.line_number, Some(42));
        assert_eq!(violation.line_content, Some("test line".to_string()));
    }
    
    #[test]
    fn test_violation_from_compiled_rule() {
        let rule = Rule::new(
            "compiled-test".to_string(),
            RuleType::Forbidden,
            "bad_pattern".to_string(),
            "Don't use bad_pattern".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = PathBuf::from("src/main.rs");
        
        let violation = Violation::from_compiled_rule(
            file_path.clone(),
            &compiled_rule,
            Some(100),
            Some("let x = bad_pattern();".to_string()),
        );
        
        assert_eq!(violation.file_path, file_path);
        assert_eq!(violation.rule.name, "compiled-test");
        assert_eq!(violation.line_number, Some(100));
    }
}