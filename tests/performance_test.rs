use std::path::PathBuf;
use synapse_mcp::{PreWriteRequest, PreWriteData, RuleGraph, PatternEnforcer};
use tempfile::TempDir;
use std::fs;
use std::time::Instant;

/// Performance test to ensure AST analysis doesn't significantly slow down validation
#[tokio::test]
async fn test_performance_with_and_without_ast() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    // Create comprehensive rules including unwrap patterns
    let rule_content = r#"---
mcp: synapse
type: rule
---

# Performance Test Rules
FORBIDDEN: `TODO` - Convert to issues
FORBIDDEN: `console.log` - Use proper logging  
FORBIDDEN: `unwrap()` - Use proper error handling
FORBIDDEN: `panic!` - Use Result returns
REQUIRED: `#[test]` - Functions should have tests
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("perf_rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    // Create a moderately complex Rust file with various patterns
    let complex_rust_code = r#"
use std::collections::HashMap;

// TODO: Refactor this entire module
pub struct DataProcessor {
    cache: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    pub fn process_data(&self, input: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
        // This unwrap could potentially be replaced with ?
        let data = input.unwrap();
        
        if data.is_empty() {
            panic!("Empty data received!"); // This should never be auto-fixed
        }
        
        console.log("Processing data: {}", data); // JavaScript-style logging
        
        let result = self.transform_data(&data)?;
        Ok(result)
    }
    
    fn transform_data(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        // TODO: Add caching logic
        let processed = data.to_uppercase();
        Ok(processed)
    }
    
    pub fn get_cached(&self, key: &str) -> Option<String> {
        self.cache.get(key).cloned()
    }
    
    // Function without test - should trigger REQUIRED violation
    pub fn cleanup(&mut self) {
        self.cache.clear();
    }
}

// TODO: Add integration tests
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = DataProcessor::new();
    let result = processor.process_data(Some("test data".to_string()))?;
    println!("Result: {}", result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_processor_creation() {
        let processor = DataProcessor::new();
        assert!(processor.cache.is_empty());
    }
    
    #[test] 
    fn test_process_data_success() {
        let processor = DataProcessor::new();
        let result = processor.process_data(Some("hello".to_string()));
        assert!(result.is_ok());
    }
}
"#;

    // Perform multiple validation runs and measure performance
    let num_iterations = 10;
    let mut total_duration = std::time::Duration::new(0, 0);
    
    println!("Running {} iterations of pre-write validation...", num_iterations);
    
    for i in 0..num_iterations {
        let start = Instant::now();
        
        let request = PreWriteRequest::new(PreWriteData {
            file_path: project_root.join("src/processor.rs"),
            content: complex_rust_code.to_string(),
        });

        let response = enforcer.validate_pre_write(request)
            .expect("Pre-write validation should not fail");
            
        let duration = start.elapsed();
        total_duration += duration;
        
        // Verify the validation found expected violations
        let data = response.data.expect("Response should have data");
        assert!(!data.valid, "Should detect violations in iteration {}", i);
        
        // Should find TODO, console.log, unwrap, panic, and missing test violations
        assert!(data.violations.len() >= 4, "Should find multiple violations in iteration {}", i);
        
        println!("Iteration {}: {:?}", i + 1, duration);
    }
    
    let average_duration = total_duration / num_iterations as u32;
    println!("Average validation time: {:?}", average_duration);
    
    // Performance assertion - should complete within reasonable time
    // Even with AST analysis, each validation should be well under 1 second
    assert!(average_duration.as_millis() < 1000, 
           "Average validation time should be under 1 second, got {:?}", average_duration);
    
    // For pre-write validation, we want sub-500ms performance
    if average_duration.as_millis() > 500 {
        println!("WARNING: Average validation time ({:?}) exceeds 500ms target", average_duration);
    }
    
    // Test that AST feature availability doesn't affect basic performance
    let feature_available = synapse_mcp::ast_fixes_available();
    println!("AST fixes feature available: {}", feature_available);
    
    // Regardless of feature availability, performance should be acceptable
    assert!(average_duration.as_millis() < 2000, 
           "Validation should be fast regardless of feature flag");
}

/// Test performance with large files
#[tokio::test]
async fn test_large_file_performance() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();

    let rule_content = r#"---
mcp: synapse
type: rule
---

FORBIDDEN: `unwrap()` - Use proper error handling
FORBIDDEN: `TODO` - Convert to issues
"#;

    let synapse_dir = project_root.join(".synapse");
    fs::create_dir(&synapse_dir).expect("Failed to create .synapse dir");
    let rule_file = synapse_dir.join("large_file_rules.md");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    let rule_graph = RuleGraph::from_project(&PathBuf::from(project_root))
        .expect("Failed to create rule graph");
    let enforcer = PatternEnforcer::new(rule_graph);

    // Generate a large file with repeated patterns
    let mut large_content = String::new();
    large_content.push_str("use std::collections::HashMap;\n\n");
    
    // Add many functions with potential violations
    for i in 0..100 {
        large_content.push_str(&format!(r#"
// TODO: Document function_{}
fn function_{}(input: Option<i32>) -> Result<i32, String> {{
    let value = input.unwrap(); // Potential AST fix target
    if value < 0 {{
        return Err("Negative value".to_string());
    }}
    Ok(value * 2)
}}
"#, i, i));
    }
    
    println!("Generated large file with {} characters", large_content.len());
    
    let start = Instant::now();
    
    let request = PreWriteRequest::new(PreWriteData {
        file_path: project_root.join("src/large_file.rs"),
        content: large_content,
    });

    let response = enforcer.validate_pre_write(request)
        .expect("Pre-write validation should not fail");
        
    let duration = start.elapsed();
    println!("Large file validation time: {:?}", duration);
    
    let data = response.data.expect("Response should have data");
    assert!(!data.valid, "Should detect violations in large file");
    
    // Should find 200 violations (2 per function: TODO and unwrap)
    assert!(data.violations.len() >= 100, "Should find many violations");
    
    // Even large files should validate reasonably quickly
    assert!(duration.as_millis() < 5000, 
           "Large file validation should complete within 5 seconds, got {:?}", duration);
    
    println!("Found {} violations in large file", data.violations.len());
}