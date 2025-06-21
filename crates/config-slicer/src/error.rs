//! Error types for config-slicer

use thiserror::Error;

/// Config-slicer error type
#[derive(Error, Debug)]
pub enum Error {
    /// Parsing error
    #[error("Parsing error: {0}")]
    Parse(String),

    /// Slicing error
    #[error("Slicing error: {0}")]
    Slice(String),

    /// Diff error
    #[error("Diff error: {0}")]
    Diff(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Config-slicer result type
pub type Result<T> = std::result::Result<T, Error>;
