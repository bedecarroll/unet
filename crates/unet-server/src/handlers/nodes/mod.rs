//! Node API handlers
//!
//! This module provides HTTP handlers for node management operations
//! including CRUD operations and derived state endpoints.

pub use crud::{create_node, delete_node, get_node, list_nodes, update_node};
pub use derived::{get_node_interfaces, get_node_metrics, get_node_status};

mod crud;
mod derived;
#[cfg(test)]
mod tests;
mod types;
