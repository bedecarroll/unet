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

/// Centralized error codes and message templates
pub mod error_codes {
    /// Error code constants for consistent error identification
    pub mod codes {
        // Configuration errors (CONFIG_*)
        /// Error code for invalid configuration file paths
        pub const CONFIG_INVALID_PATH: &str = "CONFIG_INVALID_PATH";
        /// Error code for configuration file parsing failures
        pub const CONFIG_PARSE_FAILED: &str = "CONFIG_PARSE_FAILED";
        /// Error code for configuration validation failures
        pub const CONFIG_VALIDATION_FAILED: &str = "CONFIG_VALIDATION_FAILED";
        /// Error code for environment variable override failures
        pub const CONFIG_ENV_OVERRIDE_FAILED: &str = "CONFIG_ENV_OVERRIDE_FAILED";

        // Database errors (DB_*)
        /// Error code for database connection failures
        pub const DB_CONNECTION_FAILED: &str = "DB_CONNECTION_FAILED";
        /// Error code for database query failures
        pub const DB_QUERY_FAILED: &str = "DB_QUERY_FAILED";
        /// Error code for database transaction failures
        pub const DB_TRANSACTION_FAILED: &str = "DB_TRANSACTION_FAILED";
        /// Error code for database constraint violations
        pub const DB_CONSTRAINT_VIOLATION: &str = "DB_CONSTRAINT_VIOLATION";
        /// Error code for database operation timeouts
        pub const DB_TIMEOUT: &str = "DB_TIMEOUT";

        // Policy errors (POLICY_*)
        /// Error code for policy parsing failures
        pub const POLICY_PARSE_FAILED: &str = "POLICY_PARSE_FAILED";
        /// Error code for policy validation failures
        pub const POLICY_VALIDATION_FAILED: &str = "POLICY_VALIDATION_FAILED";
        /// Error code for policy execution failures
        pub const POLICY_EXECUTION_FAILED: &str = "POLICY_EXECUTION_FAILED";
        /// Error code for policy condition evaluation failures
        pub const POLICY_CONDITION_FAILED: &str = "POLICY_CONDITION_FAILED";

        // Template errors (TEMPLATE_*)
        /// Error code for template parsing failures
        pub const TEMPLATE_PARSE_FAILED: &str = "TEMPLATE_PARSE_FAILED";
        /// Error code for template rendering failures
        pub const TEMPLATE_RENDER_FAILED: &str = "TEMPLATE_RENDER_FAILED";
        /// Error code for template not found errors
        pub const TEMPLATE_NOT_FOUND: &str = "TEMPLATE_NOT_FOUND";

        // SNMP errors (SNMP_*)
        /// Error code for SNMP connection failures
        pub const SNMP_CONNECTION_FAILED: &str = "SNMP_CONNECTION_FAILED";
        /// Error code for SNMP authentication failures
        pub const SNMP_AUTH_FAILED: &str = "SNMP_AUTH_FAILED";
        /// Error code for SNMP request timeouts
        pub const SNMP_TIMEOUT: &str = "SNMP_TIMEOUT";
        /// Error code for invalid SNMP OIDs
        pub const SNMP_INVALID_OID: &str = "SNMP_INVALID_OID";

        // Network errors (NET_*)
        /// Error code for invalid network addresses
        pub const NET_INVALID_ADDRESS: &str = "NET_INVALID_ADDRESS";
        /// Error code for network connection refused errors
        pub const NET_CONNECTION_REFUSED: &str = "NET_CONNECTION_REFUSED";
        /// Error code for DNS resolution failures
        pub const NET_DNS_RESOLUTION_FAILED: &str = "NET_DNS_RESOLUTION_FAILED";

        // Validation errors (VALID_*)
        /// Error code for missing required fields
        pub const VALID_REQUIRED_FIELD_MISSING: &str = "VALID_REQUIRED_FIELD_MISSING";
        /// Error code for invalid field formats
        pub const VALID_INVALID_FORMAT: &str = "VALID_INVALID_FORMAT";
        /// Error code for values out of valid range
        pub const VALID_VALUE_OUT_OF_RANGE: &str = "VALID_VALUE_OUT_OF_RANGE";

