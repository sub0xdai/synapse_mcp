//! Test helpers for creating hermetic filesystem test environments
//! 
//! This module provides utilities for creating isolated temporary directories
//! and project structures for testing Synapse MCP functionality.
//! 
//! # Usage
//! 
//! ```rust
//! use crate::test_helpers::TestProject;
//! 
//! #[test]
//! fn test_rule_loading() {
//!     let project = TestProject::with_synapse_dirs().unwrap();
//!     project.add_rule_file("rules/security.md", create_rule_content(&[
//!         ("FORBIDDEN", "password"),
//!     ])).unwrap();
//!     
//!     // Now test your functionality with project.root()
//! }
//! ```

#[cfg(test)]
pub mod test_helpers {
    use tempfile::TempDir;
    use std::path::{Path, PathBuf};
    use std::fs;
    use crate::{RuleType, Result, SynapseError};

    /// Test project builder for creating hermetic filesystem environments
    /// 
    /// This struct manages temporary directories and provides helpers for creating
    /// the .synapse directory structures needed for testing.
    pub struct TestProject {
        temp_dir: TempDir,
        project_root: PathBuf,
    }

    impl TestProject {
        /// Create a new empty test project in a temporary directory
        /// 
        /// # Returns
        /// 
        /// A TestProject with an isolated temporary directory
        /// 
        /// # Examples
        /// 
        /// ```rust
        /// let project = TestProject::new()?;
        /// // Work with project.root()
        /// # Ok::<(), synapse_mcp::SynapseError>(())
        /// ```
        pub fn new() -> Result<Self> {
            let temp_dir = TempDir::new()
                .map_err(|e| SynapseError::Internal(format!("Failed to create temp directory: {}", e)))?;
            let project_root = temp_dir.path().to_path_buf();
            
            Ok(Self {
                temp_dir,
                project_root,
            })
        }

        /// Create a test project with standard .synapse directory structure
        /// 
        /// This creates the following structure:
        /// - `.synapse/` - Root level rules directory
        /// - `.synapse/rules/` - Standard rules subdirectory
        /// - `.synapse/architecture/` - Architecture documentation
        /// - `.synapse/decisions/` - Decision records
        /// 
        /// # Returns
        /// 
        /// A TestProject with pre-created .synapse directories
        pub fn with_synapse_dirs() -> Result<Self> {
            let mut project = Self::new()?;
            project.create_synapse_structure()?;
            Ok(project)
        }

        /// Create standard .synapse directory structure
        fn create_synapse_structure(&mut self) -> Result<()> {
            let synapse_dir = self.project_root.join(".synapse");
            
            // Create main directories
            fs::create_dir_all(&synapse_dir)
                .map_err(|e| SynapseError::Internal(format!("Failed to create .synapse directory: {}", e)))?;
            fs::create_dir_all(synapse_dir.join("rules"))
                .map_err(|e| SynapseError::Internal(format!("Failed to create rules directory: {}", e)))?;
            fs::create_dir_all(synapse_dir.join("architecture"))
                .map_err(|e| SynapseError::Internal(format!("Failed to create architecture directory: {}", e)))?;
            fs::create_dir_all(synapse_dir.join("decisions"))
                .map_err(|e| SynapseError::Internal(format!("Failed to create decisions directory: {}", e)))?;
            
            Ok(())
        }

        /// Add a rule file to the project at the specified path
        /// 
        /// The path should be relative to the project root. For .synapse files,
        /// use paths like "rules/security.md" or ".synapse.md".
        /// 
        /// # Arguments
        /// 
        /// * `relative_path` - Path relative to project root
        /// * `content` - File content to write
        /// 
        /// # Returns
        /// 
        /// The absolute path to the created file
        /// 
        /// # Examples
        /// 
        /// ```rust
        /// let project = TestProject::new()?;
        /// let rule_file = project.add_rule_file(".synapse.md", create_rule_content(&[
        ///     ("FORBIDDEN", "TODO"),
        /// ]))?;
        /// # Ok::<(), synapse_mcp::SynapseError>(())
        /// ```
        pub fn add_rule_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
            let file_path = self.project_root.join(relative_path);
            
