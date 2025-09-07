//! AST Analysis Module for Safe Code Transformations
//! 
//! This module provides AST-based analysis for safe code transformations,
//! following SOLID principles with separate concerns for parsing, visiting,
//! and transforming code.

#[cfg(feature = "ast-fixes")]
pub mod safe_unwrap_replacer;

#[cfg(feature = "ast-fixes")]
pub use safe_unwrap_replacer::{UnwrapReplacer, Replacement, safely_replace_unwrap};

/// Error types for AST analysis operations
#[derive(Debug, thiserror::Error)]
pub enum AstAnalysisError {
    #[error("Failed to parse Rust syntax: {0}")]
    ParseError(String),
    
    #[error("Unsafe replacement detected: {0}")]
    UnsafeReplacement(String),
    
    #[error("AST feature not enabled. Enable 'ast-fixes' feature to use AST-based fixes")]
    FeatureNotEnabled,
}

/// Result type for AST analysis operations
pub type AstResult<T> = Result<T, AstAnalysisError>;

/// Check if AST fixes are available (feature flag enabled)
pub fn ast_fixes_available() -> bool {
    cfg!(feature = "ast-fixes")
}