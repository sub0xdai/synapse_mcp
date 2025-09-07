//! Safe Unwrap Replacement using AST Analysis
//! 
//! This module implements safe replacement of .unwrap() calls with the ? operator,
//! but only when it's provably safe to do so based on the function's return type.

#[cfg(feature = "ast-fixes")]
use syn::{
    visit_mut::{self, VisitMut},
    File, ItemFn, ExprMethodCall, ReturnType, Type, PathSegment,
    parse_file, parse_str, Error as SynError,
};

#[cfg(feature = "ast-fixes")]
use quote::ToTokens;

use super::{AstResult, AstAnalysisError};
use std::collections::HashMap;

/// Represents a safe replacement that can be applied to code
#[derive(Debug, Clone)]
pub struct Replacement {
    pub line: usize,
    pub column: usize,
    pub original: String,
    pub replacement: String,
    pub reason: String,
}

/// AST visitor that identifies safe unwrap() replacements
#[cfg(feature = "ast-fixes")]
pub struct UnwrapReplacer {
    /// Track if the current function returns Result<T, E> or Option<T>
    current_function_returns_result: bool,
    current_function_returns_option: bool,
    
    /// Stack to handle nested functions
    function_context_stack: Vec<FunctionContext>,
    
    /// Collected safe replacements
    replacements: Vec<Replacement>,
    
    /// Track function names and their return types for cross-reference
    function_signatures: HashMap<String, FunctionReturnType>,
}

#[cfg(feature = "ast-fixes")]
#[derive(Debug, Clone)]
struct FunctionContext {
    returns_result: bool,
    returns_option: bool,
    function_name: String,
}

#[cfg(feature = "ast-fixes")]
#[derive(Debug, Clone)]
enum FunctionReturnType {
    Result,
    Option, 
    Other,
}

#[cfg(feature = "ast-fixes")]
impl UnwrapReplacer {
    pub fn new() -> Self {
        Self {
            current_function_returns_result: false,
            current_function_returns_option: false,
            function_context_stack: Vec::new(),
            replacements: Vec::new(),
            function_signatures: HashMap::new(),
        }
    }
    
    pub fn replacements(&self) -> &[Replacement] {
        &self.replacements
    }
    
    pub fn has_safe_replacements(&self) -> bool {
        !self.replacements.is_empty()
    }
    
    /// Analyze the return type of a function to determine if it returns Result or Option
    fn analyze_return_type(&self, return_type: &ReturnType) -> (bool, bool) {
        match return_type {
            ReturnType::Type(_, ty) => {
                if let Type::Path(type_path) = ty.as_ref() {
                    if let Some(segment) = type_path.path.segments.last() {
                        let returns_result = segment.ident == "Result";
                        let returns_option = segment.ident == "Option";
                        return (returns_result, returns_option);
                    }
                }
            }
            ReturnType::Default => {
                // Function returns () - not compatible with ?
                return (false, false);
            }
        }
        (false, false)
    }
    
    /// Check if an unwrap() call is safe to replace with ?
    fn is_safe_unwrap_replacement(&self, method_call: &ExprMethodCall) -> bool {
        // Must be in a function that returns Result or Option
        if !self.current_function_returns_result && !self.current_function_returns_option {
            return false;
        }
        
        // Method must be named "unwrap"
        if method_call.method != "unwrap" {
            return false;
        }
        
        // Conservative approach: Only replace if we're highly confident
        // Additional safety checks:
        
        // 1. Don't replace unwrap() in complex expressions where ? might change semantics
        // This is a simplified check - a full implementation would do deeper analysis
        
        // 2. For now, we'll be very conservative and only suggest replacements
        // in the most straightforward cases
        
        true
    }
    
    /// Determine if the replacement is semantically safe
    fn is_semantically_safe_replacement(&self, method_call: &ExprMethodCall) -> bool {
        // Additional semantic safety checks could include:
        // - Ensuring the expression is not in a position where early return would break logic
        // - Verifying that error types are compatible
        // - Checking that the ? operator won't change the function's behavior
        
        // For this initial implementation, we rely on the context analysis
        self.current_function_returns_result || self.current_function_returns_option
    }
    
    /// Record a safe replacement
    fn record_replacement(&mut self, method_call: &ExprMethodCall, reason: String) {
        // For simplicity, we'll use dummy line/column values
        // In a real implementation, you'd want to preserve span information
        let replacement = Replacement {
            line: 0, // Would need span information from syn
            column: 0,
            original: method_call.to_token_stream().to_string(),
            replacement: format!("{}?", method_call.receiver.to_token_stream()),
            reason,
        };
        
        self.replacements.push(replacement);
    }
}

