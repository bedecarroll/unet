//! Error types for μNet Core
//!
//! This module provides a comprehensive error hierarchy with proper context
//! and user-friendly messages for all μNet operations.

use thiserror::Error;

// Re-export all public types and functions
pub use self::codes::{helpers, templates};
pub use self::reporting::ErrorReporting;

mod codes;
mod constructors;
mod conversions;
mod reporting;

#[cfg(test)]
mod tests;

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
    #[error("SNMP error for target {target}: {message}")]
    Snmp {
        /// The SNMP target that failed
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

/// Result type for μNet Core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Re-export commonly used error code constants for convenience
pub use self::codes::constants::*;
