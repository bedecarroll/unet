//! Policy file validation logic

use crate::policy::{PolicyError, PolicyParser, PolicyResult, PolicyRule};

/// Policy file validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Total number of lines processed
    pub total_lines: usize,
    /// Number of valid policy rules found
    pub valid_rules: usize,
    /// List of validation errors encountered
    pub errors: Vec<ValidationError>,
}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Line number where the error occurred
    pub line: usize,
    /// Error message describing the problem
    pub message: String,
    /// Content of the line that caused the error
    pub content: String,
}

impl ValidationResult {
    /// Create a new validation result
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_lines: 0,
            valid_rules: 0,
            errors: Vec::new(),
        }
    }

    /// Check if validation passed (no errors)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get error count
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Add a validation error
    pub fn add_error(&mut self, line: usize, message: String, content: String) {
        self.errors.push(ValidationError {
            line,
            message,
            content,
        });
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy file validator
#[derive(Debug, Clone)]
pub struct PolicyValidator;

impl PolicyValidator {
    /// Create a new policy validator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validate policy file content
    pub fn validate_policy_file(content: &str) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Parse each line and collect validation results
        for (line_num, line) in content.lines().enumerate() {
            result.total_lines += 1;

            // Skip empty lines and comments
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            match PolicyParser::parse_rule(line) {
                Ok(_rule) => {
                    result.valid_rules += 1;
                }
                Err(err) => {
                    result.add_error(line_num + 1, err.to_string(), line.to_string());
                }
            }
        }

        result
    }

    /// Validate and parse policy content into rules
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if the content cannot be parsed into valid rules
    pub fn validate_and_parse(content: &str) -> PolicyResult<Vec<PolicyRule>> {
        let validation_result = Self::validate_policy_file(content);

        if !validation_result.is_valid() {
            return Err(PolicyError::ValidationError {
                message: format!(
                    "Policy validation failed with {} errors",
                    validation_result.error_count()
                ),
            });
        }

        // Parse all rules if validation passed
        let mut rules = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let rule = PolicyParser::parse_rule(line)?;
            rules.push(rule);
        }

        Ok(rules)
    }
}

impl Default for PolicyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();
        assert_eq!(result.total_lines, 0);
        assert_eq!(result.valid_rules, 0);
        assert!(result.errors.is_empty());
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert_eq!(result.total_lines, 0);
        assert_eq!(result.valid_rules, 0);
        assert!(result.errors.is_empty());
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();
        result.total_lines = 5;
        result.valid_rules = 3;

        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);

        result.add_error(2, "Invalid syntax".to_string(), "invalid line".to_string());

        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.errors[0].line, 2);
        assert_eq!(result.errors[0].message, "Invalid syntax");
        assert_eq!(result.errors[0].content, "invalid line");
    }

    #[test]
    fn test_validation_result_multiple_errors() {
        let mut result = ValidationResult::new();

        result.add_error(1, "Error 1".to_string(), "line 1".to_string());
        result.add_error(3, "Error 2".to_string(), "line 3".to_string());
        result.add_error(5, "Error 3".to_string(), "line 5".to_string());

        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 3);
        assert_eq!(result.errors.len(), 3);
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError {
            line: 10,
            message: "Test error message".to_string(),
            content: "test content".to_string(),
        };

        assert_eq!(error.line, 10);
        assert_eq!(error.message, "Test error message");
        assert_eq!(error.content, "test content");
    }

    #[test]
    fn test_policy_validator_new() {
        let _ = PolicyValidator::new();
        // PolicyValidator is a unit struct, so just verify it can be created
        let _ = PolicyValidator;
    }

    #[test]
    fn test_policy_validator_default() {
        let _ = PolicyValidator;
        let _ = PolicyValidator::new();
        // Both should be identical (unit structs)
    }

    #[test]
    fn test_validation_result_is_valid_with_no_errors() {
        let mut result = ValidationResult::new();
        result.total_lines = 10;
        result.valid_rules = 10;

        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validation_result_is_invalid_with_errors() {
        let mut result = ValidationResult::new();
        result.total_lines = 10;
        result.valid_rules = 8;

        result.add_error(2, "Syntax error".to_string(), "bad syntax".to_string());
        result.add_error(
            7,
            "Another error".to_string(),
            "another bad line".to_string(),
        );

        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 2);
    }

    #[test]
    fn test_validation_error_fields() {
        let mut result = ValidationResult::new();
        result.add_error(
            15,
            "Field validation failed".to_string(),
            "field = invalid_value".to_string(),
        );

        let error = &result.errors[0];
        assert_eq!(error.line, 15);
        assert_eq!(error.message, "Field validation failed");
        assert_eq!(error.content, "field = invalid_value");
    }

    #[test]
    fn test_validation_result_debug() {
        let mut result = ValidationResult::new();
        result.total_lines = 5;
        result.valid_rules = 4;
        result.add_error(3, "Debug test".to_string(), "debug line".to_string());

        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("ValidationResult"));
        assert!(debug_str.contains("total_lines: 5"));
        assert!(debug_str.contains("valid_rules: 4"));
    }

    #[test]
    fn test_validation_error_debug() {
        let error = ValidationError {
            line: 42,
            message: "Debug error".to_string(),
            content: "debug content".to_string(),
        };

        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("ValidationError"));
        assert!(debug_str.contains("line: 42"));
        assert!(debug_str.contains("Debug error"));
        assert!(debug_str.contains("debug content"));
    }
}
