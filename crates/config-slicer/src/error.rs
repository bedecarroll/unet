//! Error types for config-slicer

use regex::Error as RegexError;
use std::io;
use thiserror::Error;

/// Config-slicer error type
#[derive(Error, Debug)]
pub enum ConfigSlicerError {
    /// Diffing failed.
    #[error("diff error: {0}")]
    Diff(String),

    /// I/O failed.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Match-expression parsing failed.
    #[error("invalid match expression: {0}")]
    Parse(String),

    /// Regex compilation failed.
    #[error("regex error: {0}")]
    Regex(#[from] RegexError),

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    /// Slicing failed.
    #[error("slice error: {0}")]
    Slice(String),
}

/// Config-slicer result type
pub type Result<T> = std::result::Result<T, ConfigSlicerError>;
