//! HTTP request handlers

pub mod changes;
pub mod git;
pub mod health;
pub mod links;
pub mod locations;
pub mod nodes;
pub mod policies;
pub mod templates;

// Re-export server error types for handlers
pub use crate::error::{ServerError, ServerResult};
