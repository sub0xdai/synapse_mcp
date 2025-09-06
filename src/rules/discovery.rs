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
    use tempfile::TempDir;
    use std::fs;

    fn create_rule_file(dir: &Path, content: &str) -> PathBuf {
        let rule_file = dir.join(".synapse.md");
        fs::write(&rule_file, content).unwrap();
        rule_file
    }

    #[test]
    fn test_find_rule_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        let result = discovery.find_rule_files(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_rule_files_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        let _rule_file = create_rule_file(temp_dir.path(), "# Test Rule");
        
        let result = discovery.find_rule_files(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name().unwrap(), ".synapse.md");
    }

    #[test]
    fn test_find_rule_files_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create nested directory structure
        let src_dir = temp_dir.path().join("src");
        let utils_dir = src_dir.join("utils");
        fs::create_dir_all(&utils_dir).unwrap();
        
        // Create rule files at different levels
        create_rule_file(temp_dir.path(), "# Root Rules");
        create_rule_file(&src_dir, "# Src Rules");  
        create_rule_file(&utils_dir, "# Utils Rules");
        
        let result = discovery.find_rule_files(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 3);
        
        // Should be sorted by path
        assert!(result[0].to_string_lossy().contains(".synapse.md"));
        assert!(result[1].to_string_lossy().contains("src/.synapse.md"));
        assert!(result[2].to_string_lossy().contains("utils/.synapse.md"));
    }

    #[test]
    fn test_find_rule_files_ignores_other_files() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create various files
        fs::write(temp_dir.path().join("README.md"), "# README").unwrap();
        fs::write(temp_dir.path().join("rules.md"), "# Not a rule file").unwrap();
        create_rule_file(temp_dir.path(), "# Actual rule file");
        
        let result = discovery.find_rule_files(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name().unwrap(), ".synapse.md");
    }

    #[test]
    fn test_is_rule_file() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        let rule_file = create_rule_file(temp_dir.path(), "# Test Rule");
        let other_file = temp_dir.path().join("other.md");
        fs::write(&other_file, "# Other file").unwrap();
        
        assert!(discovery.is_rule_file(&rule_file));
        assert!(!discovery.is_rule_file(&other_file));
        assert!(!discovery.is_rule_file(temp_dir.path())); // Directory
    }

    #[test]
    fn test_find_parent_rule_file() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create nested structure with rule file at root
        let src_dir = temp_dir.path().join("src");
        let file_path = src_dir.join("main.rs");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(&file_path, "// main.rs").unwrap();
        
        let root_rule_file = create_rule_file(temp_dir.path(), "# Root Rules");
        
        let result = discovery.find_parent_rule_file(&file_path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), root_rule_file);
    }

    #[test]
    fn test_find_parent_rule_file_none() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        let file_path = temp_dir.path().join("main.rs");
        fs::write(&file_path, "// main.rs").unwrap();
        
        let result = discovery.find_parent_rule_file(&file_path);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_inheritance_chain() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        // Create nested structure: root -> src -> utils -> deep
        let src_dir = temp_dir.path().join("src");
        let utils_dir = src_dir.join("utils");
        let deep_dir = utils_dir.join("deep");
        fs::create_dir_all(&deep_dir).unwrap();
        
        // Create rule files at root and src levels (skip utils)
        create_rule_file(temp_dir.path(), "# Root Rules");
        create_rule_file(&src_dir, "# Src Rules");
        
        let target_file = deep_dir.join("file.rs");
        fs::write(&target_file, "// deep file").unwrap();
        
        let chain = discovery.find_inheritance_chain(&target_file);
        
        // Should find 2 rule files in the chain (closest first)
        assert_eq!(chain.len(), 2);
        assert!(chain[0].to_string_lossy().contains("src/.synapse.md"));
        assert!(chain[1].to_string_lossy().contains(".synapse.md"));
        assert!(!chain[1].to_string_lossy().contains("src/"));
    }

    #[test]
    fn test_find_inheritance_chain_empty() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = RuleDiscovery::new();
        
        let file_path = temp_dir.path().join("main.rs");
        fs::write(&file_path, "// main.rs").unwrap();
        
        let chain = discovery.find_inheritance_chain(&file_path);
        assert_eq!(chain.len(), 0);
    }
}