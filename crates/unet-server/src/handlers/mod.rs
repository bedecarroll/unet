//! HTTP request handlers

pub mod alerting;
pub mod auth;
pub mod certificates;
pub mod changes;
pub mod cluster;
pub mod distributed_locking;
pub mod git;
pub mod health;
pub mod links;
pub mod locations;
pub mod metrics;
pub mod network_access;
pub mod nodes;
pub mod performance;
pub mod policies;
pub mod resource_management;
pub mod templates;

// Re-export server error types for handlers
pub use crate::error::{ServerError, ServerResult};
