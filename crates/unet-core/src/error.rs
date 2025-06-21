//! Error types for μNet Core

use thiserror::Error;

/// μNet Core error type
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Policy error
    #[error("Policy error: {0}")]
    Policy(String),

    /// Template error
    #[error("Template error: {0}")]
    Template(String),

    /// SNMP error
    #[error("SNMP error: {0}")]
    Snmp(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// μNet Core result type
pub type Result<T> = std::result::Result<T, Error>;
