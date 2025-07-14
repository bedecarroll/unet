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
