//! HTTP request handlers

pub mod health;
pub mod nodes;
pub mod locations;
pub mod links;

// Re-export server error types for handlers
pub use crate::error::{ServerError, ServerResult};