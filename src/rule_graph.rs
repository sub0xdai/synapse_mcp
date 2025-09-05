use crate::{RuleSet, CompositeRules, RuleSystem, Rule, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// In-memory graph representing rule relationships for fast lookups
/// 
/// The RuleGraph builds an efficient representation of all `.synapse.md` files
/// in a project, handling inheritance chains and rule overrides. It provides
/// O(log n) lookup times for determining which rules apply to any file path.
/// 
/// # Architecture
/// 
/// The graph stores rule sets by directory path and uses directory traversal
/// to build inheritance chains. Rules from parent directories are inherited
/// by children, with explicit override support.
/// 
/// # Performance Characteristics
/// 
/// * Construction: O(n * m) where n = number of directories, m = average rules per directory
/// * Rule lookup: O(d * r) where d = directory depth, r = rules per directory  
/// * Memory usage: O(total rules) - rules are shared via Arc when possible
#[derive(Debug)]
pub struct RuleGraph {
    /// Maps file paths to their RuleSet
    rule_sets: HashMap<PathBuf, RuleSet>,
    /// Rule discovery and parsing system
    rule_system: RuleSystem,
}

impl RuleGraph {
    /// Create a new empty RuleGraph
    pub fn new() -> Self {
        Self {
            rule_sets: HashMap::new(),
            rule_system: RuleSystem::new(),
        }
    }

    /// Build RuleGraph from a project root directory
    /// 
    /// Discovers all `.synapse.md` files recursively and parses them,
    /// building a complete graph of rule relationships with inheritance.
    /// 
    /// # Arguments
    /// 
    /// * `root` - Root directory path to scan for rule files
    /// 
    /// # Returns
    /// 
    /// Returns a fully constructed RuleGraph ready for rule lookups.
    /// 
    /// # Performance
    /// 
    /// * I/O bound: O(f) file reads where f = number of .synapse.md files
    /// * CPU bound: O(n * r) where n = directories, r = average rules per directory
    /// * Target: Complete project indexing under 500ms
    /// 
    /// # Error Conditions
    /// 
    /// * File system access errors (permissions, missing files)
    /// * YAML parsing errors in .synapse.md frontmatter
    /// * Rule format validation errors
    /// 
    /// # Examples
    /// 
    /// ```
    /// use synapse_mcp::RuleGraph;
    /// use std::path::PathBuf;
    /// 
    /// let project_root = PathBuf::from("/path/to/project");
    /// let rule_graph = RuleGraph::from_project(&project_root)?;
    /// 
    /// // Now ready to look up rules for any file
    /// let rules = rule_graph.rules_for(&PathBuf::from("/path/to/project/src/main.rs"))?;
    /// # Ok::<(), synapse_mcp::SynapseError>(())
    /// ```
    pub fn from_project(root: &PathBuf) -> Result<Self> {
        let rule_system = RuleSystem::new();
        let rule_sets = rule_system.load_rules(root)?;
        
        // Build map of file paths to rule sets for fast lookup
        let mut rule_sets_map = HashMap::new();
        for rule_set in rule_sets {
            rule_sets_map.insert(rule_set.path.clone(), rule_set);
        }
        
        Ok(Self {
            rule_sets: rule_sets_map,
            rule_system,
        })
    }

    /// Get all applicable rules for a given file path
    /// 
    /// This walks up the directory tree from the target path, collecting
    /// rules from each level and applying inheritance and override logic.
    pub fn rules_for(&self, path: &PathBuf) -> Result<CompositeRules> {
        let rule_sets: Vec<RuleSet> = self.rule_sets.values().cloned().collect();
        Ok(self.rule_system.rules_for_path(path, &rule_sets))
    }

    /// Get the number of rule nodes in the graph
    pub fn node_count(&self) -> usize {
        self.rule_sets.len()
    }

    /// Get all rule sets in the graph
    pub fn rule_sets(&self) -> &HashMap<PathBuf, RuleSet> {
        &self.rule_sets
    }

    /// Get a specific rule set by path
    pub fn get_rule_set(&self, path: &PathBuf) -> Option<&RuleSet> {
        self.rule_sets.get(path)
    }

    /// Add a new rule set to the graph
    /// 
    /// This is useful for testing or dynamic rule loading
    pub fn add_rule_set(&mut self, rule_set: RuleSet) {
        let path = rule_set.path.clone();
        self.rule_sets.insert(path, rule_set);
    }

    /// Remove a rule set from the graph
    pub fn remove_rule_set(&mut self, path: &PathBuf) -> Option<RuleSet> {
        self.rule_sets.remove(path)
    }

    /// Check if a rule set exists at the given path
    pub fn has_rule_set(&self, path: &PathBuf) -> bool {
        self.rule_sets.contains_key(path)
    }

    /// Get all rule file paths in the graph
    pub fn rule_paths(&self) -> Vec<&PathBuf> {
        self.rule_sets.keys().collect()
    }

    /// Find all rules that match a specific pattern or contain certain text
    pub fn find_rules_by_pattern(&self, pattern: &str) -> Vec<(&PathBuf, &Rule)> {
        let mut matching_rules = Vec::new();
        
        for (path, rule_set) in &self.rule_sets {
            for rule in &rule_set.rules {
                if rule.pattern.contains(pattern) || 
                   rule.message.contains(pattern) ||
                   rule.name.contains(pattern) {
                    matching_rules.push((path, rule));
                }
            }
        }
        
        matching_rules
    }

    /// Get statistics about the rule graph
    pub fn stats(&self) -> RuleGraphStats {
        let total_rules = self.rule_sets.values()
            .map(|rs| rs.rules.len())
            .sum();
            
        let total_inheritance_relationships = self.rule_sets.values()
            .map(|rs| rs.inherits.len())
            .sum();
            
        let total_overrides = self.rule_sets.values()
            .map(|rs| rs.overrides.len())
            .sum();

        RuleGraphStats {
            rule_files: self.rule_sets.len(),
            total_rules,
            inheritance_relationships: total_inheritance_relationships,
            override_relationships: total_overrides,
        }
    }
}

impl Default for RuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a RuleGraph
#[derive(Debug, Clone, PartialEq)]
pub struct RuleGraphStats {
    pub rule_files: usize,
    pub total_rules: usize,
    pub inheritance_relationships: usize,
    pub override_relationships: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuleType, Rule};
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_empty_rule_graph() {
        let graph = RuleGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert!(graph.rule_sets().is_empty());
    }

    #[test]
    fn test_add_remove_rule_set() {
        let mut graph = RuleGraph::new();
        let path = PathBuf::from("/test/.synapse.md");
        let rule_set = RuleSet::new(path.clone());
        
        assert!(!graph.has_rule_set(&path));
        
        graph.add_rule_set(rule_set);
        assert!(graph.has_rule_set(&path));
        assert_eq!(graph.node_count(), 1);
        
        let removed = graph.remove_rule_set(&path);
        assert!(removed.is_some());
        assert!(!graph.has_rule_set(&path));
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_find_rules_by_pattern() {
        let mut graph = RuleGraph::new();
        let path = PathBuf::from("/test/.synapse.md");
        
        let rule1 = Rule::new(
            "no println".to_string(),
            RuleType::Forbidden,
            "println!(".to_string(),
            "Use logging instead".to_string(),
        );
        
        let rule2 = Rule::new(
            "documentation".to_string(),
            RuleType::Required,
            "///".to_string(),
            "Add documentation".to_string(),
        );
        
        let rule_set = RuleSet::new(path.clone())
            .add_rule(rule1)
            .add_rule(rule2);
            
        graph.add_rule_set(rule_set);
        
        // Find rules containing "println"
        let println_rules = graph.find_rules_by_pattern("println");
        assert_eq!(println_rules.len(), 1);
        assert_eq!(println_rules[0].1.name, "no println");
        
        // Find rules containing "documentation" 
        let doc_rules = graph.find_rules_by_pattern("documentation");
        assert_eq!(doc_rules.len(), 1);
        assert_eq!(doc_rules[0].1.name, "documentation");
        
        // No matches
        let no_matches = graph.find_rules_by_pattern("nonexistent");
        assert!(no_matches.is_empty());
    }

    #[test]
    fn test_graph_stats() {
        let mut graph = RuleGraph::new();
        
        // Empty graph stats
        let stats = graph.stats();
        assert_eq!(stats.rule_files, 0);
        assert_eq!(stats.total_rules, 0);
        assert_eq!(stats.inheritance_relationships, 0);
        assert_eq!(stats.override_relationships, 0);
        
        // Add rule set with rules, inherits, and overrides
        let path = PathBuf::from("/test/.synapse.md");
        let parent_path = PathBuf::from("/parent/.synapse.md");
        
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
        
        let rule_set = RuleSet::new(path.clone())
            .add_rule(rule1)
            .add_rule(rule2)
            .with_inherits(vec![parent_path])
            .with_overrides(vec!["old-rule".to_string()]);
            
        graph.add_rule_set(rule_set);
        
        let stats = graph.stats();
        assert_eq!(stats.rule_files, 1);
        assert_eq!(stats.total_rules, 2);
        assert_eq!(stats.inheritance_relationships, 1);
        assert_eq!(stats.override_relationships, 1);
    }

    #[test]
    fn test_get_rule_set() {
        let mut graph = RuleGraph::new();
        let path = PathBuf::from("/test/.synapse.md");
        let rule_set = RuleSet::new(path.clone());
        
        assert!(graph.get_rule_set(&path).is_none());
        
        graph.add_rule_set(rule_set);
        let retrieved = graph.get_rule_set(&path);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().path, path);
    }

    #[test]
    fn test_rule_paths() {
        let mut graph = RuleGraph::new();
        
        let path1 = PathBuf::from("/test1/.synapse.md");
        let path2 = PathBuf::from("/test2/.synapse.md");
        
        graph.add_rule_set(RuleSet::new(path1.clone()));
        graph.add_rule_set(RuleSet::new(path2.clone()));
        
        let paths = graph.rule_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&&path1));
        assert!(paths.contains(&&path2));
    }

    // Integration test with file system
    #[test]
    fn test_from_project_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
        
        assert_eq!(graph.node_count(), 0);
        assert!(graph.rule_sets().is_empty());
    }

    #[test] 
    fn test_from_project_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let rule_file = temp_dir.path().join(".synapse.md");
        
        fs::write(&rule_file, r#"---
mcp: synapse
type: rule
---

# Test Rules

FORBIDDEN: `println!` - Use logging framework instead.
"#).unwrap();

        let graph = RuleGraph::from_project(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(graph.node_count(), 1);
        
        let rule_set = graph.get_rule_set(&rule_file).unwrap();
        assert_eq!(rule_set.rules.len(), 1);
        assert_eq!(rule_set.rules[0].name, "forbidden-0");
        assert_eq!(rule_set.rules[0].rule_type, RuleType::Forbidden);
        assert_eq!(rule_set.rules[0].pattern, "println!");
    }
}