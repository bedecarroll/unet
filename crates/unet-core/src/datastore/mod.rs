//! `DataStore` abstraction layer for μNet Core
//!
//! This module provides the `DataStore` trait and related types for abstracting
//! data storage operations across different backends (`CSV`, `SQLite`, `PostgreSQL`, etc.).

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{Link, Location, Node};
use crate::policy::PolicyExecutionResult;

pub mod csv;
pub mod helpers;
pub mod sqlite;
#[cfg(test)]
mod tests;
pub mod transaction_helpers;
pub mod types;

// Re-export main types for backward compatibility
pub use types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, Filter, FilterOperation,
    FilterValue, PagedResult, Pagination, QueryOptions, Sort, SortDirection, Transaction,
};

pub use helpers::{filter_contains, filter_equals_string, filter_equals_uuid, sort_asc, sort_desc};

pub use transaction_helpers::{
    batch_with_transaction, retry_transaction, with_transaction, with_transaction_control,
};

/// Main DataStore trait for abstracting data access
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Returns the name/type of this datastore implementation
    fn name(&self) -> &'static str;

    /// Checks if the datastore is healthy and can serve requests
    ///
    /// # Errors
    /// Returns an error if the datastore is unhealthy or unreachable
    async fn health_check(&self) -> DataStoreResult<()>;

    /// Begins a new transaction
    ///
    /// # Errors
    /// Returns an error if the transaction cannot be started
    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>>;

    // Node operations
    /// Creates a new node
    ///
    /// # Errors
    /// Returns an error if the node cannot be created or validation fails
    async fn create_node(&self, node: &Node) -> DataStoreResult<Node>;

    /// Gets a node by ID
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>>;

    /// Gets a node by ID, returning an error if not found
    async fn get_node_required(&self, id: &Uuid) -> DataStoreResult<Node> {
        self.get_node(id).await?.map_or_else(
            || {
                Err(DataStoreError::NotFound {
                    entity_type: "Node".to_string(),
                    id: id.to_string(),
                })
            },
            Ok,
        )
    }

    /// Lists nodes with optional filtering, sorting, and pagination
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>>;

    /// Updates an existing node
    ///
    /// # Errors
    /// Returns an error if the node cannot be updated or validation fails
    async fn update_node(&self, node: &Node) -> DataStoreResult<Node>;

    /// Deletes a node by ID
    ///
    /// # Errors
    /// Returns an error if the node cannot be deleted or doesn't exist
    async fn delete_node(&self, id: &Uuid) -> DataStoreResult<()>;

    /// Gets nodes by location ID
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn get_nodes_by_location(&self, location_id: &Uuid) -> DataStoreResult<Vec<Node>>;

    /// Searches nodes by name (case-insensitive partial match)
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn search_nodes_by_name(&self, name: &str) -> DataStoreResult<Vec<Node>>;

    // Link operations
    /// Creates a new link
    ///
    /// # Errors
    /// Returns an error if the link cannot be created or validation fails
    async fn create_link(&self, link: &Link) -> DataStoreResult<Link>;

    /// Gets a link by ID
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn get_link(&self, id: &Uuid) -> DataStoreResult<Option<Link>>;

    /// Gets a link by ID, returning an error if not found
    async fn get_link_required(&self, id: &Uuid) -> DataStoreResult<Link> {
        self.get_link(id).await?.map_or_else(
            || {
                Err(DataStoreError::NotFound {
                    entity_type: "Link".to_string(),
                    id: id.to_string(),
                })
            },
            Ok,
        )
    }

    /// Lists links with optional filtering, sorting, and pagination
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn list_links(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Link>>;

    /// Updates an existing link
    ///
    /// # Errors
    /// Returns an error if the link cannot be updated or validation fails
    async fn update_link(&self, link: &Link) -> DataStoreResult<Link>;

    /// Deletes a link by ID
    ///
    /// # Errors
    /// Returns an error if the link cannot be deleted or doesn't exist
    async fn delete_link(&self, id: &Uuid) -> DataStoreResult<()>;

    /// Gets links involving a specific node
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn get_links_for_node(&self, node_id: &Uuid) -> DataStoreResult<Vec<Link>>;

    /// Gets links between two specific nodes
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn get_links_between_nodes(
        &self,
        first_node_id: &Uuid,
        second_node_id: &Uuid,
    ) -> DataStoreResult<Vec<Link>>;

    // Location operations
    /// Creates a new location
    ///
    /// # Errors
    /// Returns an error if the location cannot be created or validation fails
    async fn create_location(&self, location: &Location) -> DataStoreResult<Location>;

    /// Gets a location by ID
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn get_location(&self, id: &Uuid) -> DataStoreResult<Option<Location>>;

    /// Gets a location by ID, returning an error if not found
    async fn get_location_required(&self, id: &Uuid) -> DataStoreResult<Location> {
        self.get_location(id).await?.map_or_else(
            || {
                Err(DataStoreError::NotFound {
                    entity_type: "Location".to_string(),
                    id: id.to_string(),
                })
            },
            Ok,
        )
    }

    /// Lists locations with optional filtering, sorting, and pagination
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn list_locations(
        &self,
        options: &QueryOptions,
    ) -> DataStoreResult<PagedResult<Location>>;

    /// Updates an existing location
    ///
    /// # Errors
    /// Returns an error if the location cannot be updated or validation fails
    async fn update_location(&self, location: &Location) -> DataStoreResult<Location>;

    /// Deletes a location by ID
    ///
    /// # Errors
    /// Returns an error if the location cannot be deleted or doesn't exist
    async fn delete_location(&self, id: &Uuid) -> DataStoreResult<()>;

    // Batch operations
    /// Performs batch operations on nodes
    ///
    /// # Errors
    /// Returns an error if any batch operation fails
    async fn batch_nodes(
        &self,
        operations: &[BatchOperation<Node>],
    ) -> DataStoreResult<BatchResult>;

    /// Performs batch operations on links
    ///
    /// # Errors
    /// Returns an error if any batch operation fails
    async fn batch_links(
        &self,
        operations: &[BatchOperation<Link>],
    ) -> DataStoreResult<BatchResult>;

    /// Performs batch operations on locations
    ///
    /// # Errors
    /// Returns an error if any batch operation fails
    async fn batch_locations(
        &self,
        operations: &[BatchOperation<Location>],
    ) -> DataStoreResult<BatchResult>;

    // Statistics and metadata
    /// Gets count of all entities
    ///
    /// # Errors
    /// Returns an error if the database query fails
    async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>>;

    /// Gets datastore statistics (implementation-specific)
    ///
    /// # Errors
    /// Returns an error if the statistics cannot be collected
    async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>>;

    // Derived state operations (basic implementation)
    /// Gets node status (derived state) by node ID
    async fn get_node_status(
        &self,
        node_id: &Uuid,
    ) -> DataStoreResult<Option<crate::models::derived::NodeStatus>> {
        // Default implementation returns a basic status
        Ok(Some(crate::models::derived::NodeStatus::new(*node_id)))
    }

    /// Gets interface status for a specific node
    async fn get_node_interfaces(
        &self,
        _node_id: &Uuid,
    ) -> DataStoreResult<Vec<crate::models::derived::InterfaceStatus>> {
        // Default implementation returns empty list
        Ok(Vec::new())
    }

    /// Gets performance metrics for a specific node
    async fn get_node_metrics(
        &self,
        _node_id: &Uuid,
    ) -> DataStoreResult<Option<crate::models::derived::PerformanceMetrics>> {
        // Default implementation returns None
        Ok(None)
    }

    // Policy-related operations
    /// Stores a policy execution result
    async fn store_policy_result(
        &self,
        node_id: &Uuid,
        rule_id: &str,
        result: &PolicyExecutionResult,
    ) -> DataStoreResult<()> {
        // Default implementation is a no-op for backward compatibility
        // Parameters are intentionally unused in this default trait implementation
        let _ = (node_id, rule_id, result);
        Ok(())
    }

    /// Gets policy execution results for a node
    async fn get_policy_results(
        &self,
        node_id: &Uuid,
    ) -> DataStoreResult<Vec<PolicyExecutionResult>> {
        // Default implementation returns empty results
        // Parameter is intentionally unused in this default trait implementation
        let _ = node_id;
        Ok(Vec::new())
    }

    /// Gets the latest policy execution results for a node
    async fn get_latest_policy_results(
        &self,
        node_id: &Uuid,
    ) -> DataStoreResult<Vec<PolicyExecutionResult>> {
        // Default implementation delegates to get_policy_results
        self.get_policy_results(node_id).await
    }

    /// Gets policy execution results for a specific rule across all nodes
    async fn get_rule_results(
        &self,
        rule_id: &str,
    ) -> DataStoreResult<Vec<(Uuid, PolicyExecutionResult)>> {
        // Default implementation returns empty results
        // Parameter is intentionally unused in this default trait implementation
        let _ = rule_id;
        Ok(Vec::new())
    }

    /// Updates custom_data field for a node (used by SET actions)
    async fn update_node_custom_data(
        &self,
        node_id: &Uuid,
        custom_data: &serde_json::Value,
    ) -> DataStoreResult<()> {
        // Default implementation: get node, update custom_data, save node
        if let Some(mut node) = self.get_node(node_id).await? {
            node.custom_data = custom_data.clone();
            self.update_node(&node).await?;
            Ok(())
        } else {
            Err(DataStoreError::NotFound {
                entity_type: "Node".to_string(),
                id: node_id.to_string(),
            })
        }
    }

    /// Gets all nodes for policy evaluation
    async fn get_nodes_for_policy_evaluation(&self) -> DataStoreResult<Vec<Node>> {
        // Default implementation: get all nodes using list_nodes with no filters
        let options = QueryOptions::default();
        let result = self.list_nodes(&options).await?;
        Ok(result.items)
    }
}
