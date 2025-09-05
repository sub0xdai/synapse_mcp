use crate::models::{CompiledRule, Violation, RuleType, PatternMatcher};
use std::path::Path;

/// Central rule checking function
/// 
/// This is the single source of truth for rule enforcement logic.
/// All CLI and server implementations should use this function.
pub fn check_rules(
    file_path: &Path,
    content: &str, 
    rules: &[CompiledRule]
) -> crate::Result<Vec<Violation>> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    for compiled_rule in rules {
        let rule = &compiled_rule.rule;
        
        match rule.rule_type {
            RuleType::Forbidden => {
                // Check if forbidden pattern exists
                let found_violations = check_forbidden_pattern(
                    file_path,
                    &lines,
                    compiled_rule,
                )?;
                violations.extend(found_violations);
            }
            RuleType::Required => {
                // Check if required pattern is missing
                if let Some(violation) = check_required_pattern(
                    file_path,
                    content,
                    compiled_rule,
                )? {
                    violations.push(violation);
                }
            }
            // Standard and Convention rules are suggestions, not enforced
            RuleType::Standard | RuleType::Convention => {
                // These could be implemented as warnings in the future
                continue;
            }
        }
    }
    
    Ok(violations)
}

/// Check for forbidden pattern violations
fn check_forbidden_pattern(
    file_path: &Path,
    lines: &[&str],
    compiled_rule: &CompiledRule,
) -> crate::Result<Vec<Violation>> {
    let mut violations = Vec::new();
    
    match &compiled_rule.matcher {
        PatternMatcher::Regex(regex) => {
            for (line_num, line) in lines.iter().enumerate() {
                if regex.is_match(line) {
                    violations.push(Violation::from_compiled_rule(
                        file_path.to_path_buf(),
                        compiled_rule,
                        Some(line_num + 1),
                        Some(line.to_string()),
                    ));
                }
            }
        }
        PatternMatcher::Literal(pattern) => {
            for (line_num, line) in lines.iter().enumerate() {
                if line.contains(pattern) {
                    violations.push(Violation::from_compiled_rule(
                        file_path.to_path_buf(),
                        compiled_rule,
                        Some(line_num + 1),
                        Some(line.to_string()),
                    ));
                }
            }
        }
    }
    
    Ok(violations)
}

