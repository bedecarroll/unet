//! HTTP request handlers

pub mod health;
pub mod links;
pub mod locations;
pub mod nodes;

// Re-export server error types for handlers
pub use crate::error::{ServerError, ServerResult};
