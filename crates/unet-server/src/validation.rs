//! Input validation and sanitization module

use axum::http::StatusCode;
use regex::Regex;
use sanitize_html::{rules::predefined::DEFAULT, sanitize_str};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

/// Validation error types
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Validation failed: {0}")]
    InvalidInput(#[from] ValidationErrors),
    #[error("Sanitization failed: {0}")]
    SanitizationFailed(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl From<ValidationError> for StatusCode {
    fn from(err: ValidationError) -> Self {
        match err {
            ValidationError::InvalidInput(_)
            | ValidationError::SanitizationFailed(_)
            | ValidationError::InvalidFormat(_) => StatusCode::BAD_REQUEST,
        }
    }
}

/// Input sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitize HTML content to prevent XSS attacks
    pub fn sanitize_html(input: &str) -> String {
        sanitize_str(&DEFAULT, input).unwrap_or_else(|_| String::new())
    }

    /// Sanitize string input by removing potentially dangerous characters
    pub fn sanitize_string(input: &str) -> Result<String, ValidationError> {
        // Remove null bytes and other control characters
        let cleaned = input
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>();

        // Limit length to prevent DoS
        if cleaned.len() > 10000 {
            return Err(ValidationError::InvalidFormat(
                "Input too long (max 10000 characters)".to_string(),
            ));
        }

        Ok(cleaned)
    }

    /// Sanitize and validate UUID strings
    pub fn sanitize_uuid(input: &str) -> Result<String, ValidationError> {
        let cleaned = Self::sanitize_string(input)?;

        // Validate UUID format
        if uuid::Uuid::parse_str(&cleaned).is_err() {
            return Err(ValidationError::InvalidFormat(
                "Invalid UUID format".to_string(),
            ));
        }

        Ok(cleaned)
    }

    /// Sanitize network identifiers (node names, etc.)
    pub fn sanitize_identifier(input: &str) -> Result<String, ValidationError> {
        let cleaned = Self::sanitize_string(input)?;

        // Allow only alphanumeric, hyphens, underscores, and dots
        let re = Regex::new(r"^[a-zA-Z0-9_.-]+$").unwrap();
        if !re.is_match(&cleaned) {
            return Err(ValidationError::InvalidFormat(
                "Identifier can only contain alphanumeric characters, hyphens, underscores, and dots".to_string(),
            ));
        }

        // Ensure reasonable length
        if cleaned.len() > 255 {
            return Err(ValidationError::InvalidFormat(
                "Identifier too long (max 255 characters)".to_string(),
            ));
        }

        Ok(cleaned)
    }

    /// Sanitize IP addresses
    pub fn sanitize_ip_address(input: &str) -> Result<String, ValidationError> {
        let cleaned = Self::sanitize_string(input)?;

        // Validate IP address format
        if cleaned.parse::<std::net::IpAddr>().is_err() {
            return Err(ValidationError::InvalidFormat(
                "Invalid IP address format".to_string(),
            ));
        }

        Ok(cleaned)
    }

    /// Sanitize email addresses
    pub fn sanitize_email(input: &str) -> Result<String, ValidationError> {
        let cleaned = Self::sanitize_string(input)?;

        // Basic email validation
        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !re.is_match(&cleaned) {
            return Err(ValidationError::InvalidFormat(
                "Invalid email format".to_string(),
            ));
        }

        Ok(cleaned.to_lowercase())
    }

    /// Sanitize JSON values to prevent injection
    pub fn sanitize_json_value(
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, ValidationError> {
        match input {
            serde_json::Value::String(s) => {
                let sanitized = Self::sanitize_string(s)?;
                Ok(serde_json::Value::String(sanitized))
            }
            serde_json::Value::Object(obj) => {
                let mut sanitized_obj = serde_json::Map::new();
                for (key, value) in obj {
                    let sanitized_key = Self::sanitize_string(key)?;
                    let sanitized_value = Self::sanitize_json_value(value)?;
                    sanitized_obj.insert(sanitized_key, sanitized_value);
                }
                Ok(serde_json::Value::Object(sanitized_obj))
            }
            serde_json::Value::Array(arr) => {
                let mut sanitized_arr = Vec::new();
                for item in arr {
                    sanitized_arr.push(Self::sanitize_json_value(item)?);
                }
                Ok(serde_json::Value::Array(sanitized_arr))
            }
            // Numbers, booleans, and null values are safe as-is
            other => Ok(other.clone()),
        }
    }
}

/// Trait for validating and sanitizing request payloads
pub trait ValidatedRequest: Validate {
    type Sanitized;

    /// Validate and sanitize the request
    fn validate_and_sanitize(self) -> Result<Self::Sanitized, ValidationError>;
}

/// Generic validation helper
pub struct ValidationHelper;

impl ValidationHelper {
    /// Validate a struct using the validator crate
    pub fn validate<T: Validate>(input: &T) -> Result<(), ValidationError> {
        input.validate().map_err(ValidationError::InvalidInput)
    }

    /// Validate and return detailed error messages
    pub fn validate_with_details<T: Validate>(
        input: &T,
    ) -> Result<(), HashMap<String, Vec<String>>> {
        match input.validate() {
            Ok(()) => Ok(()),
            Err(errors) => {
                let mut error_map = HashMap::new();
                for (field, field_errors) in errors.field_errors() {
                    let messages: Vec<String> = field_errors
                        .iter()
                        .map(|e| {
                            e.message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "Validation error".to_string())
                        })
                        .collect();
                    error_map.insert(field.to_string(), messages);
                }
                Err(error_map)
            }
        }
    }

    /// Check for SQL injection patterns
    pub fn check_sql_injection(input: &str) -> Result<(), ValidationError> {
        let dangerous_patterns = [
            "union select",
            "drop table",
            "delete from",
            "insert into",
            "update set",
            "exec ",
            "execute ",
            "script",
            "javascript:",
            "vbscript:",
            "onload=",
            "onerror=",
            "<script",
            "</script>",
        ];

        let input_lower = input.to_lowercase();
        for pattern in &dangerous_patterns {
            if input_lower.contains(pattern) {
                return Err(ValidationError::InvalidFormat(format!(
                    "Potentially dangerous content detected: {}",
                    pattern
                )));
            }
        }

        Ok(())
    }

    /// Check for path traversal attempts
    pub fn check_path_traversal(input: &str) -> Result<(), ValidationError> {
        if input.contains("..") || input.contains("~") || input.starts_with('/') {
            return Err(ValidationError::InvalidFormat(
                "Path traversal attempt detected".to_string(),
            ));
        }
        Ok(())
    }

    /// Check for command injection patterns
    pub fn check_command_injection(input: &str) -> Result<(), ValidationError> {
        let dangerous_chars = ['|', '&', ';', '`', '$', '(', ')', '<', '>'];

        if input.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(ValidationError::InvalidFormat(
                "Potentially dangerous command characters detected".to_string(),
            ));
        }

        Ok(())
    }
}

