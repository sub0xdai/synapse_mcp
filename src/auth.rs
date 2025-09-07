use axum::{
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    extract::Request,
};
use subtle::ConstantTimeEq;

/// Authentication middleware for protecting sensitive MCP endpoints
/// 
/// This middleware extracts Bearer tokens from the Authorization header
/// and performs constant-time comparison to prevent timing attacks.
/// 
/// # Security Features
/// 
/// * Constant-time token comparison using `subtle::ConstantTimeEq`
/// * Secure header parsing with proper validation
/// * No token leakage in error messages or logs
/// 
/// # Usage
/// 
/// ```rust
/// use synapse_mcp::auth::AuthMiddleware;
/// use axum::{Router, middleware};
/// 
/// let auth = AuthMiddleware::new(Some("secret_token".to_string()));
/// let protected_router = Router::new()
///     .layer(middleware::from_fn(move |req, next| auth.call(req, next)));
/// ```
#[derive(Clone)]
pub struct AuthMiddleware {
    required_token: Option<Vec<u8>>,
}

impl AuthMiddleware {
    /// Create a new authentication middleware
    /// 
    /// # Arguments
    /// 
    /// * `token` - Optional bearer token. If None, all requests are allowed.
    ///            If Some, requests must include matching Authorization header.
    pub fn new(token: Option<String>) -> Self {
        Self {
            required_token: token.map(|t| t.into_bytes()),
        }
    }

    /// Process an incoming request with authentication check
    /// 
    /// # Authentication Flow
    /// 
    /// 1. If no token is configured, allow all requests
    /// 2. Extract Authorization header from request
    /// 3. Parse Bearer token from header
    /// 4. Perform constant-time comparison with configured token
    /// 5. Return 401 Unauthorized for invalid/missing tokens
    /// 
    /// # Security Notes
    /// 
    /// * Uses constant-time comparison to prevent timing attacks
    /// * No token information is leaked in error responses
    /// * Headers are parsed securely with proper validation
    pub async fn call(&self, request: Request, next: Next) -> Response {
        // If no token is required, allow all requests
        let required_token = match &self.required_token {
            Some(token) => token,
            None => return next.run(request).await,
        };

        // Extract Authorization header
        let auth_header = match request.headers().get("authorization") {
            Some(header) => header,
            None => return unauthorized_response(),
        };

        // Parse bearer token
        let provided_token = match extract_bearer_token_from_header(auth_header) {
            Some(token) => token,
            None => return unauthorized_response(),
        };

        // Perform constant-time comparison
        let provided_bytes = provided_token.as_bytes();
        
        // Ensure both tokens are the same length for constant-time comparison
        if provided_bytes.len() != required_token.len() {
            return unauthorized_response();
        }

        // Use constant-time comparison to prevent timing attacks
        let tokens_match = provided_bytes.ct_eq(required_token).into();
        
        if tokens_match {
            next.run(request).await
        } else {
            unauthorized_response()
        }
    }
}

/// Extract bearer token from Authorization header value
/// 
/// Parses headers in the format: `Authorization: Bearer <token>`
/// 
/// # Arguments
/// 
/// * `header_value` - The Authorization header value
/// 
/// # Returns
/// 
/// * `Some(token)` if valid Bearer token found
/// * `None` if header is malformed or not Bearer type
/// 
/// # Examples
/// 
/// ```rust
/// use axum::http::HeaderValue;
/// use synapse_mcp::auth::extract_bearer_token_from_header;
/// 
/// let header = HeaderValue::from_static("Bearer my_token_123");
/// assert_eq!(extract_bearer_token_from_header(&header), Some("my_token_123"));
/// 
/// let invalid = HeaderValue::from_static("Basic username:password");
/// assert_eq!(extract_bearer_token_from_header(&invalid), None);
/// ```
fn extract_bearer_token_from_header(header_value: &axum::http::HeaderValue) -> Option<&str> {
    let header_str = header_value.to_str().ok()?;
    
    // Check if it starts with "Bearer "
    if !header_str.starts_with("Bearer ") {
        return None;
    }
    
    // Extract token part (everything after "Bearer ")
    let token = header_str.strip_prefix("Bearer ")?;
    
    // Token must not be empty
    if token.is_empty() {
        return None;
    }
    
    Some(token)
}

/// Helper function to extract bearer token from HeaderMap
/// 
/// This is a convenience function for extracting tokens from Axum's HeaderMap.
/// It wraps the lower-level header value extraction.
/// 
/// # Arguments
/// 
/// * `headers` - Request headers
/// 
/// # Returns
/// 
/// * `Some(token)` if valid Bearer token found
/// * `None` if header missing, malformed, or not Bearer type
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let auth_header = headers.get("authorization")?;
    extract_bearer_token_from_header(auth_header)
}

/// Create a standardized 401 Unauthorized response
/// 
/// Returns a minimal response without leaking authentication details.
/// The response includes the WWW-Authenticate header as per RFC 7235.
fn unauthorized_response() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Bearer")
        .body(axum::body::Body::from("Unauthorized"))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::time::Instant;

    #[test]
    fn test_extract_bearer_token_from_header_valid() {
        let header = HeaderValue::from_static("Bearer valid_token_123");
        let result = extract_bearer_token_from_header(&header);
        assert_eq!(result, Some("valid_token_123"));
    }

    #[test]
    fn test_extract_bearer_token_from_header_invalid_scheme() {
        let header = HeaderValue::from_static("Basic username:password");
        let result = extract_bearer_token_from_header(&header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_bearer_token_from_header_empty_token() {
        let header = HeaderValue::from_static("Bearer");
        let result = extract_bearer_token_from_header(&header);
        assert_eq!(result, None);
        
        let header2 = HeaderValue::from_static("Bearer ");
        let result2 = extract_bearer_token_from_header(&header2);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_extract_bearer_token_from_header_case_sensitive() {
        // Should be case-sensitive - "bearer" (lowercase) should not work
        let header = HeaderValue::from_static("bearer token123");
        let result = extract_bearer_token_from_header(&header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_bearer_token_from_headermap() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer test_token"));
        
        let result = extract_bearer_token(&headers);
        assert_eq!(result, Some("test_token"));
    }

    #[test]
    fn test_auth_middleware_no_token_required() {
        let middleware = AuthMiddleware::new(None);
        assert!(middleware.required_token.is_none());
    }

    #[test]
    fn test_auth_middleware_with_token() {
        let middleware = AuthMiddleware::new(Some("test_secret".to_string()));
        assert!(middleware.required_token.is_some());
        assert_eq!(
            middleware.required_token.as_ref().unwrap(),
            &b"test_secret".to_vec()
        );
    }

    #[test]
    fn test_constant_time_comparison_different_lengths() {
        let middleware = AuthMiddleware::new(Some("short".to_string()));
        let required = middleware.required_token.as_ref().unwrap();
        let provided = "very_long_token_that_is_different".as_bytes();
        
        // Different lengths should fail without timing leak
        let start = Instant::now();
        let result: bool = provided.ct_eq(required).into();
        let duration = start.elapsed();
        
        assert!(!result);
        
        // Should be very fast since we check length first
        assert!(duration.as_nanos() < 1_000_000); // Less than 1ms
    }

    #[test]
    fn test_unauthorized_response() {
        let response = unauthorized_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        
        let www_auth = response.headers().get("WWW-Authenticate");
        assert_eq!(www_auth.map(|h| h.to_str().unwrap()), Some("Bearer"));
    }
}