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
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::Config { .. } => "CONFIG_ERROR",
            Error::Database { .. } => "DATABASE_ERROR",
            Error::Policy { .. } => "POLICY_ERROR",
            Error::Template { .. } => "TEMPLATE_ERROR",
            Error::Snmp { .. } => "SNMP_ERROR",
            Error::Validation { .. } => "VALIDATION_ERROR",
            Error::Network { .. } => "NETWORK_ERROR",
            Error::Io { .. } => "IO_ERROR",
            Error::Serialization { .. } => "SERIALIZATION_ERROR",
            Error::Other { .. } => "OTHER_ERROR",
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Error::Config { message, .. } => {
                format!("Configuration problem: {}", message)
            }
            Error::Database {
                operation, message, ..
            } => {
                format!("Database problem during {}: {}", operation, message)
            }
            Error::Policy { rule, message, .. } => {
                format!("Policy rule '{}' failed: {}", rule, message)
            }
            Error::Template {
                template, message, ..
            } => {
                format!("Template '{}' failed: {}", template, message)
            }
            Error::Snmp {
                target, message, ..
            } => {
                format!("SNMP error for {}: {}", target, message)
            }
            Error::Validation {
                field,
                message,
                value,
            } => {
                if let Some(val) = value {
                    format!("Invalid value '{}' for {}: {}", val, field, message)
                } else {
                    format!("Invalid {}: {}", field, message)
                }
            }
            Error::Network {
                endpoint, message, ..
            } => {
                format!("Network error connecting to {}: {}", endpoint, message)
            }
            Error::Io { path, message, .. } => {
                format!("File error with '{}': {}", path, message)
            }
            Error::Serialization {
                format, message, ..
            } => {
                format!("{} format error: {}", format, message)
            }
            Error::Other {
                context, message, ..
            } => {
                format!("Error in {}: {}", context, message)
            }
        }
    }

    /// Log this error with appropriate level and context
    pub fn log(&self) {
        match self {
            Error::Config { message, .. } => {
                error!(error_code = self.error_code(), message = %message, "Configuration error");
            }
            Error::Database {
                operation, message, ..
            } => {
                error!(
                    error_code = self.error_code(),
                    operation = %operation,
                    message = %message,
                    "Database error"
                );
            }
            Error::Policy { rule, message, .. } => {
                error!(
                    error_code = self.error_code(),
                    rule = %rule,
                    message = %message,
                    "Policy error"
                );
            }
            Error::Template {
                template, message, ..
            } => {
                error!(
                    error_code = self.error_code(),
                    template = %template,
                    message = %message,
                    "Template error"
                );
            }
            Error::Snmp {
                target, message, ..
            } => {
                error!(
                    error_code = self.error_code(),
                    target = %target,
                    message = %message,
                    "SNMP error"
                );
            }
            Error::Validation {
                field,
                message,
                value,
            } => {
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
            Error::Network {
                endpoint, message, ..
            } => {
                error!(
                    error_code = self.error_code(),
                    endpoint = %endpoint,
                    message = %message,
                    "Network error"
                );
            }
            Error::Io { path, message, .. } => {
                error!(
                    error_code = self.error_code(),
                    path = %path,
                    message = %message,
                    "I/O error"
                );
            }
            Error::Serialization {
                format, message, ..
            } => {
                error!(
                    error_code = self.error_code(),
                    format = %format,
                    message = %message,
                    "Serialization error"
                );
            }
            Error::Other {
                context, message, ..
            } => {
                error!(
                    error_code = self.error_code(),
                    context = %context,
                    message = %message,
                    "Other error"
                );
            }
        }
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
    fn log_error(self) -> Self;

    /// Log the error with custom context and return the result
    fn log_error_with_context(self, context: &str) -> Self;

    /// Convert error to user-friendly message
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
