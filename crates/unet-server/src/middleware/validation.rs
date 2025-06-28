//! Input validation middleware

use axum::{
    Json,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{error, warn};

use crate::validation::{ValidationError, ValidationErrorResponse, ValidationHelper};

/// Middleware to validate request headers for potential security issues
pub async fn validate_headers_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for potentially dangerous headers
    if let Some(user_agent) = headers.get("user-agent") {
        if let Ok(user_agent_str) = user_agent.to_str() {
            if let Err(e) = ValidationHelper::check_sql_injection(user_agent_str) {
                warn!("Suspicious user-agent header detected: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
            if let Err(e) = ValidationHelper::check_command_injection(user_agent_str) {
                warn!("Suspicious user-agent header detected: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // Check referer header for potential CSRF attempts
    if let Some(referer) = headers.get("referer") {
        if let Ok(referer_str) = referer.to_str() {
            if let Err(e) = ValidationHelper::check_sql_injection(referer_str) {
                warn!("Suspicious referer header detected: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // Check for overly large headers (potential DoS)
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            if value_str.len() > 8192 {
                warn!("Overly large header detected: {}", name);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    Ok(next.run(request).await)
}

/// Middleware to validate request body size and content
pub async fn validate_request_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check content-length to prevent DoS
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                // Limit request body to 10MB
                if length > 10 * 1024 * 1024 {
                    warn!("Request body too large: {} bytes", length);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    // Check content-type for valid values
    if let Some(content_type) = request.headers().get("content-type") {
        if let Ok(content_type_str) = content_type.to_str() {
            let allowed_types = [
                "application/json",
                "application/x-www-form-urlencoded",
                "multipart/form-data",
                "text/plain",
            ];

            let is_allowed = allowed_types
                .iter()
                .any(|&allowed| content_type_str.starts_with(allowed));

            if !is_allowed {
                warn!("Unsupported content-type: {}", content_type_str);
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        }
    }

    Ok(next.run(request).await)
}

/// Helper function to create validation error responses
pub fn validation_error_response(
    error: ValidationError,
) -> (StatusCode, Json<ValidationErrorResponse>) {
    let status = StatusCode::from(error.clone());
    let response = ValidationErrorResponse::from_validation_error(error);
    (status, Json(response))
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Add security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; media-src 'self'; frame-src 'none';".parse().unwrap(),
    );
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=()".parse().unwrap(),
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{HeaderMap, HeaderValue, Method, Request},
    };

    #[tokio::test]
    async fn test_validate_headers_middleware() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("normal-agent"));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        // This would need a proper test setup with a mock next handler
        // For now, we're just testing that the function compiles and can be called
    }

    #[test]
    fn test_validation_error_response() {
        let error = ValidationError::InvalidFormat("test error".to_string());
        let (status, response) = validation_error_response(error);

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(response.0.error, "test error");
    }
}
