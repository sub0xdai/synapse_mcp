pub mod discovery;
pub mod parser;

pub use discovery::RuleDiscovery;
pub use parser::RuleParser;
use crate::models::{RuleSet, CompositeRules};
use std::path::PathBuf;

/// Main interface for rule system
#[derive(Debug)]
pub struct RuleSystem {
    discovery: RuleDiscovery,
    pub parser: RuleParser,
}

impl RuleSystem {
    pub fn new() -> Self {
        Self {
            discovery: RuleDiscovery::new(),
            parser: RuleParser::new(),
        }
    }

    /// Find and parse all .synapse.md files in a directory tree
    pub fn load_rules(&self, root_path: &PathBuf) -> crate::Result<Vec<RuleSet>> {
        let rule_files = self.discovery.find_rule_files(root_path)?;
        let mut rule_sets = Vec::new();

        for file_path in rule_files {
            match self.parser.parse_rule_file(&file_path) {
                Ok(rule_set) => rule_sets.push(rule_set),
                Err(e) => {
                    eprintln!("Warning: Failed to parse rule file {}: {}", file_path.display(), e);
                    continue;
                }
            }
        }

        Ok(rule_sets)
    }

    /// Build composite rules for a specific file path considering inheritance
    pub fn rules_for_path(&self, target_path: &PathBuf, rule_sets: &[RuleSet]) -> CompositeRules {
        let mut composite = CompositeRules::new();
        let mut inheritance_chain = Vec::new();
        let mut applicable_rule_sets = Vec::new();
        let mut visited_paths = std::collections::HashSet::new();

        // First, collect all applicable rule sets by walking up the directory tree
        let mut current_dir = target_path.parent();
        
        while let Some(dir) = current_dir {
            let potential_rule_file = dir.join(".synapse.md");
            
            // Find matching rule set
            if let Some(rule_set) = rule_sets.iter().find(|rs| rs.path == potential_rule_file) {
                if !visited_paths.contains(&rule_set.path) {
                    inheritance_chain.push(rule_set.path.clone());
                    applicable_rule_sets.push(rule_set);
                    visited_paths.insert(rule_set.path.clone());
                    
                    // Follow explicit inheritance paths
                    self.add_inherited_rule_sets(rule_set, rule_sets, &mut applicable_rule_sets, 
                                                &mut inheritance_chain, &mut visited_paths);
                }
            }

            current_dir = dir.parent();
        }

        // Second, collect all overrides from all rule sets (children override parents)
        for rule_set in &applicable_rule_sets {
            for override_id in &rule_set.overrides {
                composite = composite.add_override(override_id.clone());
            }
        }

        // Third, add rules from all levels, skipping overridden ones
        // Process in reverse order so children's rules come first (proper precedence)
        for rule_set in applicable_rule_sets.iter().rev() {
            for rule in rule_set.rules.iter().rev() {
                // Skip if rule is overridden (check both ID and name for compatibility)
                if !composite.overridden_rules.contains(&rule.id) && 
                   !composite.overridden_rules.contains(&rule.name) {
                    composite = composite.add_rule(rule.clone());
                }
            }
        }

        composite.with_inheritance_chain(inheritance_chain)
    }

    /// Helper method to recursively add inherited rule sets
    fn add_inherited_rule_sets<'a>(&self, 
                                   rule_set: &RuleSet, 
                                   all_rule_sets: &'a [RuleSet],
                                   applicable_rule_sets: &mut Vec<&'a RuleSet>,
                                   inheritance_chain: &mut Vec<PathBuf>,
                                   visited_paths: &mut std::collections::HashSet<PathBuf>) {
        
        for inherit_path in &rule_set.inherits {
            // Resolve relative paths relative to the current rule set's directory
            let base_dir = rule_set.path.parent().unwrap_or_else(|| std::path::Path::new("."));
            let absolute_inherit_path = base_dir.join(inherit_path).canonicalize()
                .unwrap_or_else(|_| base_dir.join(inherit_path));
            
            // Find the inherited rule set
            if let Some(inherited_rule_set) = all_rule_sets.iter()
                .find(|rs| rs.path.canonicalize().unwrap_or_else(|_| rs.path.clone()) == absolute_inherit_path ||
                           rs.path == absolute_inherit_path) {
                
                if !visited_paths.contains(&inherited_rule_set.path) {
                    inheritance_chain.push(inherited_rule_set.path.clone());
                    applicable_rule_sets.push(inherited_rule_set);
                    visited_paths.insert(inherited_rule_set.path.clone());
                    
                    // Recursively follow inheritance chain
                    self.add_inherited_rule_sets(inherited_rule_set, all_rule_sets, 
                                                applicable_rule_sets, inheritance_chain, visited_paths);
                }
            }
        }
    }
}

impl Default for RuleSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rule_system_creation() {
        let _rule_system = RuleSystem::new();
        // Basic smoke test - should not panic
        assert!(true);
    }

    #[test] 
    fn test_load_rules_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let rule_system = RuleSystem::new();
        
        let result = rule_system.load_rules(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_rules_for_path_no_rules() {
        let rule_system = RuleSystem::new();
        let rule_sets = vec![];
        let target_path = PathBuf::from("/some/file.rs");
        
        let composite = rule_system.rules_for_path(&target_path, &rule_sets);
        assert_eq!(composite.applicable_rules.len(), 0);
        assert_eq!(composite.inheritance_chain.len(), 0);
    }
}