/// Response type for validation errors
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    pub error: String,
    pub details: Option<HashMap<String, Vec<String>>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ValidationErrorResponse {
    pub fn new(message: String, details: Option<HashMap<String, Vec<String>>>) -> Self {
        Self {
            error: message,
            details,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn from_validation_error(err: ValidationError) -> Self {
        match err {
            ValidationError::InvalidInput(validation_errors) => {
                let mut details = HashMap::new();
                for (field, field_errors) in validation_errors.field_errors() {
                    let messages: Vec<String> = field_errors
                        .iter()
                        .map(|e| {
                            e.message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "Validation error".to_string())
                        })
                        .collect();
                    details.insert(field.to_string(), messages);
                }
                Self::new("Validation failed".to_string(), Some(details))
            }
            ValidationError::SanitizationFailed(msg) | ValidationError::InvalidFormat(msg) => {
                Self::new(msg, None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Validate)]
    struct TestStruct {
        #[validate(length(min = 1, max = 100))]
        name: String,
        #[validate(email)]
        email: String,
    }

    #[test]
    fn test_sanitize_string() {
        let result = InputSanitizer::sanitize_string("normal string");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "normal string");

        let result = InputSanitizer::sanitize_string("string\x00with\x01control\x02chars");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "stringwithcontrolchars");
    }

    #[test]
    fn test_sanitize_identifier() {
        assert!(InputSanitizer::sanitize_identifier("valid_name-123").is_ok());
        assert!(InputSanitizer::sanitize_identifier("invalid@name").is_err());
        assert!(InputSanitizer::sanitize_identifier("invalid name").is_err());
    }

    #[test]
    fn test_sanitize_ip_address() {
        assert!(InputSanitizer::sanitize_ip_address("192.168.1.1").is_ok());
        assert!(InputSanitizer::sanitize_ip_address("::1").is_ok());
        assert!(InputSanitizer::sanitize_ip_address("invalid.ip").is_err());
    }

    #[test]
    fn test_sql_injection_detection() {
        assert!(ValidationHelper::check_sql_injection("normal text").is_ok());
        assert!(ValidationHelper::check_sql_injection("'; DROP TABLE users; --").is_err());
        assert!(ValidationHelper::check_sql_injection("1 UNION SELECT * FROM passwords").is_err());
    }

    #[test]
    fn test_path_traversal_detection() {
        assert!(ValidationHelper::check_path_traversal("normal/path").is_ok());
        assert!(ValidationHelper::check_path_traversal("../../../etc/passwd").is_err());
        assert!(ValidationHelper::check_path_traversal("~/sensitive").is_err());
    }

    #[test]
    fn test_command_injection_detection() {
        assert!(ValidationHelper::check_command_injection("normal text").is_ok());
        assert!(ValidationHelper::check_command_injection("rm -rf / | echo done").is_err());
        assert!(ValidationHelper::check_command_injection("$(malicious_command)").is_err());
    }
}
