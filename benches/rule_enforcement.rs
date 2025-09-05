use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use synapse_mcp::{RuleGraph, check_rules, CompiledRule, Rule, RuleType};

/// Generate test files with various content patterns
fn generate_test_files(temp_dir: &TempDir, count: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    // Create different types of test files
    for i in 0..count {
        let file_path = temp_dir.path().join(format!("test_file_{}.rs", i));
        
        let content = match i % 4 {
            0 => {
                // Clean file - no violations
                format!(r#"
// File {}
fn main() {{
    let result = calculate_value();
    println!("Result: {{}}", result);
}}

fn calculate_value() -> i32 {{
    42
}}
"#, i)
            }
            1 => {
                // File with TODO comment (violation)
                format!(r#"
// File {}
fn main() {{
    // TODO: Implement this properly
    let result = calculate_value();
    println!("Result: {{}}", result);
}}

fn calculate_value() -> i32 {{
    todo!() // Another TODO
}}
"#, i)
            }
            2 => {
                // File with println! usage (violation)
                format!(r#"
// File {}
fn main() {{
    println!("Direct println usage");
    let result = calculate_value();
    println!("Result: {{}}", result);
}}

fn calculate_value() -> i32 {{
    println!("Debug print");
    42
}}
"#, i)
            }
            3 => {
                // File with multiple violations
                format!(r#"
// File {}
fn main() {{
    // TODO: Fix this
    println!("Direct println usage");
    let result = calculate_value();
    println!("Result: {{}}", result);
}}

fn calculate_value() -> i32 {{
    // TODO: Optimize this
    println!("Debug print");
    42
}}
"#, i)
            }
            _ => unreachable!(),
        };
        
        fs::write(&file_path, content).expect("Failed to write test file");
        files.push(file_path);
    }
    
    files
}

/// Generate a set of compiled rules for testing
fn generate_test_rules() -> Vec<CompiledRule> {
    let rules = vec![
        Rule {
            id: "forbidden-todo".to_string(),
            name: "No TODO comments".to_string(),
            rule_type: RuleType::Forbidden,
            pattern: "TODO".to_string(),
            message: "TODO comments should be converted to proper issue tracking".to_string(),
            tags: vec!["code-quality".to_string()],
            metadata: std::collections::HashMap::new(),
        },
        Rule {
            id: "forbidden-println".to_string(),
            name: "No direct println!".to_string(),
            rule_type: RuleType::Forbidden,
            pattern: "println!".to_string(),
            message: "Use logging instead of direct println! calls".to_string(),
            tags: vec!["logging".to_string()],
            metadata: std::collections::HashMap::new(),
        },
        Rule {
            id: "required-tests".to_string(),
            name: "Tests required".to_string(),
            rule_type: RuleType::Required,
            pattern: "#\\[test\\]".to_string(),
            message: "All modules must have tests".to_string(),
            tags: vec!["testing".to_string()],
            metadata: std::collections::HashMap::new(),
        },
    ];
    
    rules.into_iter()
        .map(|rule| CompiledRule::from_rule(rule))
        .collect()
}

/// Benchmark rule checking performance with different file counts
fn bench_rule_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_enforcement");
    
    // Test with different numbers of files
    for file_count in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("check_rules", file_count),
            file_count,
            |b, &file_count| {
                // Setup
                let temp_dir = TempDir::new().expect("Failed to create temp dir");
                let files = generate_test_files(&temp_dir, file_count);
                let rules = generate_test_rules();
                
                b.iter(|| {
                    let mut total_violations = 0;
                    
                    for file_path in &files {
                        let content = fs::read_to_string(file_path)
                            .expect("Failed to read file");
                        
                        let violations = check_rules(
                            black_box(file_path),
                            black_box(&content),
                            black_box(&rules)
                        ).expect("Rule checking failed");
                        
                        total_violations += violations.len();
                    }
                    
                    black_box(total_violations);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark RuleGraph performance
fn bench_rule_graph_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_graph");
    
    // Create a temporary project with nested rule files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = PathBuf::from(temp_dir.path());
    
    // Create root rule file
    let root_rule = r#"---
mcp: synapse
type: rule
---

# Root Rules

FORBIDDEN: `TODO` - TODO comments should be converted to proper issue tracking
FORBIDDEN: `println!` - Use logging instead of direct println! calls
"#;
    
    fs::write(project_root.join(".synapse.md"), root_rule).expect("Failed to write root rule");
    
    // Create nested directories with rules
    for i in 0..10 {
        let dir = project_root.join(format!("module_{}", i));
        fs::create_dir(&dir).expect("Failed to create directory");
        
        let nested_rule = format!(r#"---
mcp: synapse
type: rule
---

# Module {} Rules

REQUIRED: `#[test]` - All modules must have tests
FORBIDDEN: `unwrap()` - Use proper error handling instead of unwrap
"#, i);
        
        fs::write(dir.join(".synapse.md"), nested_rule).expect("Failed to write nested rule");
    }
    
    group.bench_function("from_project", |b| {
        b.iter(|| {
            let rule_graph = RuleGraph::from_project(black_box(&project_root))
                .expect("Failed to create RuleGraph");
            black_box(rule_graph);
        });
    });
    
    // Benchmark rules_for operation
    let rule_graph = RuleGraph::from_project(&project_root)
        .expect("Failed to create RuleGraph");
    
    let test_files: Vec<PathBuf> = (0..10)
        .map(|i| project_root.join(format!("module_{}/test.rs", i)))
        .collect();
    
    group.bench_function("rules_for", |b| {
        b.iter(|| {
            for file_path in &test_files {
                let rules = rule_graph.rules_for(black_box(file_path))
                    .expect("Failed to get rules");
                black_box(rules);
            }
        });
    });
    
    group.finish();
}

/// Main benchmark for 100 files (as requested)
fn bench_rule_checking_100_files(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let files = generate_test_files(&temp_dir, 100);
    let rules = generate_test_rules();
    
    c.bench_function("bench_rule_checking_100_files", |b| {
        b.iter(|| {
            let mut total_violations = 0;
            
            for file_path in &files {
                let content = fs::read_to_string(file_path)
                    .expect("Failed to read file");
                
                let violations = check_rules(
                    black_box(file_path),
                    black_box(&content),
                    black_box(&rules)
                ).expect("Rule checking failed");
                
                total_violations += violations.len();
            }
            
            black_box(total_violations);
        });
    });
}

/// Benchmark pattern matching performance
fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");
    
    // Test different pattern types
    let test_content = "
        fn main() {
            // TODO: Implement this
            println!(\"Hello world\");
            let result = some_function().unwrap();
            #[test]
            fn test_something() {
                assert_eq!(1, 1);
            }
        }
    ".to_string();
    
    let literal_rule = CompiledRule::from_rule(Rule {
        id: "literal".to_string(),
        name: "Literal Pattern".to_string(),
        rule_type: RuleType::Forbidden,
        pattern: "TODO".to_string(),
        message: "No TODOs".to_string(),
        tags: vec![],
        metadata: std::collections::HashMap::new(),
    });
    
    let regex_rule = CompiledRule::from_rule(Rule {
        id: "regex".to_string(),
        name: "Regex Pattern".to_string(),
        rule_type: RuleType::Forbidden,
        pattern: r#"println!\s*\("#.to_string(),
        message: "No println!".to_string(),
        tags: vec![],
        metadata: std::collections::HashMap::new(),
    });
    
    group.bench_function("literal_pattern", |b| {
        b.iter(|| {
            let violations = check_rules(
                black_box(&PathBuf::from("test.rs")),
                black_box(&test_content),
                black_box(&[literal_rule.clone()])
            ).expect("Rule checking failed");
            black_box(violations);
        });
    });
    
    group.bench_function("regex_pattern", |b| {
        b.iter(|| {
            let violations = check_rules(
                black_box(&PathBuf::from("test.rs")),
                black_box(&test_content),
                black_box(&[regex_rule.clone()])
            ).expect("Rule checking failed");
            black_box(violations);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_rule_checking,
    bench_rule_graph_operations,
    bench_rule_checking_100_files,
    bench_pattern_matching
);
criterion_main!(benches);