//! Node API handlers
//!
//! This module provides HTTP handlers for node management operations
//! including CRUD operations and derived state endpoints.

pub use crud::{create_node, delete_node, get_node, list_nodes, update_node};
pub use derived::{get_node_interfaces, get_node_metrics, get_node_status};

#[cfg(test)]
mod create_tests;
mod crud;
#[cfg(test)]
mod crud_tests; // Contains organized test modules by operation type
#[cfg(test)]
mod delete_tests;
mod derived;
#[cfg(test)]
mod read_tests;
#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests;
mod types;
#[cfg(test)]
mod update_tests;
