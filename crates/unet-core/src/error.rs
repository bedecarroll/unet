//! Error types for μNet Core
//!
//! This module provides a comprehensive error hierarchy with proper context
//! and user-friendly messages for all μNet operations.

use thiserror::Error;
use tracing::error;

/// μNet Core error type with comprehensive context
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error with details
    #[error("Configuration error: {message}")]
    Config {
        /// Human-readable error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Database error with operation context
    #[error("Database error during {operation}: {message}")]
    Database {
        /// The database operation that failed
        operation: String,
        /// Human-readable error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Policy error with rule context
    #[error("Policy error in rule '{rule}': {message}")]
    Policy {
        /// The policy rule that failed
        rule: String,
        /// Human-readable error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Template error with template context
    #[error("Template error in '{template}': {message}")]
    Template {
        /// The template that failed
        template: String,
        /// Human-readable error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// SNMP error with target context
    #[error("SNMP error for target '{target}': {message}")]
    Snmp {
        /// The SNMP target (IP/hostname)
        target: String,
        /// Human-readable error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Validation error with field context
    #[error("Validation error for field '{field}': {message}")]
    Validation {
        /// The field that failed validation
        field: String,
        /// Human-readable error message
        message: String,
        /// The invalid value (optional)
        value: Option<String>,
    },

    /// Network error with connectivity context
    #[error("Network error connecting to '{endpoint}': {message}")]
    Network {
        /// The network endpoint
        endpoint: String,
        /// Human-readable error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O error with file context
    #[error("I/O error with file '{path}': {message}")]
    Io {
        /// The file path involved
        path: String,
        /// Human-readable error message
        message: String,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Serialization error with format context
    #[error("Serialization error for {format}: {message}")]
    Serialization {
        /// The serialization format (JSON, YAML, etc.)
        format: String,
        /// Human-readable error message
        message: String,
        /// The underlying serialization error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Other error with context
    #[error("Error in {context}: {message}")]
    Other {
        /// The operation context
        context: String,
        /// Human-readable error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    /// Create a configuration error with a simple message
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
        }
    }

    /// Create a configuration error with source
    pub fn config_with_source<S: Into<String>, E>(message: S, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Config {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a database error with operation context
    pub fn database<S1: Into<String>, S2: Into<String>>(operation: S1, message: S2) -> Self {
        Self::Database {
            operation: operation.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a database error with operation context and source
    pub fn database_with_source<S1: Into<String>, S2: Into<String>, E>(
        operation: S1,
        message: S2,
        source: E,
    ) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Database {
            operation: operation.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a validation error
    pub fn validation<S1: Into<String>, S2: Into<String>>(field: S1, message: S2) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            value: None,
        }
    }

    /// Create a validation error with value context
    pub fn validation_with_value<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        field: S1,
        message: S2,
        value: S3,
    ) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            value: Some(value.into()),
        }
    }

    /// Create an SNMP error with target context
    pub fn snmp<S1: Into<String>, S2: Into<String>>(target: S1, message: S2) -> Self {
        Self::Snmp {
            target: target.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create an SNMP error with target context and source
    pub fn snmp_with_source<S1: Into<String>, S2: Into<String>, E>(
        target: S1,
        message: S2,
        source: E,
    ) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Snmp {
            target: target.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a network error
    pub fn network<S1: Into<String>, S2: Into<String>>(endpoint: S1, message: S2) -> Self {
        Self::Network {
            endpoint: endpoint.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a network error with source
    pub fn network_with_source<S1: Into<String>, S2: Into<String>, E>(
        endpoint: S1,
        message: S2,
        source: E,
    ) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Network {
            endpoint: endpoint.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Get the error code for this error type
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::Config { .. } => "CONFIG_ERROR",
            Self::Database { .. } => "DATABASE_ERROR",
            Self::Policy { .. } => "POLICY_ERROR",
            Self::Template { .. } => "TEMPLATE_ERROR",
            Self::Snmp { .. } => "SNMP_ERROR",
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::Network { .. } => "NETWORK_ERROR",
            Self::Io { .. } => "IO_ERROR",
            Self::Serialization { .. } => "SERIALIZATION_ERROR",
            Self::Other { .. } => "OTHER_ERROR",
        }
    }

    /// Get a user-friendly error message
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Config { message, .. } => {
                format!("Configuration problem: {message}")
            }
            Self::Database {
                operation, message, ..
            } => {
                format!("Database problem during {operation}: {message}")
            }
            Self::Policy { rule, message, .. } => {
                format!("Policy rule '{rule}' failed: {message}")
            }
            Self::Template {
                template, message, ..
            } => {
                format!("Template '{template}' failed: {message}")
            }
            Self::Snmp {
                target, message, ..
            } => {
                format!("SNMP error for {target}: {message}")
            }
            Self::Validation {
                field,
                message,
                value,
            } => value.as_ref().map_or_else(
                || format!("Invalid {field}: {message}"),
                |val| format!("Invalid value '{val}' for {field}: {message}"),
            ),
            Self::Network {
                endpoint, message, ..
            } => {
                format!("Network error connecting to {endpoint}: {message}")
            }
            Self::Io { path, message, .. } => {
                format!("File error with '{path}': {message}")
            }
            Self::Serialization {
                format, message, ..
            } => {
                format!("{format} format error: {message}")
            }
            Self::Other {
                context, message, ..
            } => {
                format!("Error in {context}: {message}")
            }
        }
    }

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

// Implement From traits for common error conversions
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            path: "<unknown>".to_string(),
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            format: "JSON".to_string(),
            message: err.to_string(),
            source: Box::new(err),
        }
    }
}

/// μNet Core result type
pub type Result<T> = std::result::Result<T, Error>;

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