        // I/O errors (IO_*)
        /// Error code for file not found errors
        pub const IO_FILE_NOT_FOUND: &str = "IO_FILE_NOT_FOUND";
        /// Error code for permission denied errors
        pub const IO_PERMISSION_DENIED: &str = "IO_PERMISSION_DENIED";
        /// Error code for disk full errors
        pub const IO_DISK_FULL: &str = "IO_DISK_FULL";

        // Serialization errors (SERIAL_*)
        /// Error code for JSON serialization failures
        pub const SERIAL_JSON_FAILED: &str = "SERIAL_JSON_FAILED";
        /// Error code for TOML serialization failures
        pub const SERIAL_TOML_FAILED: &str = "SERIAL_TOML_FAILED";
        /// Error code for YAML serialization failures
        pub const SERIAL_YAML_FAILED: &str = "SERIAL_YAML_FAILED";
    }

    /// Centralized error message templates for consistent messaging
    pub mod templates {
        /// Configuration error message templates
        pub mod config {
            /// Template for invalid configuration file path errors
            pub const INVALID_PATH: &str = "Configuration file path contains invalid UTF-8: {path}";
            /// Template for configuration parsing failures
            pub const PARSE_FAILED: &str = "Failed to parse configuration file '{file}': {error}";
            /// Template for configuration validation failures
            pub const VALIDATION_FAILED: &str =
                "Configuration validation failed for field '{field}': {reason}";
            /// Template for environment variable override failures
            pub const ENV_OVERRIDE_FAILED: &str =
                "Failed to apply environment variable override '{var}': {error}";
        }

        /// Database error message templates
        pub mod database {
            /// Template for database connection failures
            pub const CONNECTION_FAILED: &str = "Failed to connect to database at '{url}': {error}";
            /// Template for database query failures
            pub const QUERY_FAILED: &str =
                "Database query failed for operation '{operation}': {error}";
            /// Template for database transaction failures
            pub const TRANSACTION_FAILED: &str =
                "Database transaction failed during '{operation}': {error}";
            /// Template for database constraint violations
            pub const CONSTRAINT_VIOLATION: &str =
                "Database constraint violation in '{table}': {constraint}";
            /// Template for database operation timeouts
            pub const TIMEOUT: &str =
                "Database operation '{operation}' timed out after {seconds} seconds";
        }

        /// Policy error message templates
        pub mod policy {
            /// Template for policy parsing failures
            pub const PARSE_FAILED: &str = "Failed to parse policy rule '{rule}': {error}";
            /// Template for policy validation failures
            pub const VALIDATION_FAILED: &str =
                "Policy validation failed for rule '{rule}': {reason}";
            /// Template for policy execution failures
            pub const EXECUTION_FAILED: &str = "Policy execution failed for rule '{rule}': {error}";
            /// Template for policy condition evaluation failures
            pub const CONDITION_FAILED: &str = "Policy condition evaluation failed: {condition}";
        }

        /// Template error message templates
        pub mod template {
            /// Template for template parsing failures
            pub const PARSE_FAILED: &str = "Failed to parse template '{template}': {error}";
            /// Template for template rendering failures
            pub const RENDER_FAILED: &str = "Failed to render template '{template}': {error}";
            /// Template for template not found errors
            pub const NOT_FOUND: &str = "Template '{template}' not found in search paths";
        }

        /// SNMP error message templates
        pub mod snmp {
            /// Template for SNMP connection failures
            pub const CONNECTION_FAILED: &str =
                "Failed to connect to SNMP agent at '{address}': {error}";
            /// Template for SNMP authentication failures
            pub const AUTH_FAILED: &str = "SNMP authentication failed for '{address}': {error}";
            /// Template for SNMP request timeouts
            pub const TIMEOUT: &str =
                "SNMP request to '{address}' timed out after {seconds} seconds";
            /// Template for invalid SNMP OID errors
            pub const INVALID_OID: &str = "Invalid SNMP OID '{oid}': {reason}";
        }

