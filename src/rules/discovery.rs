use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct RuleDiscovery;

impl RuleDiscovery {
    pub fn new() -> Self {
        Self
    }

    /// Find all .md files inside .synapse/ directories in a directory tree
    pub fn find_rule_files(&self, root_path: &Path) -> crate::Result<Vec<PathBuf>> {
        let mut rule_files = Vec::new();

        // Find all .synapse directories and their .md files
        for entry in WalkDir::new(root_path)
            .into_iter()
            .filter_entry(|e| {
                // Skip .git and other common ignore directories
                e.file_name() != ".git" && 
                e.file_name() != "target" && 
                e.file_name() != "node_modules"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Check if this is a .md file inside a .synapse directory
            if path.is_file() && 
               path.extension() == Some("md".as_ref()) &&
               path.components().any(|c| c.as_os_str() == ".synapse") {
                rule_files.push(path.to_path_buf());
            }
        }

        rule_files.sort();
        Ok(rule_files)
    }

    /// Check if a file is inside a .synapse directory and is a .md file
    pub fn is_rule_file(&self, path: &Path) -> bool {
        path.is_file() && 
        path.extension() == Some("md".as_ref()) &&
        path.components().any(|c| c.as_os_str() == ".synapse")
    }

    /// Find the nearest parent .synapse directory and return all .md files in it
    pub fn find_parent_rule_files(&self, target_path: &Path) -> Vec<PathBuf> {
        let mut current_dir = if target_path.is_dir() {
            Some(target_path)
        } else {
            target_path.parent()
        };

        while let Some(dir) = current_dir {
            let synapse_dir = dir.join(".synapse");
            if synapse_dir.exists() && synapse_dir.is_dir() {
                let mut rule_files = Vec::new();
                if let Ok(entries) = std::fs::read_dir(synapse_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() && path.extension() == Some("md".as_ref()) {
                            rule_files.push(path);
                        }
                    }
                }
                rule_files.sort();
                return rule_files;
            }
            current_dir = dir.parent();
        }

        Vec::new()
    }

    /// Find all parent .synapse directories and their .md files walking up the directory tree
    pub fn find_inheritance_chain(&self, target_path: &Path) -> Vec<PathBuf> {
        let mut chain = Vec::new();
        let mut current_dir = if target_path.is_dir() {
            Some(target_path)
        } else {
            target_path.parent()
        };

        while let Some(dir) = current_dir {
            let synapse_dir = dir.join(".synapse");
            if synapse_dir.exists() && synapse_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(synapse_dir) {
                    let mut dir_rule_files = Vec::new();
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() && path.extension() == Some("md".as_ref()) {
                            dir_rule_files.push(path);
                        }
                    }
                    dir_rule_files.sort();
                    chain.extend(dir_rule_files);
                }
            }
            current_dir = dir.parent();
        }

        chain
    }
}

impl Default for RuleDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use crate::test_helpers::test_helpers::{TestProject, create_rule_content};

    #[test]
    fn test_find_rule_files_empty_directory() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        let result = discovery.find_rule_files(project.root()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_rule_files_single_file() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        project.add_rule_file(".synapse/rules.md", &create_rule_content(&[("FORBIDDEN", "TODO")])).unwrap();
        
        let result = discovery.find_rule_files(project.root()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name().unwrap(), "rules.md");
        assert!(result[0].to_string_lossy().contains(".synapse"));
    }

    #[test]
    fn test_find_rule_files_nested_directories() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create nested .synapse directories 
        project.create_nested_synapse(&["src", "src/utils"]).unwrap();
        
        // Create rule files at different levels
        project.add_rule_file(".synapse/root.md", &create_rule_content(&[("FORBIDDEN", "TODO")])).unwrap();
        project.add_rule_file("src/.synapse/src_rules.md", &create_rule_content(&[("REQUIRED", "#[test]")])).unwrap();
        project.add_rule_file("src/utils/.synapse/utils_rules.md", &create_rule_content(&[("STANDARD", "inline")])).unwrap();
        
        let result = discovery.find_rule_files(project.root()).unwrap();
        assert_eq!(result.len(), 3);
        
        // Should be sorted by path
        assert!(result[0].to_string_lossy().contains(".synapse/root.md"));
        assert!(result[1].to_string_lossy().contains("src/.synapse/src_rules.md"));
        assert!(result[2].to_string_lossy().contains("src/utils/.synapse/utils_rules.md"));
    }

    #[test]
    fn test_find_rule_files_ignores_other_files() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create various files that should be ignored
        project.add_file("README.md", "# README").unwrap();
        project.add_file("rules.md", "# Not a rule file").unwrap();
        project.add_rule_file(".synapse/actual_rule.md", &create_rule_content(&[("FORBIDDEN", "TODO")])).unwrap();
        
        let result = discovery.find_rule_files(project.root()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name().unwrap(), "actual_rule.md");
    }

    #[test]
    fn test_is_rule_file() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        let rule_file = project.add_rule_file(".synapse/test_rule.md", &create_rule_content(&[("FORBIDDEN", "TODO")])).unwrap();
        let other_file = project.add_file("other.md", "# Other file").unwrap();
        
        assert!(discovery.is_rule_file(&rule_file));
        assert!(!discovery.is_rule_file(&other_file));
        assert!(!discovery.is_rule_file(project.root())); // Directory
    }

    #[test]
    fn test_find_parent_rule_files() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create nested structure with rule file at root
        project.add_file("src/main.rs", "// main.rs").unwrap();
        let root_rule_file = project.add_rule_file(".synapse/root_rules.md", &create_rule_content(&[("FORBIDDEN", "TODO")])).unwrap();
        
        let file_path = project.path("src/main.rs");
        let result = discovery.find_parent_rule_files(&file_path);
        assert!(!result.is_empty());
        assert_eq!(result[0], root_rule_file);
    }

    #[test]
    fn test_find_parent_rule_files_none() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        project.add_file("main.rs", "// main.rs").unwrap();
        
        let file_path = project.path("main.rs");
        let result = discovery.find_parent_rule_files(&file_path);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_inheritance_chain() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create nested structure: root -> src -> utils -> deep
        project.create_nested_synapse(&["src", "src/utils"]).unwrap();
        project.add_file("src/utils/deep/file.rs", "// deep file").unwrap();
        
        // Create rule files at root and src levels (skip utils level)
        project.add_rule_file(".synapse/root_rules.md", &create_rule_content(&[("FORBIDDEN", "TODO")])).unwrap();
        project.add_rule_file("src/.synapse/src_rules.md", &create_rule_content(&[("REQUIRED", "#[test]")])).unwrap();
        
        let target_file = project.path("src/utils/deep/file.rs");
        let chain = discovery.find_inheritance_chain(&target_file);
        
        // Should find 2 rule files in the chain (closest first: src, then root)
        assert_eq!(chain.len(), 2);
        assert!(chain[0].to_string_lossy().contains("src/.synapse/src_rules.md"));
        assert!(chain[1].to_string_lossy().contains(".synapse/root_rules.md"));
        assert!(!chain[1].to_string_lossy().contains("src/"));
    }

    #[test]
    fn test_find_inheritance_chain_empty() {
        let project = TestProject::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        project.add_file("main.rs", "// main.rs").unwrap();
        
        let file_path = project.path("main.rs");
        let chain = discovery.find_inheritance_chain(&file_path);
        assert_eq!(chain.len(), 0);
    }
}