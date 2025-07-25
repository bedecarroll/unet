//! HTTP request handlers

pub mod health;
pub mod nodes;
pub mod policies;

// Re-export server error types for handlers
pub use crate::error::{ServerError, ServerResult};