            // Create parent directories if they don't exist
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| SynapseError::Internal(format!("Failed to create parent directories: {}", e)))?;
            }
            
            fs::write(&file_path, content)
                .map_err(|e| SynapseError::Internal(format!("Failed to write file {}: {}", file_path.display(), e)))?;
            
            Ok(file_path)
        }

        /// Create nested .synapse directory structure in multiple subdirectories
        /// 
        /// This creates .synapse directories at the specified relative paths.
        /// Useful for testing directory-based rule inheritance.
        /// 
        /// # Arguments
        /// 
        /// * `dirs` - Relative paths where .synapse directories should be created
        /// 
        /// # Examples
        /// 
        /// ```rust
        /// let project = TestProject::new()?;
        /// project.create_nested_synapse(&["src", "tests", "src/utils"])?;
        /// // Creates: src/.synapse/, tests/.synapse/, src/utils/.synapse/
        /// # Ok::<(), synapse_mcp::SynapseError>(())
        /// ```
        pub fn create_nested_synapse(&self, dirs: &[&str]) -> Result<()> {
            for dir_path in dirs {
                let synapse_path = self.project_root.join(dir_path).join(".synapse");
                fs::create_dir_all(&synapse_path)
                    .map_err(|e| SynapseError::Internal(format!("Failed to create nested .synapse directory {}: {}", synapse_path.display(), e)))?;
            }
            Ok(())
        }

        /// Add a regular file (not a rule file) to the project
        /// 
        /// Useful for creating test source files or other project content.
        /// 
        /// # Arguments
        /// 
        /// * `relative_path` - Path relative to project root
        /// * `content` - File content to write
        pub fn add_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
            self.add_rule_file(relative_path, content) // Same implementation
        }

        /// Get the path to the project root directory
        /// 
        /// Use this to pass the project root to functions that expect
        /// a project directory path.
        pub fn root(&self) -> &Path {
            &self.project_root
        }

        /// Get a path relative to the project root
        /// 
        /// # Arguments
        /// 
        /// * `relative` - Relative path from project root
        /// 
        /// # Returns
        /// 
        /// Absolute path to the specified location
        pub fn path(&self, relative: &str) -> PathBuf {
            self.project_root.join(relative)
        }

        /// Get the temporary directory (for advanced usage)
        /// 
        /// Most tests should use `root()` instead.
        pub fn temp_dir(&self) -> &TempDir {
            &self.temp_dir
        }
    }

    /// Create standard rule file content with YAML frontmatter
    /// 
    /// # Arguments
    /// 
    /// * `rules` - Vector of (rule_type, pattern, message) tuples
    /// 
    /// # Returns
    /// 
    /// Complete rule file content with frontmatter and rule definitions
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let content = create_rule_content(&[
    ///     ("FORBIDDEN", "TODO"),
    ///     ("REQUIRED", "#[test]"),
    /// ]);
    /// ```
    pub fn create_rule_content(rules: &[(&str, &str)]) -> String {
        let mut content = String::from(
            "---\n\
             mcp: synapse\n\
             type: rule\n\
             ---\n\n\
             # Test Rules\n\n"
        );

        for (rule_type, pattern) in rules {
            content.push_str(&format!("{}: `{}` - Test rule for {}\n", rule_type, pattern, pattern));
        }

        content
    }

    /// Create .synapse file content with typed rules and detailed frontmatter
    /// 
    /// This is a more advanced version of create_rule_content that allows
    /// specifying RuleType enums and custom messages.
    /// 
    /// # Arguments
    /// 
    /// * `node_type` - Type of node for frontmatter (e.g., "rule", "decision")
    /// * `rules` - Vector of (RuleType, pattern, message) tuples
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use crate::RuleType;
    /// 
    /// let content = create_synapse_file("rule", vec![
    ///     (RuleType::Forbidden, "println!", "Use logging framework"),
    ///     (RuleType::Required, "#[test]", "All functions need tests"),
    /// ]);
    /// ```
    pub fn create_synapse_file(node_type: &str, rules: Vec<(RuleType, &str, &str)>) -> String {
        let mut content = format!(
            "---\n\
             mcp: synapse\n\
             type: {}\n\
             ---\n\n\
             # Generated Test Rules\n\n",
            node_type
        );

        for (rule_type, pattern, message) in rules {
            let rule_type_str = match rule_type {
                RuleType::Forbidden => "FORBIDDEN",
                RuleType::Required => "REQUIRED", 
                RuleType::Standard => "STANDARD",
                RuleType::Convention => "CONVENTION",
            };
            content.push_str(&format!("{}: `{}` - {}\n", rule_type_str, pattern, message));
        }

        content
    }

    /// Create a simple rule file for legacy .synapse.md format
    /// 
    /// For backward compatibility with tests that expect .synapse.md files
    /// in the root or specific directories.
    pub fn create_simple_rule(rule_type: &str, pattern: &str, message: &str) -> String {
        format!(
            "---\n\
             mcp: synapse\n\
             type: rule\n\
             ---\n\n\
             # Simple Test Rule\n\n\
             {}: `{}` - {}\n",
            rule_type, pattern, message
        )
    }

    /// Create project structure for rule inheritance testing
    /// 
    /// Creates a project with nested directories, each containing .synapse
    /// directories with sample rules for testing inheritance chains.
    /// 
    /// Structure created:
    /// - `.synapse/rules/global.md`
    /// - `src/.synapse/coding.md`
    /// - `src/utils/.synapse/utility.md`
    /// - `tests/.synapse/testing.md`
    pub fn create_inheritance_test_project() -> Result<TestProject> {
        let project = TestProject::new()?;
        
        // Create nested .synapse directories
        project.create_nested_synapse(&["src", "src/utils", "tests"])?;
        
        // Add global rules
        project.add_rule_file(".synapse/rules/global.md", 
            &create_rule_content(&[
                ("FORBIDDEN", "TODO"),
                ("REQUIRED", "SPDX-License-Identifier"),
            ]))?;
        
        // Add source-specific rules
        project.add_rule_file("src/.synapse/coding.md",
            &create_rule_content(&[
                ("FORBIDDEN", "unwrap()"),
                ("REQUIRED", "#[derive(Debug)]"),
            ]))?;
        
        // Add utility-specific rules
        project.add_rule_file("src/utils/.synapse/utility.md",
            &create_rule_content(&[
                ("STANDARD", "inline"),
                ("CONVENTION", "short function names"),
            ]))?;
        
        // Add test-specific rules
        project.add_rule_file("tests/.synapse/testing.md",
            &create_rule_content(&[
                ("REQUIRED", "#[test]"),
                ("CONVENTION", "descriptive test names"),
            ]))?;
        
        Ok(project)
    }
}