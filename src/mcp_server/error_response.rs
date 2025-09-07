use serde::{Deserialize, Serialize};
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use crate::SynapseError;

/// HTTP error response structure for JSON API responses
/// 
/// This provides a consistent error format across all MCP server endpoints,
/// following KISS principle with clear, actionable error information.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Human-readable error message
    pub message: String,
    /// HTTP status code as integer for client parsing
    pub error_code: u16,
    /// Unique request ID for debugging and tracing
    pub request_id: String,
}

impl ErrorResponse {
    /// Create a new error response with generated request ID
    pub fn new(message: String, status_code: StatusCode) -> Self {
        Self {
            message,
            error_code: status_code.as_u16(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    /// Create an error response with custom request ID (for tracing)
    pub fn with_request_id(message: String, status_code: StatusCode, request_id: String) -> Self {
        Self {
            message,
            error_code: status_code.as_u16(),
            request_id,
        }
    }
}

/// Convert SynapseError into HTTP response with appropriate status codes
/// 
/// This implementation follows SOLID principles by mapping domain errors
/// to HTTP semantics in a single, focused location.
impl IntoResponse for SynapseError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            // Authentication and authorization errors
            SynapseError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg),
            
            // Client request errors
            SynapseError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            SynapseError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            SynapseError::Validation(msg) => (StatusCode::BAD_REQUEST, format!("Validation failed: {}", msg)),
            
            // Business logic errors
            SynapseError::RuleViolation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, format!("Rule violation: {}", msg)),
            
            // Configuration and parsing errors (server issues)
            SynapseError::Configuration(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Configuration error: {}", msg)),
            SynapseError::Parse(msg) => (StatusCode::BAD_REQUEST, format!("Parse error: {}", msg)),
            
            // External service errors
            SynapseError::Neo4j(err) => (StatusCode::SERVICE_UNAVAILABLE, format!("Database error: {}", err)),
            
            // File system and I/O errors  
            SynapseError::Io(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("I/O error: {}", err)),
            
            // Serialization errors (usually client data issues)
            SynapseError::Serde(err) => (StatusCode::BAD_REQUEST, format!("Data format error: {}", err)),
            SynapseError::Yaml(err) => (StatusCode::BAD_REQUEST, format!("YAML format error: {}", err)),
            
            // Internal server errors (catch-all)
            SynapseError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", msg)),
        };

        let error_response = ErrorResponse::new(message, status);
        (status, Json(error_response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new("Test error".to_string(), StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Test error");
        assert_eq!(error.error_code, 400);
        assert!(!error.request_id.is_empty());
    }

    #[test]
    fn test_error_response_with_request_id() {
        let request_id = "custom-123".to_string();
        let error = ErrorResponse::with_request_id(
            "Test error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id.clone()
        );
        assert_eq!(error.request_id, request_id);
        assert_eq!(error.error_code, 500);
    }

    #[test]
    fn test_synapse_error_status_mapping() {
        // Test authentication error mapping
        let auth_error = SynapseError::Authentication("Invalid token".to_string());
        let response = auth_error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Test bad request mapping
        let bad_req_error = SynapseError::BadRequest("Invalid input".to_string());
        let response = bad_req_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test not found mapping
        let not_found_error = SynapseError::NotFound("Resource missing".to_string());
        let response = not_found_error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Test rule violation mapping (unprocessable entity for business logic errors)
        let rule_error = SynapseError::RuleViolation("Forbidden pattern found".to_string());
        let response = rule_error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}