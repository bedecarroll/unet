//! Error constructor methods for μNet Core Error types

use super::Error;

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
}
