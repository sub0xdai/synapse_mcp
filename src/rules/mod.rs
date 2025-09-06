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
                    // Only warn for actual parse errors, silently skip non-synapse files
                    let error_msg = e.to_string();
                    if error_msg.contains("not marked for synapse MCP") 
                        || error_msg.contains("missing 'mcp' field") 
                        || error_msg.contains("no YAML frontmatter") {
                        // Silently skip files without synapse marker
                        continue;
                    } else {
                        eprintln!("Warning: Failed to parse rule file {}: {}", file_path.display(), e);
                        continue;
                    }
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

        // Create a map from canonical DIRECTORY path to its RuleSet
        let dir_rule_map: std::collections::HashMap<PathBuf, &RuleSet> = rule_sets
            .iter()
            .filter_map(|rs| {
                rs.path.parent()
                    .and_then(|p| p.canonicalize().ok())
                    .map(|canon_dir| (canon_dir, rs))
            })
            .collect();

        // Canonicalize the target path once
        let canonical_target = match target_path.canonicalize() {
            Ok(path) => path,
            Err(_) => target_path.to_path_buf(),
        };

        // Walk up the directory tree, looking up DIRECTORIES in the map
        let mut current_dir = canonical_target.parent();
        while let Some(dir) = current_dir {
            if let Ok(canon_dir) = dir.canonicalize() {
                if let Some(rule_set) = dir_rule_map.get(&canon_dir) {
                    if visited_paths.insert(rule_set.path.clone()) {
                        applicable_rule_sets.push(*rule_set);
                        self.add_inherited_rule_sets(
                            rule_set,
                            &dir_rule_map,
                            &mut applicable_rule_sets,
                            &mut visited_paths,
                        );
                    }
                }
            }
            current_dir = dir.parent();
        }

        inheritance_chain.extend(visited_paths.iter().cloned());

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
                                   dir_rule_map: &'a std::collections::HashMap<PathBuf, &RuleSet>,
                                   applicable_rule_sets: &mut Vec<&'a RuleSet>,
                                   visited_paths: &mut std::collections::HashSet<PathBuf>) {
        for inherit_path in &rule_set.inherits {
            let base_dir = rule_set.path.parent().unwrap_or_else(|| std::path::Path::new("."));
            if let Ok(absolute_inherit_path) = base_dir.join(inherit_path).canonicalize() {
                // The inherited path could be a file or a directory. We check for both.
                // Case 1: Path is a directory.
                if let Some(inherited_rule_set) = dir_rule_map.get(&absolute_inherit_path) {
                    if visited_paths.insert(inherited_rule_set.path.clone()) {
                        applicable_rule_sets.push(*inherited_rule_set);
                        self.add_inherited_rule_sets(inherited_rule_set, dir_rule_map, applicable_rule_sets, visited_paths);
                    }
                // Case 2: Path is a file, so we get its parent directory.
                } else if let Some(parent_dir) = absolute_inherit_path.parent() {
                    if let Some(inherited_rule_set) = dir_rule_map.get(parent_dir) {
                        if visited_paths.insert(inherited_rule_set.path.clone()) {
                            applicable_rule_sets.push(*inherited_rule_set);
                            self.add_inherited_rule_sets(inherited_rule_set, dir_rule_map, applicable_rule_sets, visited_paths);
                        }
                    }
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