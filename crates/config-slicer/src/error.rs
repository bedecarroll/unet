//! Error types for config-slicer

use std::io;
use thiserror::Error;

/// Config-slicer error type
#[derive(Error, Debug)]
pub enum ConfigSlicerError {
    /// Diff error
    #[error("Diff error: {0}")]
    Diff(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),

    /// Parsing error
    #[error("Parsing error: {0}")]
    Parse(String),

    /// Slicing error
    #[error("Slicing error: {0}")]
    Slice(String),
}

/// Config-slicer result type
pub type Result<T> = std::result::Result<T, ConfigSlicerError>;