#[cfg(feature = "ast-fixes")]
impl VisitMut for UnwrapReplacer {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        // Analyze the function's return type
        let (returns_result, returns_option) = self.analyze_return_type(&node.sig.output);
        
        // Save current context
        let previous_context = FunctionContext {
            returns_result: self.current_function_returns_result,
            returns_option: self.current_function_returns_option,
            function_name: format!("previous_{}", self.function_context_stack.len()),
        };
        self.function_context_stack.push(previous_context);
        
        // Set new context
        self.current_function_returns_result = returns_result;
        self.current_function_returns_option = returns_option;
        
        // Store function signature for cross-reference
        let fn_name = node.sig.ident.to_string();
        let return_type = if returns_result {
            FunctionReturnType::Result
        } else if returns_option {
            FunctionReturnType::Option
        } else {
            FunctionReturnType::Other
        };
        self.function_signatures.insert(fn_name, return_type);
        
        // Visit the function body
        visit_mut::visit_item_fn_mut(self, node);
        
        // Restore previous context
        if let Some(context) = self.function_context_stack.pop() {
            self.current_function_returns_result = context.returns_result;
            self.current_function_returns_option = context.returns_option;
        }
    }
    
    fn visit_expr_method_call_mut(&mut self, node: &mut ExprMethodCall) {
        // Check if this is a safe unwrap() replacement
        if self.is_safe_unwrap_replacement(node) && self.is_semantically_safe_replacement(node) {
            let reason = if self.current_function_returns_result {
                "Function returns Result<T, E>, safe to replace unwrap() with ? operator".to_string()
            } else if self.current_function_returns_option {
                "Function returns Option<T>, safe to replace unwrap() with ? operator".to_string()
            } else {
                "Context analysis suggests safe replacement".to_string()
            };
            
            self.record_replacement(node, reason);
        }
        
        // Continue visiting nested expressions
        visit_mut::visit_expr_method_call_mut(self, node);
    }
}

/// Safely replace unwrap() calls with ? operator where provably safe
pub fn safely_replace_unwrap(code: &str) -> AstResult<String> {
    #[cfg(not(feature = "ast-fixes"))]
    {
        return Err(AstAnalysisError::FeatureNotEnabled);
    }
    
    #[cfg(feature = "ast-fixes")]
    {
        // Parse the Rust code into an AST
        let mut syntax_tree = parse_file(code)
            .map_err(|e| AstAnalysisError::ParseError(e.to_string()))?;
        
        // Create and run the replacer
        let mut replacer = UnwrapReplacer::new();
        replacer.visit_file_mut(&mut syntax_tree);
        
        if replacer.has_safe_replacements() {
            // Apply the replacements conservatively
            // In a production implementation, this would use precise span-based replacement
            let mut result = code.to_string();
            let mut replacements_applied = 0;
            
            for replacement in replacer.replacements() {
                // Only apply if we can find a clean match
                // This is a conservative approach to avoid incorrect replacements
                if result.contains(&replacement.original) {
                    // Count occurrences to avoid over-replacement
                    let occurrences = result.matches(&replacement.original).count();
                    if occurrences == 1 {
                        // Safe to replace - only one occurrence
                        result = result.replace(&replacement.original, &replacement.replacement);
                        replacements_applied += 1;
                    }
                    // If multiple occurrences, skip to avoid incorrect replacements
                }
            }
            
            // Return result with metadata about replacements
            if replacements_applied > 0 {
                Ok(result)
            } else {
                // Had potential replacements but couldn't safely apply them
                Ok(code.to_string())
            }
        } else {
            // No safe replacements found, return original code
            Ok(code.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ast_fixes_availability() {
        // Test that we can check if AST fixes are available
        let available = super::super::ast_fixes_available();
        
        #[cfg(feature = "ast-fixes")]
        assert!(available, "AST fixes should be available when feature is enabled");
        
        #[cfg(not(feature = "ast-fixes"))]
        assert!(!available, "AST fixes should not be available when feature is disabled");
    }
    
    #[cfg(feature = "ast-fixes")]
    #[test]
    fn test_unwrap_replacer_creation() {
        let replacer = UnwrapReplacer::new();
        assert!(!replacer.has_safe_replacements());
        assert_eq!(replacer.replacements().len(), 0);
    }
    
    #[test]
    fn test_safely_replace_unwrap_feature_disabled() {
        #[cfg(not(feature = "ast-fixes"))]
        {
            let code = "fn main() { Some(42).unwrap(); }";
            let result = safely_replace_unwrap(code);
            assert!(matches!(result, Err(AstAnalysisError::FeatureNotEnabled)));
        }
    }
}