        /// Network error message templates
        pub mod network {
            /// Template for invalid network address errors
            pub const INVALID_ADDRESS: &str = "Invalid network address '{address}': {reason}";
            /// Template for network connection refused errors
            pub const CONNECTION_REFUSED: &str = "Connection refused to '{address}:{port}'";
            /// Template for DNS resolution failures
            pub const DNS_RESOLUTION_FAILED: &str =
                "DNS resolution failed for '{hostname}': {error}";
        }

        /// Validation error message templates
        pub mod validation {
            /// Template for missing required field errors
            pub const REQUIRED_FIELD_MISSING: &str = "Required field '{field}' is missing";
            /// Template for invalid field format errors
            pub const INVALID_FORMAT: &str = "Field '{field}' has invalid format: {expected}";
            /// Template for value out of range errors
            pub const VALUE_OUT_OF_RANGE: &str =
                "Field '{field}' value {value} is out of range [{min}, {max}]";
        }

        /// I/O error message templates
        pub mod io {
            /// Template for file not found errors
            pub const FILE_NOT_FOUND: &str = "File not found: '{path}'";
            /// Template for permission denied errors
            pub const PERMISSION_DENIED: &str = "Permission denied accessing '{path}'";
            /// Template for disk full errors
            pub const DISK_FULL: &str = "Insufficient disk space for operation on '{path}'";
        }

        /// Serialization error message templates
        pub mod serialization {
            /// Template for JSON serialization failures
            pub const JSON_FAILED: &str = "JSON serialization failed for type '{type}': {error}";
            /// Template for TOML serialization failures
            pub const TOML_FAILED: &str = "TOML serialization failed for type '{type}': {error}";
            /// Template for YAML serialization failures
            pub const YAML_FAILED: &str = "YAML serialization failed for type '{type}': {error}";
        }
    }

    /// Helper functions for creating standardized error messages
    pub mod helpers {
        use super::super::Error;

        /// Create a configuration error with standardized messaging
        #[must_use]
        pub fn config_error(template: &str, args: &[(&str, &str)]) -> Error {
            let message = format_template(template, args);
            Error::config(message)
        }

        /// Create a database error with standardized messaging
        #[must_use]
        pub fn database_error(operation: &str, template: &str, args: &[(&str, &str)]) -> Error {
            let message = format_template(template, args);
            Error::database(operation, message)
        }

        /// Create a policy error with standardized messaging  
        #[must_use]
        pub fn policy_error(rule: &str, template: &str, args: &[(&str, &str)]) -> Error {
            let message = format_template(template, args);
            Error::Policy {
                rule: rule.to_string(),
                message,
                source: None,
            }
        }

        /// Create a network error with standardized messaging
        #[must_use]
        pub fn network_error(endpoint: &str, template: &str, args: &[(&str, &str)]) -> Error {
            let message = format_template(template, args);
            Error::network(endpoint, message)
        }

        /// Create a validation error with standardized messaging
        #[must_use]
        pub fn validation_error(field: &str, template: &str, args: &[(&str, &str)]) -> Error {
            let message = format_template(template, args);
            Error::validation(field, message)
        }

        /// Format a template string with named arguments
        ///
        /// This is a simple template formatter that replaces {key} with values
        /// from the args slice. For more complex formatting needs, consider using
        /// a dedicated template engine.
        fn format_template(template: &str, args: &[(&str, &str)]) -> String {
            let mut result = template.to_string();
            for (key, value) in args {
                let placeholder = format!("{{{key}}}");
                result = result.replace(&placeholder, value);
            }
            result
        }
    }
}

/// Re-export commonly used error code constants for convenience
pub use error_codes::codes::*;
