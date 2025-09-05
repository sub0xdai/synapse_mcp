use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::{RuleType, Violation};

/// Generic API request wrapper that can contain any payload type
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiRequest<T> {
    pub data: T,
    pub metadata: Option<RequestMetadata>,
}

/// Generic API response wrapper that provides consistent structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: Option<ResponseMetadata>,
}

/// Optional metadata that can be included with requests
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestMetadata {
    pub request_id: Option<String>,
    pub timestamp: Option<String>,
    pub client_version: Option<String>,
}

/// Optional metadata included in responses
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseMetadata {
    pub request_id: Option<String>,
    pub processing_time_ms: Option<u64>,
    pub timestamp: Option<String>,
}

/// Data payload for checking files against rules
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckData {
    pub files: Vec<PathBuf>,
    pub dry_run: Option<bool>,
}

/// Data payload returned from rule checking
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckResultData {
    pub violations: Vec<RuleViolationDto>,
    pub files_checked: usize,
    pub rules_applied: usize,
}

/// Data payload for requesting rule context
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextData {
    pub path: PathBuf,
    pub format: Option<String>,
}

/// Data payload for rule context information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextResultData {
    pub context: Option<String>,
    pub applicable_rules: Vec<RuleContextInfo>,
    pub inheritance_chain: Vec<PathBuf>,
    pub overridden_rules: Vec<String>,
}

/// Data payload for requesting rules for a specific path
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesForPathData {
    pub path: PathBuf,
}

/// Data payload for rules applicable to a path
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesForPathResultData {
    pub path: PathBuf,
    pub rules: Vec<RuleContextInfo>,
    pub inheritance_chain: Vec<PathBuf>,
    pub overridden_rules: Vec<String>,
}

/// DTO for rule violations (for serialization)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleViolationDto {
    pub file_path: PathBuf,
    pub rule_name: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub message: String,
    pub line_number: Option<usize>,
    pub line_content: Option<String>,
}

impl From<&Violation> for RuleViolationDto {
    fn from(violation: &Violation) -> Self {
        Self {
            file_path: violation.file_path.clone(),
            rule_name: violation.rule.name.clone(),
            rule_type: violation.rule.rule_type.clone(),
            pattern: violation.rule.pattern.clone(),
            message: violation.rule.message.clone(),
            line_number: violation.line_number,
            line_content: violation.line_content.clone(),
        }
    }
}

/// Rule information formatted for AI consumption
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleContextInfo {
    pub name: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub message: String,
    pub tags: Vec<String>,
    pub enforcement_level: String,
}

/// Type aliases for cleaner code
pub type CheckRequest = ApiRequest<CheckData>;
pub type CheckResponse = ApiResponse<CheckResultData>;

pub type ContextRequest = ApiRequest<ContextData>;
pub type ContextResponse = ApiResponse<ContextResultData>;

pub type RulesForPathRequest = ApiRequest<RulesForPathData>;
pub type RulesForPathResponse = ApiResponse<RulesForPathResultData>;

impl<T> ApiRequest<T> {
    /// Create a simple request with just data
    pub fn new(data: T) -> Self {
        Self {
            data,
            metadata: None,
        }
    }

    /// Create a request with metadata
    pub fn with_metadata(data: T, metadata: RequestMetadata) -> Self {
        Self {
            data,
            metadata: Some(metadata),
        }
    }
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: None,
        }
    }

    /// Create an error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata: None,
        }
    }

    /// Create a successful response with metadata
    pub fn success_with_metadata(data: T, metadata: ResponseMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: Some(metadata),
        }
    }
}

impl RuleContextInfo {
    pub fn rule_type_display(&self) -> &str {
        match self.rule_type {
            RuleType::Forbidden => "FORBIDDEN",
            RuleType::Required => "REQUIRED", 
            RuleType::Standard => "STANDARD",
            RuleType::Convention => "CONVENTION",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_request_creation() {
        let data = CheckData {
            files: vec![PathBuf::from("test.rs")],
            dry_run: Some(true),
        };
        
        let request = ApiRequest::new(data.clone());
        assert_eq!(request.data.files, vec![PathBuf::from("test.rs")]);
        assert_eq!(request.data.dry_run, Some(true));
        assert!(request.metadata.is_none());
    }

    #[test]
    fn test_api_response_success() {
        let data = CheckResultData {
            violations: vec![],
            files_checked: 1,
            rules_applied: 0,
        };
        
        let response = ApiResponse::success(data);
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<CheckResultData> = ApiResponse::error("Test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_type_aliases() {
        let data = CheckData {
            files: vec![],
            dry_run: None,
        };
        
        let _request: CheckRequest = ApiRequest::new(data);
        // Just testing compilation works
    }

    #[test]
    fn test_rule_context_info_display() {
        let rule_info = RuleContextInfo {
            name: "test-rule".to_string(),
            rule_type: RuleType::Forbidden,
            pattern: "test".to_string(),
            message: "Test message".to_string(),
            tags: vec![],
            enforcement_level: "BLOCKING".to_string(),
        };
        
        assert_eq!(rule_info.rule_type_display(), "FORBIDDEN");
    }
}