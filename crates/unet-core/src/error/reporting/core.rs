//! Error logging and reporting functionality for μNet Core

use super::super::{Error, Result};
use tracing::error;

impl Error {
    /// Log this error with appropriate level and context
    pub fn log(&self) {
        match self {
            Self::Config { message, .. } => self.log_config_error(message),
            Self::Database {
                operation, message, ..
            } => self.log_database_error(operation, message),
            Self::Policy { rule, message, .. } => self.log_policy_error(rule, message),
            Self::Template {
                template, message, ..
            } => self.log_template_error(template, message),
            Self::Snmp {
                target, message, ..
            } => self.log_snmp_error(target, message),
            Self::Validation {
                field,
                message,
                value,
            } => self.log_validation_error(field, message, value.as_ref()),
            Self::Network {
                endpoint, message, ..
            } => self.log_network_error(endpoint, message),
            Self::Io { path, message, .. } => self.log_io_error(path, message),
            Self::Serialization {
                format, message, ..
            } => self.log_serialization_error(format, message),
            Self::Other {
                context, message, ..
            } => self.log_other_error(context, message),
        }
    }

    fn log_config_error(&self, message: &str) {
        error!(error_code = self.error_code(), message = %message, "Configuration error");
    }

    fn log_database_error(&self, operation: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            operation = %operation,
            message = %message,
            "Database error"
        );
    }

    fn log_policy_error(&self, rule: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            rule = %rule,
            message = %message,
            "Policy error"
        );
    }

    fn log_template_error(&self, template: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            template = %template,
            message = %message,
            "Template error"
        );
    }

    fn log_snmp_error(&self, target: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            target = %target,
            message = %message,
            "SNMP error"
        );
    }

    fn log_validation_error(&self, field: &str, message: &str, value: Option<&String>) {
        if let Some(val) = value {
            error!(
                error_code = self.error_code(),
                field = %field,
                value = %val,
                message = %message,
                "Validation error"
            );
        } else {
            error!(
                error_code = self.error_code(),
                field = %field,
                message = %message,
                "Validation error"
            );
        }
    }

    fn log_network_error(&self, endpoint: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            endpoint = %endpoint,
            message = %message,
            "Network error"
        );
    }

    fn log_io_error(&self, path: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            path = %path,
            message = %message,
            "I/O error"
        );
    }

    fn log_serialization_error(&self, format: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            format = %format,
            message = %message,
            "Serialization error"
        );
    }

    fn log_other_error(&self, context: &str, message: &str) {
        error!(
            error_code = self.error_code(),
            context = %context,
            message = %message,
            "Other error"
        );
    }
}

/// Extension trait for Results to add error logging
pub trait ErrorReporting<T> {
    /// Log the error if present and return the result
    #[must_use]
    fn log_error(self) -> Self;

    /// Log the error with custom context and return the result
    #[must_use]
    fn log_error_with_context(self, context: &str) -> Self;

    /// Convert error to user-friendly message
    ///
    /// # Errors
    /// Returns an error with a user-friendly message if the original result was an error
    fn user_friendly(self) -> std::result::Result<T, String>;
}

impl<T> ErrorReporting<T> for Result<T> {
    fn log_error(self) -> Self {
        if let Err(ref e) = self {
            e.log();
        }
        self
    }

    fn log_error_with_context(self, context: &str) -> Self {
        if let Err(ref e) = self {
            error!(context = %context, error = %e, "Operation failed");
        }
        self
    }

    fn user_friendly(self) -> std::result::Result<T, String> {
        self.map_err(|e| e.user_message())
    }
}