/// Check for required pattern violations
fn check_required_pattern(
    file_path: &Path,
    content: &str,
    compiled_rule: &CompiledRule,
) -> crate::Result<Option<Violation>> {
    let pattern_found = match &compiled_rule.matcher {
        PatternMatcher::Regex(regex) => {
            content.lines().any(|line| regex.is_match(line))
        }
        PatternMatcher::Literal(pattern) => {
            content.contains(pattern)
        }
    };
    
    if pattern_found {
        Ok(None)
    } else {
        Ok(Some(Violation::from_compiled_rule(
            file_path.to_path_buf(),
            compiled_rule,
            None,
            None,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Rule, RuleType, CompiledRule};
    use std::path::PathBuf;

    #[test]
    fn test_check_forbidden_pattern_with_regex() {
        let rule = Rule::new(
            "no-println".to_string(),
            RuleType::Forbidden,
            r"println!\(".to_string(),
            "Use logging instead of println!".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let content = r#"
            fn main() {
                println!("Hello, world!");
                log::info("This is ok");
            }
        "#;
        
        let violations = check_rules(file_path, content, &[compiled_rule]).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule.name, "no-println");
        assert_eq!(violations[0].rule.rule_type, RuleType::Forbidden);
        assert!(violations[0].line_number.is_some());
        assert!(violations[0].line_content.is_some());
    }
    
    #[test]
    fn test_check_forbidden_pattern_with_literal() {
        let rule = Rule::new(
            "no-todo".to_string(),
            RuleType::Forbidden,
            "TODO".to_string(), // This will become a literal pattern
            "Remove TODO comments".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let content = r#"
            fn main() {
                // TODO: implement this
                let x = 42;
            }
        "#;
        
        let violations = check_rules(file_path, content, &[compiled_rule]).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule.name, "no-todo");
    }
    
    #[test]
    fn test_check_required_pattern_missing() {
        let rule = Rule::new(
            "must-have-license".to_string(),
            RuleType::Required,
            "// SPDX-License-Identifier".to_string(),
            "All files must have SPDX license header".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let content_without_license = r#"
            fn main() {
                println!("Hello");
            }
        "#;
        
        let violations = check_rules(file_path, content_without_license, &[compiled_rule]).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule.name, "must-have-license");
        assert_eq!(violations[0].rule.rule_type, RuleType::Required);
        assert!(violations[0].line_number.is_none()); // Required violations don't have line numbers
    }
    
    #[test]
    fn test_check_required_pattern_present() {
        let rule = Rule::new(
            "must-have-license".to_string(),
            RuleType::Required,
            "// SPDX-License-Identifier".to_string(),
            "All files must have SPDX license header".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let content_with_license = r#"
            // SPDX-License-Identifier: MIT
            fn main() {
                println!("Hello");
            }
        "#;
        
        let violations = check_rules(file_path, content_with_license, &[compiled_rule]).unwrap();
        
        assert_eq!(violations.len(), 0);
    }
    
    #[test]
    fn test_check_no_violations_clean_file() {
        let rule = Rule::new(
            "no-unwrap".to_string(),
            RuleType::Forbidden,
            "unwrap()".to_string(),
            "Use proper error handling".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(rule);
        let file_path = Path::new("test.rs");
        
        let clean_content = r#"
            fn main() -> Result<(), Box<dyn std::error::Error>> {
                let value = some_operation()?;
                Ok(())
            }
        "#;
        
        let violations = check_rules(file_path, clean_content, &[compiled_rule]).unwrap();
        
        assert_eq!(violations.len(), 0);
    }
    
    #[test]
    fn test_check_multiple_rules() {
        let forbidden_rule = Rule::new(
            "no-println".to_string(),
            RuleType::Forbidden,
            "println!".to_string(),
            "Use logging".to_string(),
        );
        
        let required_rule = Rule::new(
            "needs-test".to_string(),
            RuleType::Required,
            "#[test]".to_string(),
            "Must have tests".to_string(),
        );
        
        let compiled_rules = vec![
            CompiledRule::from_rule(forbidden_rule),
            CompiledRule::from_rule(required_rule),
        ];
        
        let file_path = Path::new("test.rs");
        let content = r#"
            fn main() {
                println!("No logging here");
            }
            // No test module
        "#;
        
        let violations = check_rules(file_path, content, &compiled_rules).unwrap();
        
        // Should have both forbidden (println!) and missing required (#[test])
        assert_eq!(violations.len(), 2);
        
        let forbidden_count = violations.iter().filter(|v| v.rule.rule_type == RuleType::Forbidden).count();
        let required_count = violations.iter().filter(|v| v.rule.rule_type == RuleType::Required).count();
        
        assert_eq!(forbidden_count, 1);
        assert_eq!(required_count, 1);
    }
    
    #[test]
    fn test_standard_rules_not_enforced() {
        let standard_rule = Rule::new(
            "prefer-iterators".to_string(),
            RuleType::Standard,
            "for.*in.*".to_string(),
            "Consider using iterator methods".to_string(),
        );
        
        let compiled_rule = CompiledRule::from_rule(standard_rule);
        let file_path = Path::new("test.rs");
        
        let content = r#"
            fn main() {
                for i in 0..10 {
                    println!("{}", i);
                }
            }
        "#;
        
        let violations = check_rules(file_path, content, &[compiled_rule]).unwrap();
        
        // Standard rules should not create violations
        assert_eq!(violations.len(), 0);
    }
}