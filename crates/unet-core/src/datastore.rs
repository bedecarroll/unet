//! DataStore abstraction layer for μNet Core
//!
//! This module provides the DataStore trait and related types for abstracting
//! data storage operations across different backends (CSV, SQLite, PostgreSQL, etc.).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::template::{Template, TemplateAssignment, TemplateUsage, TemplateVersion};
use crate::models::{Link, Location, Node};
use crate::policy::PolicyExecutionResult;

/// Errors that can occur during datastore operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum DataStoreError {
    /// Entity not found
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound {
        /// The type of entity that was not found
        entity_type: String,
        /// The ID of the entity that was not found
        id: String,
    },

    /// Validation error
    #[error("Validation error: {message}")]
    ValidationError {
        /// The validation error message
        message: String,
    },

    /// Constraint violation (e.g., foreign key, unique constraint)
    #[error("Constraint violation: {message}")]
    ConstraintViolation {
        /// The constraint violation message
        message: String,
    },

    /// Transaction error
    #[error("Transaction error: {message}")]
    TransactionError {
        /// The transaction error message
        message: String,
    },

    /// Connection or I/O error
    #[error("Connection error: {message}")]
    ConnectionError {
        /// The connection error message
        message: String,
    },

    /// Internal datastore error
    #[error("Internal error: {message}")]
    InternalError {
        /// The internal error message
        message: String,
    },

    /// Operation timeout
    #[error("Operation timeout after {seconds} seconds")]
    Timeout {
        /// The number of seconds before timeout occurred
        seconds: u64,
    },

    /// Unsupported operation
    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation {
        /// The name of the unsupported operation
        operation: String,
    },
}

/// Result type for datastore operations
pub type DataStoreResult<T> = Result<T, DataStoreError>;

/// Query filter for searching entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Field name to filter on
    pub field: String,
    /// Filter operation
    pub operation: FilterOperation,
    /// Value to compare against
    pub value: FilterValue,
}

/// Filter operations supported by the datastore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperation {
    /// Exact match
    Equals,
    /// Not equal
    NotEquals,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// String contains (case-insensitive)
    Contains,
    /// String starts with
    StartsWith,
    /// String ends with
    EndsWith,
    /// Value is in list
    In,
    /// Value is not in list
    NotIn,
    /// Value is null
    IsNull,
    /// Value is not null
    IsNotNull,
}

/// Filter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// UUID value
    Uuid(Uuid),
    /// List of string values
    StringList(Vec<String>),
    /// List of integer values
    IntegerList(Vec<i64>),
    /// List of UUID values
    UuidList(Vec<Uuid>),
    /// Null value
    Null,
}

/// Sorting specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    /// Field name to sort by
    pub field: String,
    /// Sort direction
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending order
    Ascending,
    /// Descending order
    Descending,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Number of items per page (1-1000)
    pub limit: usize,
    /// Number of items to skip
    pub offset: usize,
}

impl Pagination {
    /// Creates a new pagination with validation
    pub fn new(limit: usize, offset: usize) -> DataStoreResult<Self> {
        if limit == 0 || limit > 1000 {
            return Err(DataStoreError::ValidationError {
                message: "Limit must be between 1 and 1000".to_string(),
            });
        }

        Ok(Self { limit, offset })
    }

    /// Creates pagination for a specific page
    pub fn page(page: usize, page_size: usize) -> DataStoreResult<Self> {
        if page == 0 {
            return Err(DataStoreError::ValidationError {
                message: "Page must be greater than 0".to_string(),
            });
        }

        let offset = (page - 1) * page_size;
        Self::new(page_size, offset)
    }
}

/// REST-based DataStore implementation for remote server interaction
pub mod rest {
    use super::*;
    use reqwest::Client;

    /// Simple REST client DataStore
    pub struct RestDataStore {
        client: Client,
        base_url: String,
    }

    impl RestDataStore {
        /// Create a new RestDataStore with the given base URL
        pub fn new(base_url: &str) -> Self {
            Self {
                client: Client::new(),
                base_url: base_url.trim_end_matches('/').to_string(),
            }
        }

        fn endpoint(&self, path: &str) -> String {
            format!("{}/{}", self.base_url, path.trim_start_matches('/'))
        }
    }

    #[async_trait]
    impl DataStore for RestDataStore {
        fn name(&self) -> &'static str {
            "REST"
        }

        async fn health_check(&self) -> DataStoreResult<()> {
            let url = self.endpoint("health");
            let resp =
                self.client
                    .get(url)
                    .send()
                    .await
                    .map_err(|e| DataStoreError::ConnectionError {
                        message: e.to_string(),
                    })?;
            if resp.status().is_success() {
                Ok(())
            } else {
                Err(DataStoreError::ConnectionError {
                    message: resp.status().to_string(),
                })
            }
        }

        async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "begin_transaction".to_string(),
            })
        }

        async fn create_node(&self, node: &Node) -> DataStoreResult<Node> {
            let url = self.endpoint("api/v1/nodes");
            let resp = self.client.post(url).json(node).send().await.map_err(|e| {
                DataStoreError::ConnectionError {
                    message: e.to_string(),
                }
            })?;
            if resp.status().is_success() {
                let api: ServerApiResponse<Node> =
                    resp.json()
                        .await
                        .map_err(|e| DataStoreError::InternalError {
                            message: e.to_string(),
                        })?;
                Ok(api.data)
            } else {
                Err(DataStoreError::InternalError {
                    message: resp.status().to_string(),
                })
            }
        }

        async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>> {
            let url = self.endpoint(&format!("api/v1/nodes/{}", id));
            let resp =
                self.client
                    .get(url)
                    .send()
                    .await
                    .map_err(|e| DataStoreError::ConnectionError {
                        message: e.to_string(),
                    })?;
            if resp.status().is_success() {
                let api: ServerApiResponse<Node> =
                    resp.json()
                        .await
                        .map_err(|e| DataStoreError::InternalError {
                            message: e.to_string(),
                        })?;
                Ok(Some(api.data))
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                Ok(None)
            } else {
                Err(DataStoreError::InternalError {
                    message: resp.status().to_string(),
                })
            }
        }

        async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> {
            let url = self.endpoint("api/v1/nodes");
            let mut req = self.client.get(url);

            if let Some(p) = &options.pagination {
                let page = (p.offset / p.limit) + 1;
                req = req.query(&[("page", page as u64), ("per_page", p.limit as u64)]);
            }

            for filter in &options.filters {
                if let FilterValue::String(val) = &filter.value {
                    req = req.query(&[(filter.field.as_str(), val)]);
                }
            }

            let resp = req
                .send()
                .await
                .map_err(|e| DataStoreError::ConnectionError {
                    message: e.to_string(),
                })?;

            if !resp.status().is_success() {
                return Err(DataStoreError::InternalError {
                    message: resp.status().to_string(),
                });
            }

            let api: ServerApiResponse<ServerPaginatedResponse<Node>> =
                resp.json()
                    .await
                    .map_err(|e| DataStoreError::InternalError {
                        message: e.to_string(),
                    })?;

            Ok(PagedResult {
                items: api.data.data,
                total_count: api.data.total as usize,
                page_size: api.data.per_page as usize,
                page: api.data.page as usize,
                total_pages: api.data.total_pages as usize,
                has_next: api.data.has_next,
                has_previous: api.data.has_prev,
            })
        }

        async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
            let url = self.endpoint(&format!("api/v1/nodes/{}", node.id));
            let resp = self.client.put(url).json(node).send().await.map_err(|e| {
                DataStoreError::ConnectionError {
                    message: e.to_string(),
                }
            })?;
            if resp.status().is_success() {
                let api: ServerApiResponse<Node> =
                    resp.json()
                        .await
                        .map_err(|e| DataStoreError::InternalError {
                            message: e.to_string(),
                        })?;
                Ok(api.data)
            } else {
                Err(DataStoreError::InternalError {
                    message: resp.status().to_string(),
                })
            }
        }

        async fn delete_node(&self, id: &Uuid) -> DataStoreResult<()> {
            let url = self.endpoint(&format!("api/v1/nodes/{}", id));
            let resp = self.client.delete(url).send().await.map_err(|e| {
                DataStoreError::ConnectionError {
                    message: e.to_string(),
                }
            })?;
            if resp.status().is_success() {
                Ok(())
            } else {
                Err(DataStoreError::InternalError {
                    message: resp.status().to_string(),
                })
            }
        }

        async fn get_nodes_by_location(&self, _location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_nodes_by_location".to_string(),
            })
        }

        async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<Node>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "search_nodes_by_name".to_string(),
            })
        }

        async fn create_link(&self, _link: &Link) -> DataStoreResult<Link> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "create_link".to_string(),
            })
        }

        async fn get_link(&self, _id: &Uuid) -> DataStoreResult<Option<Link>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_link".to_string(),
            })
        }

        async fn list_links(&self, _options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "list_links".to_string(),
            })
        }

        async fn update_link(&self, _link: &Link) -> DataStoreResult<Link> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "update_link".to_string(),
            })
        }

        async fn delete_link(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "delete_link".to_string(),
            })
        }

        async fn get_links_for_node(&self, _node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_links_for_node".to_string(),
            })
        }

        async fn get_links_between_nodes(
            &self,
            _first_node_id: &Uuid,
            _second_node_id: &Uuid,
        ) -> DataStoreResult<Vec<Link>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_links_between_nodes".to_string(),
            })
        }

        async fn create_location(&self, _location: &Location) -> DataStoreResult<Location> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "create_location".to_string(),
            })
        }

        async fn get_location(&self, _id: &Uuid) -> DataStoreResult<Option<Location>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_location".to_string(),
            })
        }

        async fn list_locations(
            &self,
            _options: &QueryOptions,
        ) -> DataStoreResult<PagedResult<Location>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "list_locations".to_string(),
            })
        }

        async fn update_location(&self, _location: &Location) -> DataStoreResult<Location> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "update_location".to_string(),
            })
        }

        async fn delete_location(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "delete_location".to_string(),
            })
        }

        async fn get_child_locations(&self, _parent_id: &Uuid) -> DataStoreResult<Vec<Location>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_child_locations".to_string(),
            })
        }

        async fn get_location_tree(&self) -> DataStoreResult<Vec<Location>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_location_tree".to_string(),
            })
        }

        async fn validate_location_hierarchy(
            &self,
            _child_id: &Uuid,
            _new_parent_id: &Uuid,
        ) -> DataStoreResult<()> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "validate_location_hierarchy".to_string(),
            })
        }

        async fn batch_nodes(
            &self,
            _operations: &[BatchOperation<Node>],
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "batch_nodes".to_string(),
            })
        }

        async fn batch_links(
            &self,
            _operations: &[BatchOperation<Link>],
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "batch_links".to_string(),
            })
        }

        async fn batch_locations(
            &self,
            _operations: &[BatchOperation<Location>],
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "batch_locations".to_string(),
            })
        }

        async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_entity_counts".to_string(),
            })
        }

        async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_statistics".to_string(),
            })
        }

        // The remaining trait methods are not currently supported over REST
    }

    #[derive(Deserialize)]
    struct ServerApiResponse<T> {
        data: T,
        success: bool,
        #[serde(default)]
        message: Option<String>,
    }

    #[derive(Deserialize)]
    struct ServerPaginatedResponse<T> {
        data: Vec<T>,
        total: u64,
        page: u64,
        per_page: u64,
        total_pages: u64,
        has_next: bool,
        has_prev: bool,
    }
}

/// Query parameters for list operations
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Filters to apply
    pub filters: Vec<Filter>,
    /// Sorting specification
    pub sort: Vec<Sort>,
    /// Pagination parameters
    pub pagination: Option<Pagination>,
}

/// Result of a paginated query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    /// Items in this page
    pub items: Vec<T>,
    /// Total number of items matching the query
    pub total_count: usize,
    /// Number of items per page
    pub page_size: usize,
    /// Current page number (1-based)
    pub page: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Whether there are more pages
    pub has_next: bool,
    /// Whether there are previous pages
    pub has_previous: bool,
}

impl<T> PagedResult<T> {
    /// Creates a new paged result
    pub fn new(items: Vec<T>, total_count: usize, pagination: Option<&Pagination>) -> Self {
        let (page_size, page, total_pages, has_next, has_previous) = match pagination {
            Some(p) => {
                let page = (p.offset / p.limit) + 1;
                let total_pages = (total_count + p.limit - 1) / p.limit;
                let has_next = page < total_pages;
                let has_previous = page > 1;
                (p.limit, page, total_pages, has_next, has_previous)
            }
            None => (total_count, 1, 1, false, false),
        };

        Self {
            items,
            total_count,
            page_size,
            page,
            total_pages,
            has_next,
            has_previous,
        }
    }
}

/// Transaction handle for batch operations
#[async_trait]
pub trait Transaction: Send + Sync {
    /// Commits the transaction
    async fn commit(self: Box<Self>) -> DataStoreResult<()>;

    /// Rolls back the transaction
    async fn rollback(self: Box<Self>) -> DataStoreResult<()>;
}

/// Batch operation for efficient bulk operations
#[derive(Debug, Clone)]
pub enum BatchOperation<T> {
    /// Insert entity
    Insert(T),
    /// Update entity
    Update(T),
    /// Delete entity by ID
    Delete(Uuid),
}

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub error_count: usize,
    /// Errors that occurred during batch operation
    pub errors: Vec<(usize, DataStoreError)>,
}

/// Main DataStore trait for abstracting data access
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Returns the name/type of this datastore implementation
    fn name(&self) -> &'static str;

    /// Checks if the datastore is healthy and can serve requests
    async fn health_check(&self) -> DataStoreResult<()>;

    /// Begins a new transaction
    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>>;

    // Node operations
    /// Creates a new node
    async fn create_node(&self, node: &Node) -> DataStoreResult<Node>;

    /// Gets a node by ID
    async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>>;

    /// Gets a node by ID, returning an error if not found
    async fn get_node_required(&self, id: &Uuid) -> DataStoreResult<Node> {
        match self.get_node(id).await? {
            Some(node) => Ok(node),
            None => Err(DataStoreError::NotFound {
                entity_type: "Node".to_string(),
                id: id.to_string(),
            }),
        }
    }

    /// Lists nodes with optional filtering, sorting, and pagination
    async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>>;

    /// Updates an existing node
    async fn update_node(&self, node: &Node) -> DataStoreResult<Node>;

    /// Deletes a node by ID
    async fn delete_node(&self, id: &Uuid) -> DataStoreResult<()>;

    /// Gets nodes by location ID
    async fn get_nodes_by_location(&self, location_id: &Uuid) -> DataStoreResult<Vec<Node>>;

    /// Searches nodes by name (case-insensitive partial match)
    async fn search_nodes_by_name(&self, name: &str) -> DataStoreResult<Vec<Node>>;

    // Link operations
    /// Creates a new link
    async fn create_link(&self, link: &Link) -> DataStoreResult<Link>;

    /// Gets a link by ID
    async fn get_link(&self, id: &Uuid) -> DataStoreResult<Option<Link>>;

    /// Gets a link by ID, returning an error if not found
    async fn get_link_required(&self, id: &Uuid) -> DataStoreResult<Link> {
        match self.get_link(id).await? {
            Some(link) => Ok(link),
            None => Err(DataStoreError::NotFound {
                entity_type: "Link".to_string(),
                id: id.to_string(),
            }),
        }
    }

    /// Lists links with optional filtering, sorting, and pagination
    async fn list_links(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Link>>;

    /// Updates an existing link
    async fn update_link(&self, link: &Link) -> DataStoreResult<Link>;

    /// Deletes a link by ID
    async fn delete_link(&self, id: &Uuid) -> DataStoreResult<()>;

    /// Gets links involving a specific node
    async fn get_links_for_node(&self, node_id: &Uuid) -> DataStoreResult<Vec<Link>>;

    /// Gets links between two specific nodes
    async fn get_links_between_nodes(
        &self,
        first_node_id: &Uuid,
        second_node_id: &Uuid,
    ) -> DataStoreResult<Vec<Link>>;

    // Location operations
    /// Creates a new location
    async fn create_location(&self, location: &Location) -> DataStoreResult<Location>;

    /// Gets a location by ID
    async fn get_location(&self, id: &Uuid) -> DataStoreResult<Option<Location>>;

    /// Gets a location by ID, returning an error if not found
    async fn get_location_required(&self, id: &Uuid) -> DataStoreResult<Location> {
        match self.get_location(id).await? {
            Some(location) => Ok(location),
            None => Err(DataStoreError::NotFound {
                entity_type: "Location".to_string(),
                id: id.to_string(),
            }),
        }
    }

    /// Lists locations with optional filtering, sorting, and pagination
    async fn list_locations(
        &self,
        options: &QueryOptions,
    ) -> DataStoreResult<PagedResult<Location>>;

    /// Updates an existing location
    async fn update_location(&self, location: &Location) -> DataStoreResult<Location>;

    /// Deletes a location by ID
    async fn delete_location(&self, id: &Uuid) -> DataStoreResult<()>;

    /// Gets child locations of a parent location
    async fn get_child_locations(&self, parent_id: &Uuid) -> DataStoreResult<Vec<Location>>;

    /// Gets the full location hierarchy as a tree
    async fn get_location_tree(&self) -> DataStoreResult<Vec<Location>>;

    /// Validates that a parent-child relationship doesn't create cycles
    async fn validate_location_hierarchy(
        &self,
        child_id: &Uuid,
        new_parent_id: &Uuid,
    ) -> DataStoreResult<()>;

    // Batch operations
    /// Performs batch operations on nodes
    async fn batch_nodes(
        &self,
        operations: &[BatchOperation<Node>],
    ) -> DataStoreResult<BatchResult>;

    /// Performs batch operations on links
    async fn batch_links(
        &self,
        operations: &[BatchOperation<Link>],
    ) -> DataStoreResult<BatchResult>;

    /// Performs batch operations on locations
    async fn batch_locations(
        &self,
        operations: &[BatchOperation<Location>],
    ) -> DataStoreResult<BatchResult>;

    // Statistics and metadata
    /// Gets count of all entities
    async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>>;

    /// Gets datastore statistics (implementation-specific)
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
        _node_id: &Uuid, // Trait method signature requires this parameter
    ) -> DataStoreResult<Vec<crate::models::derived::InterfaceStatus>> {
        // Default implementation returns empty list
        Ok(Vec::new())
    }

    /// Gets performance metrics for a specific node
    async fn get_node_metrics(
        &self,
        _node_id: &Uuid, // Trait method signature requires this parameter
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
        let _ = (node_id, rule_id, result);
        Ok(())
    }

    /// Gets policy execution results for a node
    async fn get_policy_results(
        &self,
        node_id: &Uuid,
    ) -> DataStoreResult<Vec<PolicyExecutionResult>> {
        // Default implementation returns empty results
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

    // Template operations
    /// Creates a new template
    async fn create_template(&self, _template: &Template) -> DataStoreResult<Template> {
        Err(DataStoreError::UnsupportedOperation {
            operation: "create_template not implemented".to_string(),
        })
    }

    /// Gets a template by ID
    async fn get_template(&self, id: &Uuid) -> DataStoreResult<Option<Template>> {
        let _ = id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template not implemented".to_string(),
        })
    }

    /// Gets a template by ID, returning an error if not found
    async fn get_template_required(&self, id: &Uuid) -> DataStoreResult<Template> {
        match self.get_template(id).await? {
            Some(template) => Ok(template),
            None => Err(DataStoreError::NotFound {
                entity_type: "Template".to_string(),
                id: id.to_string(),
            }),
        }
    }

    /// Lists templates with optional filtering, sorting, and pagination
    async fn list_templates(
        &self,
        options: &QueryOptions,
    ) -> DataStoreResult<PagedResult<Template>> {
        let _ = options;
        Err(DataStoreError::UnsupportedOperation {
            operation: "list_templates not implemented".to_string(),
        })
    }

    /// Updates an existing template
    async fn update_template(&self, template: &Template) -> DataStoreResult<Template> {
        let _ = template;
        Err(DataStoreError::UnsupportedOperation {
            operation: "update_template not implemented".to_string(),
        })
    }

    /// Deletes a template by ID
    async fn delete_template(&self, id: &Uuid) -> DataStoreResult<()> {
        let _ = id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "delete_template not implemented".to_string(),
        })
    }

    /// Gets templates by vendor and type
    async fn get_templates_by_vendor_and_type(
        &self,
        vendor: &str,
        template_type: &str,
    ) -> DataStoreResult<Vec<Template>> {
        let _ = (vendor, template_type);
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_templates_by_vendor_and_type not implemented".to_string(),
        })
    }

    // Template assignment operations
    /// Creates a new template assignment
    async fn create_template_assignment(
        &self,
        assignment: &TemplateAssignment,
    ) -> DataStoreResult<TemplateAssignment> {
        let _ = assignment;
        Err(DataStoreError::UnsupportedOperation {
            operation: "create_template_assignment not implemented".to_string(),
        })
    }

    /// Gets a template assignment by ID
    async fn get_template_assignment(
        &self,
        id: &Uuid,
    ) -> DataStoreResult<Option<TemplateAssignment>> {
        let _ = id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template_assignment not implemented".to_string(),
        })
    }

    /// Lists template assignments for a node
    async fn get_template_assignments_for_node(
        &self,
        node_id: &Uuid,
    ) -> DataStoreResult<Vec<TemplateAssignment>> {
        let _ = node_id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template_assignments_for_node not implemented".to_string(),
        })
    }

    /// Lists template assignments for a template
    async fn get_template_assignments_for_template(
        &self,
        template_id: &Uuid,
    ) -> DataStoreResult<Vec<TemplateAssignment>> {
        let _ = template_id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template_assignments_for_template not implemented".to_string(),
        })
    }

    /// Updates a template assignment
    async fn update_template_assignment(
        &self,
        assignment: &TemplateAssignment,
    ) -> DataStoreResult<TemplateAssignment> {
        let _ = assignment;
        Err(DataStoreError::UnsupportedOperation {
            operation: "update_template_assignment not implemented".to_string(),
        })
    }

    /// Deletes a template assignment by ID
    async fn delete_template_assignment(&self, id: &Uuid) -> DataStoreResult<()> {
        let _ = id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "delete_template_assignment not implemented".to_string(),
        })
    }

    // Template version operations
    /// Creates a new template version
    async fn create_template_version(
        &self,
        version: &TemplateVersion,
    ) -> DataStoreResult<TemplateVersion> {
        let _ = version;
        Err(DataStoreError::UnsupportedOperation {
            operation: "create_template_version not implemented".to_string(),
        })
    }

    /// Gets template versions for a template
    async fn get_template_versions(
        &self,
        template_id: &Uuid,
    ) -> DataStoreResult<Vec<TemplateVersion>> {
        let _ = template_id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template_versions not implemented".to_string(),
        })
    }

    /// Gets a specific template version
    async fn get_template_version(
        &self,
        template_id: &Uuid,
        version: &str,
    ) -> DataStoreResult<Option<TemplateVersion>> {
        let _ = (template_id, version);
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template_version not implemented".to_string(),
        })
    }

    /// Gets the latest template version
    async fn get_latest_template_version(
        &self,
        template_id: &Uuid,
    ) -> DataStoreResult<Option<TemplateVersion>> {
        let _ = template_id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_latest_template_version not implemented".to_string(),
        })
    }

    /// Deletes a template version
    async fn delete_template_version(&self, id: &Uuid) -> DataStoreResult<()> {
        let _ = id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "delete_template_version not implemented".to_string(),
        })
    }

    // Template usage operations
    /// Records template usage
    async fn record_template_usage(&self, usage: &TemplateUsage) -> DataStoreResult<TemplateUsage> {
        let _ = usage;
        Err(DataStoreError::UnsupportedOperation {
            operation: "record_template_usage not implemented".to_string(),
        })
    }

    /// Gets template usage analytics
    async fn get_template_usage(
        &self,
        template_id: &Uuid,
        limit: Option<usize>,
    ) -> DataStoreResult<Vec<TemplateUsage>> {
        let _ = (template_id, limit);
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template_usage not implemented".to_string(),
        })
    }

    /// Gets template usage statistics
    async fn get_template_usage_stats(
        &self,
        template_id: &Uuid,
    ) -> DataStoreResult<HashMap<String, serde_json::Value>> {
        let _ = template_id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_template_usage_stats not implemented".to_string(),
        })
    }

    // Change tracking operations
    /// Creates a new configuration change
    async fn create_configuration_change(
        &self,
        change: &crate::models::change_tracking::ConfigurationChange,
    ) -> DataStoreResult<crate::models::change_tracking::ConfigurationChange> {
        let _ = change;
        Err(DataStoreError::UnsupportedOperation {
            operation: "create_configuration_change not implemented".to_string(),
        })
    }

    /// Gets a configuration change by ID
    async fn get_configuration_change(
        &self,
        id: &str,
    ) -> DataStoreResult<Option<crate::models::change_tracking::ConfigurationChange>> {
        let _ = id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_configuration_change not implemented".to_string(),
        })
    }

    /// Updates an existing configuration change
    async fn update_configuration_change(
        &self,
        change: &crate::models::change_tracking::ConfigurationChange,
    ) -> DataStoreResult<crate::models::change_tracking::ConfigurationChange> {
        let _ = change;
        Err(DataStoreError::UnsupportedOperation {
            operation: "update_configuration_change not implemented".to_string(),
        })
    }

    /// Lists configuration changes for a specific entity
    async fn get_configuration_changes_for_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> DataStoreResult<Vec<crate::models::change_tracking::ConfigurationChange>> {
        let _ = (entity_type, entity_id);
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_configuration_changes_for_entity not implemented".to_string(),
        })
    }

    /// Lists configuration changes by status
    async fn get_configuration_changes_by_status(
        &self,
        status: crate::models::change_tracking::ChangeStatus,
    ) -> DataStoreResult<Vec<crate::models::change_tracking::ConfigurationChange>> {
        let _ = status;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_configuration_changes_by_status not implemented".to_string(),
        })
    }

    /// Lists all configuration changes with optional filtering and pagination
    async fn list_configuration_changes(
        &self,
        options: &QueryOptions,
    ) -> DataStoreResult<PagedResult<crate::models::change_tracking::ConfigurationChange>> {
        let _ = options;
        Err(DataStoreError::UnsupportedOperation {
            operation: "list_configuration_changes not implemented".to_string(),
        })
    }

    /// Creates a new audit log entry
    async fn create_audit_log_entry(
        &self,
        entry: &crate::models::change_tracking::ChangeAuditLog,
    ) -> DataStoreResult<crate::models::change_tracking::ChangeAuditLog> {
        let _ = entry;
        Err(DataStoreError::UnsupportedOperation {
            operation: "create_audit_log_entry not implemented".to_string(),
        })
    }

    /// Gets audit log entries for a specific change
    async fn get_audit_log_entries_for_change(
        &self,
        change_id: &str,
    ) -> DataStoreResult<Vec<crate::models::change_tracking::ChangeAuditLog>> {
        let _ = change_id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_audit_log_entries_for_change not implemented".to_string(),
        })
    }

    /// Lists audit log entries with optional filtering and pagination
    async fn list_audit_log_entries(
        &self,
        options: &QueryOptions,
    ) -> DataStoreResult<PagedResult<crate::models::change_tracking::ChangeAuditLog>> {
        let _ = options;
        Err(DataStoreError::UnsupportedOperation {
            operation: "list_audit_log_entries not implemented".to_string(),
        })
    }

    /// Creates a new approval workflow
    async fn create_approval_workflow(
        &self,
        workflow: &crate::models::change_tracking::ChangeApprovalWorkflow,
    ) -> DataStoreResult<crate::models::change_tracking::ChangeApprovalWorkflow> {
        let _ = workflow;
        Err(DataStoreError::UnsupportedOperation {
            operation: "create_approval_workflow not implemented".to_string(),
        })
    }

    /// Gets an approval workflow for a specific change
    async fn get_approval_workflow_for_change(
        &self,
        change_id: &str,
    ) -> DataStoreResult<Option<crate::models::change_tracking::ChangeApprovalWorkflow>> {
        let _ = change_id;
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_approval_workflow_for_change not implemented".to_string(),
        })
    }

    /// Updates an existing approval workflow
    async fn update_approval_workflow(
        &self,
        workflow: &crate::models::change_tracking::ChangeApprovalWorkflow,
    ) -> DataStoreResult<crate::models::change_tracking::ChangeApprovalWorkflow> {
        let _ = workflow;
        Err(DataStoreError::UnsupportedOperation {
            operation: "update_approval_workflow not implemented".to_string(),
        })
    }

    /// Creates a new rollback snapshot
    async fn create_rollback_snapshot(
        &self,
        snapshot: &crate::models::change_tracking::ChangeRollbackSnapshot,
    ) -> DataStoreResult<crate::models::change_tracking::ChangeRollbackSnapshot> {
        let _ = snapshot;
        Err(DataStoreError::UnsupportedOperation {
            operation: "create_rollback_snapshot not implemented".to_string(),
        })
    }

    /// Gets a rollback snapshot for a specific change and type
    async fn get_rollback_snapshot_for_change(
        &self,
        change_id: &str,
        snapshot_type: crate::models::change_tracking::SnapshotType,
    ) -> DataStoreResult<Option<crate::models::change_tracking::ChangeRollbackSnapshot>> {
        let _ = (change_id, snapshot_type);
        Err(DataStoreError::UnsupportedOperation {
            operation: "get_rollback_snapshot_for_change not implemented".to_string(),
        })
    }

    /// Lists rollback snapshots for a specific entity
    async fn list_rollback_snapshots_for_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> DataStoreResult<Vec<crate::models::change_tracking::ChangeRollbackSnapshot>> {
        let _ = (entity_type, entity_id);
        Err(DataStoreError::UnsupportedOperation {
            operation: "list_rollback_snapshots_for_entity not implemented".to_string(),
        })
    }
}

// Helper functions for creating common query options

/// Creates a filter for exact string match
pub fn filter_equals_string(field: &str, value: &str) -> Filter {
    Filter {
        field: field.to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::String(value.to_string()),
    }
}

/// Creates a filter for UUID match
pub fn filter_equals_uuid(field: &str, value: Uuid) -> Filter {
    Filter {
        field: field.to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(value),
    }
}

/// Creates a filter for string contains (case-insensitive)
pub fn filter_contains(field: &str, value: &str) -> Filter {
    Filter {
        field: field.to_string(),
        operation: FilterOperation::Contains,
        value: FilterValue::String(value.to_string()),
    }
}

/// Creates ascending sort by field
pub fn sort_asc(field: &str) -> Sort {
    Sort {
        field: field.to_string(),
        direction: SortDirection::Ascending,
    }
}

/// Creates descending sort by field
pub fn sort_desc(field: &str) -> Sort {
    Sort {
        field: field.to_string(),
        direction: SortDirection::Descending,
    }
}

/// Simple CSV-based DataStore implementation for demo and testing
pub mod csv {
    use super::*;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use tokio::fs;
    use tokio::sync::Mutex;

    /// CSV-based DataStore implementation
    pub struct CsvStore {
        base_path: PathBuf,
        data: Arc<Mutex<CsvData>>,
    }

    #[derive(Debug, Default)]
    struct CsvData {
        nodes: HashMap<Uuid, Node>,
        links: HashMap<Uuid, Link>,
        locations: HashMap<Uuid, Location>,
    }

    /// Simple transaction implementation for CSV store
    pub struct CsvTransaction {
        store: Arc<CsvStore>,
        changes: Mutex<CsvData>,
        committed: Mutex<bool>,
    }

    impl CsvStore {
        /// Creates a new CSV store with the given base directory
        pub async fn new<P: AsRef<Path>>(base_path: P) -> DataStoreResult<Self> {
            let base_path = base_path.as_ref().to_path_buf();

            // Create directory if it doesn't exist
            if let Some(parent) = base_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| DataStoreError::ConnectionError {
                        message: format!("Failed to create directory: {}", e),
                    })?;
            }

            let store = Self {
                base_path,
                data: Arc::new(Mutex::new(CsvData::default())),
            };

            // Load existing data
            store.load_data().await?;

            Ok(store)
        }

        /// Loads data from CSV files
        async fn load_data(&self) -> DataStoreResult<()> {
            // For simplicity, we'll use JSON files instead of CSV for demo
            // In a real implementation, you'd use a CSV library
            let nodes_file = self.base_path.join("nodes.json");
            let links_file = self.base_path.join("links.json");
            let locations_file = self.base_path.join("locations.json");

            let mut data = self.data.lock().await;

            // Load nodes
            if nodes_file.exists() {
                let content = fs::read_to_string(&nodes_file).await.map_err(|e| {
                    DataStoreError::ConnectionError {
                        message: format!("Failed to read nodes file: {}", e),
                    }
                })?;
                let nodes: Vec<Node> =
                    serde_json::from_str(&content).map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to parse nodes: {}", e),
                    })?;
                for node in nodes {
                    data.nodes.insert(node.id, node);
                }
            }

            // Load links
            if links_file.exists() {
                let content = fs::read_to_string(&links_file).await.map_err(|e| {
                    DataStoreError::ConnectionError {
                        message: format!("Failed to read links file: {}", e),
                    }
                })?;
                let links: Vec<Link> =
                    serde_json::from_str(&content).map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to parse links: {}", e),
                    })?;
                for link in links {
                    data.links.insert(link.id, link);
                }
            }

            // Load locations
            if locations_file.exists() {
                let content = fs::read_to_string(&locations_file).await.map_err(|e| {
                    DataStoreError::ConnectionError {
                        message: format!("Failed to read locations file: {}", e),
                    }
                })?;
                let locations: Vec<Location> =
                    serde_json::from_str(&content).map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to parse locations: {}", e),
                    })?;
                for location in locations {
                    data.locations.insert(location.id, location);
                }
            }

            Ok(())
        }

        /// Saves data to CSV files
        async fn save_data(&self) -> DataStoreResult<()> {
            let data = self.data.lock().await;

            // Save nodes
            let nodes: Vec<&Node> = data.nodes.values().collect();
            let nodes_content = serde_json::to_string_pretty(&nodes).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to serialize nodes: {}", e),
                }
            })?;
            fs::write(self.base_path.join("nodes.json"), nodes_content)
                .await
                .map_err(|e| DataStoreError::ConnectionError {
                    message: format!("Failed to write nodes file: {}", e),
                })?;

            // Save links
            let links: Vec<&Link> = data.links.values().collect();
            let links_content = serde_json::to_string_pretty(&links).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to serialize links: {}", e),
                }
            })?;
            fs::write(self.base_path.join("links.json"), links_content)
                .await
                .map_err(|e| DataStoreError::ConnectionError {
                    message: format!("Failed to write links file: {}", e),
                })?;

            // Save locations
            let locations: Vec<&Location> = data.locations.values().collect();
            let locations_content = serde_json::to_string_pretty(&locations).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to serialize locations: {}", e),
                }
            })?;
            fs::write(self.base_path.join("locations.json"), locations_content)
                .await
                .map_err(|e| DataStoreError::ConnectionError {
                    message: format!("Failed to write locations file: {}", e),
                })?;

            Ok(())
        }

        /// Applies filters to a collection
        fn apply_filters<T, F>(&self, items: Vec<T>, filters: &[Filter], field_getter: F) -> Vec<T>
        where
            F: Fn(&T, &str) -> Option<FilterValue>,
        {
            if filters.is_empty() {
                return items;
            }

            items
                .into_iter()
                .filter(|item| {
                    filters.iter().all(|filter| {
                        let field_value = field_getter(item, &filter.field);
                        match field_value {
                            Some(value) => self.matches_filter(&value, filter),
                            None => matches!(filter.operation, FilterOperation::IsNull),
                        }
                    })
                })
                .collect()
        }

        /// Checks if a value matches a filter
        fn matches_filter(&self, value: &FilterValue, filter: &Filter) -> bool {
            use FilterOperation::*;
            use FilterValue as FV;

            match (&filter.operation, value, &filter.value) {
                (Equals, FV::String(a), FV::String(b)) => a == b,
                (Equals, FV::Uuid(a), FV::Uuid(b)) => a == b,
                (NotEquals, a, b) => !self.matches_filter(
                    a,
                    &Filter {
                        field: filter.field.clone(),
                        operation: Equals,
                        value: b.clone(),
                    },
                ),
                (Contains, FV::String(a), FV::String(b)) => {
                    a.to_lowercase().contains(&b.to_lowercase())
                }
                (StartsWith, FV::String(a), FV::String(b)) => {
                    a.to_lowercase().starts_with(&b.to_lowercase())
                }
                (EndsWith, FV::String(a), FV::String(b)) => {
                    a.to_lowercase().ends_with(&b.to_lowercase())
                }
                (IsNull, _, _) => false, // Handled in apply_filters
                (IsNotNull, _, _) => true,
                _ => false, // Unsupported filter combinations
            }
        }
    }

    #[async_trait]
    impl Transaction for CsvTransaction {
        async fn commit(self: Box<Self>) -> DataStoreResult<()> {
            let mut committed = self.committed.lock().await;
            if *committed {
                return Err(DataStoreError::TransactionError {
                    message: "Transaction already committed or rolled back".to_string(),
                });
            }

            // Apply changes to the store
            let changes = self.changes.lock().await;
            let mut store_data = self.store.data.lock().await;

            // Merge changes (simplified - in real implementation you'd need proper conflict resolution)
            for (id, node) in &changes.nodes {
                store_data.nodes.insert(*id, node.clone());
            }
            for (id, link) in &changes.links {
                store_data.links.insert(*id, link.clone());
            }
            for (id, location) in &changes.locations {
                store_data.locations.insert(*id, location.clone());
            }

            drop(store_data);
            drop(changes);
            self.store.save_data().await?;
            *committed = true;
            Ok(())
        }

        async fn rollback(self: Box<Self>) -> DataStoreResult<()> {
            let mut committed = self.committed.lock().await;
            if *committed {
                return Err(DataStoreError::TransactionError {
                    message: "Transaction already committed or rolled back".to_string(),
                });
            }
            *committed = true;
            Ok(())
        }
    }

    #[async_trait]
    impl DataStore for CsvStore {
        fn name(&self) -> &'static str {
            "CSV"
        }

        async fn health_check(&self) -> DataStoreResult<()> {
            // Check if base directory is accessible
            if !self.base_path.parent().unwrap_or(&self.base_path).exists() {
                return Err(DataStoreError::ConnectionError {
                    message: "Base directory is not accessible".to_string(),
                });
            }
            Ok(())
        }

        async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> {
            Ok(Box::new(CsvTransaction {
                store: Arc::new(CsvStore {
                    base_path: self.base_path.clone(),
                    data: Arc::clone(&self.data),
                }),
                changes: Mutex::new(CsvData::default()),
                committed: Mutex::new(false),
            }))
        }

        // Node operations
        async fn create_node(&self, node: &Node) -> DataStoreResult<Node> {
            node.validate()
                .map_err(|e| DataStoreError::ValidationError { message: e })?;

            {
                let mut data = self.data.lock().await;
                if data.nodes.contains_key(&node.id) {
                    return Err(DataStoreError::ConstraintViolation {
                        message: format!("Node with ID {} already exists", node.id),
                    });
                }
                data.nodes.insert(node.id, node.clone());
            }

            self.save_data().await?;
            Ok(node.clone())
        }

        async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>> {
            let data = self.data.lock().await;
            Ok(data.nodes.get(id).cloned())
        }

        async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> {
            let data = self.data.lock().await;
            let mut nodes: Vec<Node> = data.nodes.values().cloned().collect();
            drop(data); // Release lock early

            // Apply filters (simplified)
            nodes = self.apply_filters(nodes, &options.filters, |node, field| match field {
                "name" => Some(FilterValue::String(node.name.clone())),
                "vendor" => Some(FilterValue::String(node.vendor.to_string())),
                "role" => Some(FilterValue::String(node.role.to_string())),
                "lifecycle" => Some(FilterValue::String(node.lifecycle.to_string())),
                "location_id" => node.location_id.map(FilterValue::Uuid),
                _ => None,
            });

            // Apply sorting (simplified)
            if let Some(sort) = options.sort.first() {
                nodes.sort_by(|a, b| {
                    let ordering = match sort.field.as_str() {
                        "name" => a.name.cmp(&b.name),
                        _ => std::cmp::Ordering::Equal,
                    };
                    match sort.direction {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });
            }

            let total_count = nodes.len();

            // Apply pagination
            if let Some(pagination) = &options.pagination {
                let start = pagination.offset.min(nodes.len());
                let end = (pagination.offset + pagination.limit).min(nodes.len());
                nodes = nodes[start..end].to_vec();
            }

            Ok(PagedResult::new(
                nodes,
                total_count,
                options.pagination.as_ref(),
            ))
        }

        async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
            node.validate()
                .map_err(|e| DataStoreError::ValidationError { message: e })?;

            {
                let mut data = self.data.lock().await;
                if !data.nodes.contains_key(&node.id) {
                    return Err(DataStoreError::NotFound {
                        entity_type: "Node".to_string(),
                        id: node.id.to_string(),
                    });
                }
                data.nodes.insert(node.id, node.clone());
            }

            self.save_data().await?;
            Ok(node.clone())
        }

        async fn delete_node(&self, id: &Uuid) -> DataStoreResult<()> {
            {
                let mut data = self.data.lock().await;
                if data.nodes.remove(id).is_none() {
                    return Err(DataStoreError::NotFound {
                        entity_type: "Node".to_string(),
                        id: id.to_string(),
                    });
                }
            }

            self.save_data().await
        }

        async fn get_nodes_by_location(&self, location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
            let data = self.data.lock().await;
            Ok(data
                .nodes
                .values()
                .filter(|node| node.location_id == Some(*location_id))
                .cloned()
                .collect())
        }

        async fn search_nodes_by_name(&self, name: &str) -> DataStoreResult<Vec<Node>> {
            let data = self.data.lock().await;
            let name_lower = name.to_lowercase();
            Ok(data
                .nodes
                .values()
                .filter(|node| node.name.to_lowercase().contains(&name_lower))
                .cloned()
                .collect())
        }

        // Link operations (simplified implementations)
        async fn create_link(&self, link: &Link) -> DataStoreResult<Link> {
            link.validate()
                .map_err(|e| DataStoreError::ValidationError { message: e })?;

            let mut data = self.data.lock().await;
            if data.links.contains_key(&link.id) {
                return Err(DataStoreError::ConstraintViolation {
                    message: format!("Link with ID {} already exists", link.id),
                });
            }

            data.links.insert(link.id, link.clone());
            drop(data);
            self.save_data().await?;
            Ok(link.clone())
        }

        async fn get_link(&self, id: &Uuid) -> DataStoreResult<Option<Link>> {
            let data = self.data.lock().await;
            Ok(data.links.get(id).cloned())
        }

        async fn list_links(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> {
            let data = self.data.lock().await;
            let mut links: Vec<Link> = data.links.values().cloned().collect();
            let total_count = links.len();

            // Apply pagination (simplified)
            if let Some(pagination) = &options.pagination {
                let start = pagination.offset.min(links.len());
                let end = (pagination.offset + pagination.limit).min(links.len());
                links = links[start..end].to_vec();
            }

            Ok(PagedResult::new(
                links,
                total_count,
                options.pagination.as_ref(),
            ))
        }

        async fn update_link(&self, link: &Link) -> DataStoreResult<Link> {
            link.validate()
                .map_err(|e| DataStoreError::ValidationError { message: e })?;

            let mut data = self.data.lock().await;
            if !data.links.contains_key(&link.id) {
                return Err(DataStoreError::NotFound {
                    entity_type: "Link".to_string(),
                    id: link.id.to_string(),
                });
            }

            data.links.insert(link.id, link.clone());
            drop(data);
            self.save_data().await?;
            Ok(link.clone())
        }

        async fn delete_link(&self, id: &Uuid) -> DataStoreResult<()> {
            let mut data = self.data.lock().await;
            if data.links.remove(id).is_none() {
                return Err(DataStoreError::NotFound {
                    entity_type: "Link".to_string(),
                    id: id.to_string(),
                });
            }
            drop(data);
            self.save_data().await
        }

        async fn get_links_for_node(&self, node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
            let data = self.data.lock().await;
            Ok(data
                .links
                .values()
                .filter(|link| link.involves_node(*node_id))
                .cloned()
                .collect())
        }

        async fn get_links_between_nodes(
            &self,
            first_node_id: &Uuid,
            second_node_id: &Uuid,
        ) -> DataStoreResult<Vec<Link>> {
            let data = self.data.lock().await;
            Ok(data
                .links
                .values()
                .filter(|link| link.connects_nodes(*first_node_id, *second_node_id))
                .cloned()
                .collect())
        }

        // Location operations (simplified implementations)
        async fn create_location(&self, location: &Location) -> DataStoreResult<Location> {
            location
                .validate()
                .map_err(|e| DataStoreError::ValidationError { message: e })?;

            {
                let mut data = self.data.lock().await;
                if data.locations.contains_key(&location.id) {
                    return Err(DataStoreError::ConstraintViolation {
                        message: format!("Location with ID {} already exists", location.id),
                    });
                }
                data.locations.insert(location.id, location.clone());
            }

            self.save_data().await?;
            Ok(location.clone())
        }

        async fn get_location(&self, id: &Uuid) -> DataStoreResult<Option<Location>> {
            let data = self.data.lock().await;
            Ok(data.locations.get(id).cloned())
        }

        async fn list_locations(
            &self,
            options: &QueryOptions,
        ) -> DataStoreResult<PagedResult<Location>> {
            let data = self.data.lock().await;
            let mut locations: Vec<Location> = data.locations.values().cloned().collect();
            let total_count = locations.len();
            drop(data); // Release lock early

            // Apply pagination (simplified)
            if let Some(pagination) = &options.pagination {
                let start = pagination.offset.min(locations.len());
                let end = (pagination.offset + pagination.limit).min(locations.len());
                locations = locations[start..end].to_vec();
            }

            Ok(PagedResult::new(
                locations,
                total_count,
                options.pagination.as_ref(),
            ))
        }

        async fn update_location(&self, location: &Location) -> DataStoreResult<Location> {
            location
                .validate()
                .map_err(|e| DataStoreError::ValidationError { message: e })?;

            {
                let mut data = self.data.lock().await;
                if !data.locations.contains_key(&location.id) {
                    return Err(DataStoreError::NotFound {
                        entity_type: "Location".to_string(),
                        id: location.id.to_string(),
                    });
                }
                data.locations.insert(location.id, location.clone());
            }

            self.save_data().await?;
            Ok(location.clone())
        }

        async fn delete_location(&self, id: &Uuid) -> DataStoreResult<()> {
            {
                let mut data = self.data.lock().await;
                if data.locations.remove(id).is_none() {
                    return Err(DataStoreError::NotFound {
                        entity_type: "Location".to_string(),
                        id: id.to_string(),
                    });
                }
            }

            self.save_data().await
        }

        async fn get_child_locations(&self, parent_id: &Uuid) -> DataStoreResult<Vec<Location>> {
            let data = self.data.lock().await;
            Ok(data
                .locations
                .values()
                .filter(|location| location.parent_id == Some(*parent_id))
                .cloned()
                .collect())
        }

        async fn get_location_tree(&self) -> DataStoreResult<Vec<Location>> {
            let data = self.data.lock().await;
            Ok(data.locations.values().cloned().collect())
        }

        async fn validate_location_hierarchy(
            &self,
            child_id: &Uuid,
            new_parent_id: &Uuid,
        ) -> DataStoreResult<()> {
            let data = self.data.lock().await;
            let locations: Vec<Location> = data.locations.values().cloned().collect();
            drop(data); // Release lock early

            if Location::detect_circular_reference(&locations, *new_parent_id, *child_id) {
                return Err(DataStoreError::ConstraintViolation {
                    message: "Circular reference detected in location hierarchy".to_string(),
                });
            }

            Ok(())
        }

        // Batch operations (simplified)
        async fn batch_nodes(
            &self,
            operations: &[BatchOperation<Node>],
        ) -> DataStoreResult<BatchResult> {
            let mut success_count = 0;
            let mut errors = Vec::new();

            for (index, operation) in operations.iter().enumerate() {
                let result = match operation {
                    BatchOperation::Insert(node) => self.create_node(node).await.map(|_| ()),
                    BatchOperation::Update(node) => self.update_node(node).await.map(|_| ()),
                    BatchOperation::Delete(id) => self.delete_node(id).await,
                };

                match result {
                    Ok(()) => success_count += 1,
                    Err(e) => errors.push((index, e)),
                }
            }

            Ok(BatchResult {
                success_count,
                error_count: errors.len(),
                errors,
            })
        }

        async fn batch_links(
            &self,
            operations: &[BatchOperation<Link>],
        ) -> DataStoreResult<BatchResult> {
            let mut success_count = 0;
            let mut errors = Vec::new();

            for (index, operation) in operations.iter().enumerate() {
                let result = match operation {
                    BatchOperation::Insert(link) => self.create_link(link).await.map(|_| ()),
                    BatchOperation::Update(link) => self.update_link(link).await.map(|_| ()),
                    BatchOperation::Delete(id) => self.delete_link(id).await,
                };

                match result {
                    Ok(()) => success_count += 1,
                    Err(e) => errors.push((index, e)),
                }
            }

            Ok(BatchResult {
                success_count,
                error_count: errors.len(),
                errors,
            })
        }

        async fn batch_locations(
            &self,
            operations: &[BatchOperation<Location>],
        ) -> DataStoreResult<BatchResult> {
            let mut success_count = 0;
            let mut errors = Vec::new();

            for (index, operation) in operations.iter().enumerate() {
                let result = match operation {
                    BatchOperation::Insert(location) => {
                        self.create_location(location).await.map(|_| ())
                    }
                    BatchOperation::Update(location) => {
                        self.update_location(location).await.map(|_| ())
                    }
                    BatchOperation::Delete(id) => self.delete_location(id).await,
                };

                match result {
                    Ok(()) => success_count += 1,
                    Err(e) => errors.push((index, e)),
                }
            }

            Ok(BatchResult {
                success_count,
                error_count: errors.len(),
                errors,
            })
        }

        async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
            let data = self.data.lock().await;
            let mut counts = HashMap::new();
            counts.insert("nodes".to_string(), data.nodes.len());
            counts.insert("links".to_string(), data.links.len());
            counts.insert("locations".to_string(), data.locations.len());
            Ok(counts)
        }

        async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
            let mut stats = HashMap::new();
            stats.insert(
                "type".to_string(),
                serde_json::Value::String("CSV".to_string()),
            );
            stats.insert(
                "base_path".to_string(),
                serde_json::Value::String(self.base_path.display().to_string()),
            );
            Ok(stats)
        }
    }
}

/// SQLite-based DataStore implementation using SeaORM
pub mod sqlite {
    use super::*;
    use crate::entities::{links, locations, nodes};
    use crate::models::{DeviceRole, Lifecycle, Vendor};
    use chrono::Utc;
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
        DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
        Set, TransactionTrait,
    };
    use std::time::Duration;

    /// SQLite-based DataStore implementation
    pub struct SqliteStore {
        db: DatabaseConnection,
    }

    /// Helper function to convert SeaORM link entity to our Link model
    fn entity_to_link(entity: links::Model) -> DataStoreResult<Link> {
        let id = entity
            .id
            .parse::<Uuid>()
            .map_err(|e| DataStoreError::ValidationError {
                message: format!("Invalid UUID: {}", e),
            })?;

        let node_a_id =
            entity
                .node_a_id
                .parse::<Uuid>()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid node A UUID: {}", e),
                })?;

        let node_b_id =
            if let Some(node_b_id_str) = entity.node_b_id {
                Some(node_b_id_str.parse::<Uuid>().map_err(|e| {
                    DataStoreError::ValidationError {
                        message: format!("Invalid node B UUID: {}", e),
                    }
                })?)
            } else {
                None
            };

        let custom_data = if let Some(ref data_str) = entity.custom_data {
            serde_json::from_str(data_str).unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        Ok(Link {
            id,
            name: entity.name,
            node_a_id,
            node_a_interface: entity.interface_a,
            node_z_id: node_b_id,
            node_z_interface: entity.interface_b,
            description: entity.description,
            bandwidth: entity.capacity.map(|c| c as u64),
            link_type: None, // Not stored in entity yet
            is_internet_circuit: entity.is_internet_circuit != 0,
            custom_data,
        })
    }

    /// Helper function to convert SeaORM location entity to our Location model
    fn entity_to_location(entity: locations::Model) -> DataStoreResult<Location> {
        let id = entity
            .id
            .parse::<Uuid>()
            .map_err(|e| DataStoreError::ValidationError {
                message: format!("Invalid UUID: {}", e),
            })?;

        let parent_id =
            if let Some(parent_id_str) = entity.parent_id {
                Some(parent_id_str.parse::<Uuid>().map_err(|e| {
                    DataStoreError::ValidationError {
                        message: format!("Invalid parent UUID: {}", e),
                    }
                })?)
            } else {
                None
            };

        let custom_data = if let Some(ref data_str) = entity.custom_data {
            serde_json::from_str(data_str).unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        Ok(Location {
            id,
            name: entity.name,
            location_type: entity.location_type,
            parent_id,
            path: entity.path,
            description: entity.description,
            address: entity.address,
            custom_data,
        })
    }

    /// Helper function to convert SeaORM node entity to our Node model
    fn entity_to_node(entity: nodes::Model) -> DataStoreResult<Node> {
        let vendor =
            entity
                .vendor
                .parse::<Vendor>()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid vendor: {}", e),
                })?;

        let role =
            entity
                .role
                .parse::<DeviceRole>()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid role: {}", e),
                })?;

        let lifecycle =
            entity
                .lifecycle
                .parse::<Lifecycle>()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid lifecycle: {}", e),
                })?;

        let id = entity
            .id
            .parse::<Uuid>()
            .map_err(|e| DataStoreError::ValidationError {
                message: format!("Invalid UUID: {}", e),
            })?;

        let location_id = if let Some(loc_id_str) = entity.location_id {
            Some(
                loc_id_str
                    .parse::<Uuid>()
                    .map_err(|e| DataStoreError::ValidationError {
                        message: format!("Invalid location UUID: {}", e),
                    })?,
            )
        } else {
            None
        };

        let management_ip = if let Some(ip_str) = entity.management_ip {
            Some(
                ip_str
                    .parse()
                    .map_err(|e| DataStoreError::ValidationError {
                        message: format!("Invalid IP address: {}", e),
                    })?,
            )
        } else {
            None
        };

        let custom_data = if let Some(ref data_str) = entity.custom_data {
            serde_json::from_str(data_str).unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        let domain = entity.domain.clone().unwrap_or_default();
        let name = entity.name.clone();
        let fqdn = entity.fqdn.unwrap_or_else(|| {
            if !domain.is_empty() {
                format!("{}.{}", name, domain)
            } else {
                name.clone()
            }
        });

        Ok(Node {
            id,
            name,
            domain,
            fqdn,
            vendor,
            model: entity.model,
            role,
            lifecycle,
            management_ip,
            location_id,
            platform: None, // Not stored in entity yet
            version: None,  // Not stored in entity yet
            serial_number: entity.serial_number,
            asset_tag: entity.asset_tag,
            purchase_date: None,    // Not stored in entity yet
            warranty_expires: None, // Not stored in entity yet
            custom_data,
        })
    }

    /// SeaORM transaction wrapper
    pub struct SqliteTransaction {
        txn: DatabaseTransaction,
    }

    impl SqliteStore {
        /// Creates a new SQLite store with the given database URL
        pub async fn new(database_url: &str) -> DataStoreResult<Self> {
            let mut opt = ConnectOptions::new(database_url);
            opt.max_connections(100)
                .min_connections(5)
                .connect_timeout(Duration::from_secs(8))
                .acquire_timeout(Duration::from_secs(8))
                .idle_timeout(Duration::from_secs(8))
                .max_lifetime(Duration::from_secs(8))
                .sqlx_logging(false);

            let db = Database::connect(opt)
                .await
                .map_err(|e| DataStoreError::ConnectionError {
                    message: format!("Failed to connect to database: {}", e),
                })?;

            Ok(Self { db })
        }

        /// Get the database connection for testing
        pub fn connection(&self) -> &DatabaseConnection {
            &self.db
        }
    }

    #[async_trait]
    impl Transaction for SqliteTransaction {
        async fn commit(self: Box<Self>) -> DataStoreResult<()> {
            self.txn
                .commit()
                .await
                .map_err(|e| DataStoreError::TransactionError {
                    message: format!("Failed to commit transaction: {}", e),
                })
        }

        async fn rollback(self: Box<Self>) -> DataStoreResult<()> {
            self.txn
                .rollback()
                .await
                .map_err(|e| DataStoreError::TransactionError {
                    message: format!("Failed to rollback transaction: {}", e),
                })
        }
    }

    #[async_trait]
    impl DataStore for SqliteStore {
        fn name(&self) -> &'static str {
            "SQLite"
        }

        async fn health_check(&self) -> DataStoreResult<()> {
            self.db
                .ping()
                .await
                .map_err(|e| DataStoreError::ConnectionError {
                    message: format!("Database health check failed: {}", e),
                })
        }

        async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> {
            let txn = self
                .db
                .begin()
                .await
                .map_err(|e| DataStoreError::TransactionError {
                    message: format!("Failed to begin transaction: {}", e),
                })?;

            Ok(Box::new(SqliteTransaction { txn }))
        }

        // Note: For now, we'll implement stub methods that return UnsupportedOperation
        // These would be implemented with actual SeaORM entities once we have migrations set up

        async fn create_node(&self, node: &Node) -> DataStoreResult<Node> {
            let active_node = nodes::ActiveModel {
                id: Set(node.id.to_string()),
                name: Set(node.name.clone()),
                fqdn: Set(Some(node.fqdn.clone())),
                domain: Set(Some(node.domain.clone())),
                vendor: Set(node.vendor.to_string()),
                model: Set(node.model.clone()),
                role: Set(node.role.to_string()),
                lifecycle: Set(node.lifecycle.to_string()),
                serial_number: Set(node.serial_number.clone()),
                asset_tag: Set(node.asset_tag.clone()),
                location_id: Set(node.location_id.map(|id| id.to_string())),
                management_ip: Set(node.management_ip.map(|ip| ip.to_string())),
                description: Set(None), // Not used in Node model yet
                custom_data: Set(Some(
                    serde_json::to_string(&node.custom_data).unwrap_or_default(),
                )),
                created_at: Set(Utc::now().to_rfc3339()),
                updated_at: Set(Utc::now().to_rfc3339()),
            };

            active_node
                .insert(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to create node: {}", e),
                })?;

            // Convert back to Node model
            self.get_node(&node.id)
                .await?
                .ok_or_else(|| DataStoreError::NotFound {
                    entity_type: "Node".to_string(),
                    id: node.id.to_string(),
                })
        }

        async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>> {
            let entity = nodes::Entity::find_by_id(id.to_string())
                .one(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to query node: {}", e),
                })?;

            match entity {
                Some(e) => Ok(Some(entity_to_node(e)?)),
                None => Ok(None),
            }
        }

        async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> {
            let mut query = nodes::Entity::find();

            // Apply filters
            for filter in &options.filters {
                match filter.field.as_str() {
                    "name" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(nodes::Column::Name.contains(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Name filter must be a string".to_string(),
                            });
                        }
                    },
                    "vendor" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(nodes::Column::Vendor.eq(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Vendor filter must be a string".to_string(),
                            });
                        }
                    },
                    "role" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(nodes::Column::Role.eq(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Role filter must be a string".to_string(),
                            });
                        }
                    },
                    "lifecycle" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(nodes::Column::Lifecycle.eq(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Lifecycle filter must be a string".to_string(),
                            });
                        }
                    },
                    _ => {
                        return Err(DataStoreError::ValidationError {
                            message: format!("Unsupported filter field: {}", filter.field),
                        });
                    }
                }
            }

            // Apply sorting
            for sort in &options.sort {
                match sort.field.as_str() {
                    "name" => {
                        query = match sort.direction {
                            SortDirection::Ascending => query.order_by_asc(nodes::Column::Name),
                            SortDirection::Descending => query.order_by_desc(nodes::Column::Name),
                        };
                    }
                    "created_at" => {
                        query = match sort.direction {
                            SortDirection::Ascending => {
                                query.order_by_asc(nodes::Column::CreatedAt)
                            }
                            SortDirection::Descending => {
                                query.order_by_desc(nodes::Column::CreatedAt)
                            }
                        };
                    }
                    _ => {
                        return Err(DataStoreError::ValidationError {
                            message: format!("Unsupported sort field: {}", sort.field),
                        });
                    }
                }
            }

            // Get total count
            let total_count =
                query
                    .clone()
                    .count(&self.db)
                    .await
                    .map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to count nodes: {}", e),
                    })?;

            // Apply pagination
            if let Some(pagination) = &options.pagination {
                query = query
                    .offset(pagination.offset as u64)
                    .limit(pagination.limit as u64);
            }

            // Execute query
            let entities =
                query
                    .all(&self.db)
                    .await
                    .map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to query nodes: {}", e),
                    })?;

            // Convert entities to Node models
            let mut nodes = Vec::new();
            for entity in entities {
                nodes.push(entity_to_node(entity)?);
            }

            Ok(PagedResult::new(
                nodes,
                total_count as usize,
                options.pagination.as_ref(),
            ))
        }

        async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
            let active_node = nodes::ActiveModel {
                id: Set(node.id.to_string()),
                name: Set(node.name.clone()),
                fqdn: Set(Some(node.fqdn.clone())),
                domain: Set(Some(node.domain.clone())),
                vendor: Set(node.vendor.to_string()),
                model: Set(node.model.clone()),
                role: Set(node.role.to_string()),
                lifecycle: Set(node.lifecycle.to_string()),
                serial_number: Set(node.serial_number.clone()),
                asset_tag: Set(node.asset_tag.clone()),
                location_id: Set(node.location_id.map(|id| id.to_string())),
                management_ip: Set(node.management_ip.map(|ip| ip.to_string())),
                description: Set(None), // Not used in Node model yet
                custom_data: Set(Some(
                    serde_json::to_string(&node.custom_data).unwrap_or_default(),
                )),
                created_at: Set(Utc::now().to_rfc3339()), // This should ideally preserve original
                updated_at: Set(Utc::now().to_rfc3339()),
            };

            active_node
                .update(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to update node: {}", e),
                })?;

            self.get_node(&node.id)
                .await?
                .ok_or_else(|| DataStoreError::NotFound {
                    entity_type: "Node".to_string(),
                    id: node.id.to_string(),
                })
        }

        async fn delete_node(&self, id: &Uuid) -> DataStoreResult<()> {
            let result = nodes::Entity::delete_by_id(id.to_string())
                .exec(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to delete node: {}", e),
                })?;

            if result.rows_affected == 0 {
                return Err(DataStoreError::NotFound {
                    entity_type: "Node".to_string(),
                    id: id.to_string(),
                });
            }

            Ok(())
        }

        async fn get_nodes_by_location(&self, location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
            let entities = nodes::Entity::find()
                .filter(nodes::Column::LocationId.eq(location_id.to_string()))
                .all(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to query nodes by location: {}", e),
                })?;

            let mut nodes = Vec::new();
            for entity in entities {
                nodes.push(entity_to_node(entity)?);
            }

            Ok(nodes)
        }

        async fn search_nodes_by_name(&self, name: &str) -> DataStoreResult<Vec<Node>> {
            let entities = nodes::Entity::find()
                .filter(nodes::Column::Name.contains(name))
                .all(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to search nodes by name: {}", e),
                })?;

            let mut nodes = Vec::new();
            for entity in entities {
                nodes.push(entity_to_node(entity)?);
            }

            Ok(nodes)
        }

        async fn create_link(&self, link: &Link) -> DataStoreResult<Link> {
            let active_link = links::ActiveModel {
                id: Set(link.id.to_string()),
                name: Set(link.name.clone()),
                node_a_id: Set(link.node_a_id.to_string()),
                interface_a: Set(link.node_a_interface.clone()),
                node_b_id: Set(link.node_z_id.map(|id| id.to_string())),
                interface_b: Set(link.node_z_interface.clone()),
                capacity: Set(link.bandwidth.map(|b| b as i64)),
                utilization: Set(None), // Not in Link model yet
                is_internet_circuit: Set(if link.is_internet_circuit { 1 } else { 0 }),
                circuit_id: Set(None), // Not in Link model yet
                provider: Set(None),   // Not in Link model yet
                description: Set(link.description.clone()),
                custom_data: Set(Some(
                    serde_json::to_string(&link.custom_data).unwrap_or_default(),
                )),
                created_at: Set(Utc::now().to_rfc3339()),
                updated_at: Set(Utc::now().to_rfc3339()),
            };

            active_link
                .insert(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to create link: {}", e),
                })?;

            self.get_link(&link.id)
                .await?
                .ok_or_else(|| DataStoreError::NotFound {
                    entity_type: "Link".to_string(),
                    id: link.id.to_string(),
                })
        }

        async fn get_link(&self, id: &Uuid) -> DataStoreResult<Option<Link>> {
            let entity = links::Entity::find_by_id(id.to_string())
                .one(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to query link: {}", e),
                })?;

            match entity {
                Some(e) => Ok(Some(entity_to_link(e)?)),
                None => Ok(None),
            }
        }

        async fn list_links(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> {
            let mut query = links::Entity::find();

            // Apply filters
            for filter in &options.filters {
                match filter.field.as_str() {
                    "name" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(links::Column::Name.contains(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Name filter must be a string".to_string(),
                            });
                        }
                    },
                    "node_a_id" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(links::Column::NodeAId.eq(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Node A ID filter must be a string".to_string(),
                            });
                        }
                    },
                    "node_b_id" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(links::Column::NodeBId.eq(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Node B ID filter must be a string".to_string(),
                            });
                        }
                    },
                    _ => {
                        return Err(DataStoreError::ValidationError {
                            message: format!("Unsupported filter field: {}", filter.field),
                        });
                    }
                }
            }

            // Apply sorting
            for sort in &options.sort {
                match sort.field.as_str() {
                    "name" => {
                        query = match sort.direction {
                            SortDirection::Ascending => query.order_by_asc(links::Column::Name),
                            SortDirection::Descending => query.order_by_desc(links::Column::Name),
                        };
                    }
                    "created_at" => {
                        query = match sort.direction {
                            SortDirection::Ascending => {
                                query.order_by_asc(links::Column::CreatedAt)
                            }
                            SortDirection::Descending => {
                                query.order_by_desc(links::Column::CreatedAt)
                            }
                        };
                    }
                    _ => {
                        return Err(DataStoreError::ValidationError {
                            message: format!("Unsupported sort field: {}", sort.field),
                        });
                    }
                }
            }

            // Get total count
            let total_count =
                query
                    .clone()
                    .count(&self.db)
                    .await
                    .map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to count links: {}", e),
                    })?;

            // Apply pagination
            if let Some(pagination) = &options.pagination {
                query = query
                    .offset(pagination.offset as u64)
                    .limit(pagination.limit as u64);
            }

            // Execute query
            let entities =
                query
                    .all(&self.db)
                    .await
                    .map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to query links: {}", e),
                    })?;

            // Convert entities to Link models
            let mut links = Vec::new();
            for entity in entities {
                links.push(entity_to_link(entity)?);
            }

            Ok(PagedResult::new(
                links,
                total_count as usize,
                options.pagination.as_ref(),
            ))
        }

        async fn update_link(&self, link: &Link) -> DataStoreResult<Link> {
            let active_link = links::ActiveModel {
                id: Set(link.id.to_string()),
                name: Set(link.name.clone()),
                node_a_id: Set(link.node_a_id.to_string()),
                interface_a: Set(link.node_a_interface.clone()),
                node_b_id: Set(link.node_z_id.map(|id| id.to_string())),
                interface_b: Set(link.node_z_interface.clone()),
                capacity: Set(link.bandwidth.map(|b| b as i64)),
                utilization: Set(None), // Not in Link model yet
                is_internet_circuit: Set(if link.is_internet_circuit { 1 } else { 0 }),
                circuit_id: Set(None), // Not in Link model yet
                provider: Set(None),   // Not in Link model yet
                description: Set(link.description.clone()),
                custom_data: Set(Some(
                    serde_json::to_string(&link.custom_data).unwrap_or_default(),
                )),
                created_at: Set(Utc::now().to_rfc3339()), // This should ideally preserve original
                updated_at: Set(Utc::now().to_rfc3339()),
            };

            active_link
                .update(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to update link: {}", e),
                })?;

            self.get_link(&link.id)
                .await?
                .ok_or_else(|| DataStoreError::NotFound {
                    entity_type: "Link".to_string(),
                    id: link.id.to_string(),
                })
        }

        async fn delete_link(&self, id: &Uuid) -> DataStoreResult<()> {
            let result = links::Entity::delete_by_id(id.to_string())
                .exec(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to delete link: {}", e),
                })?;

            if result.rows_affected == 0 {
                return Err(DataStoreError::NotFound {
                    entity_type: "Link".to_string(),
                    id: id.to_string(),
                });
            }

            Ok(())
        }

        async fn get_links_for_node(&self, node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
            let entities = links::Entity::find()
                .filter(
                    links::Column::NodeAId
                        .eq(node_id.to_string())
                        .or(links::Column::NodeBId.eq(node_id.to_string())),
                )
                .all(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to query links for node: {}", e),
                })?;

            let mut links = Vec::new();
            for entity in entities {
                links.push(entity_to_link(entity)?);
            }

            Ok(links)
        }

        async fn get_links_between_nodes(
            &self,
            first_node_id: &Uuid,
            second_node_id: &Uuid,
        ) -> DataStoreResult<Vec<Link>> {
            let entities = links::Entity::find()
                .filter(
                    (links::Column::NodeAId
                        .eq(first_node_id.to_string())
                        .and(links::Column::NodeBId.eq(second_node_id.to_string())))
                    .or(links::Column::NodeAId
                        .eq(second_node_id.to_string())
                        .and(links::Column::NodeBId.eq(first_node_id.to_string()))),
                )
                .all(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to query links between nodes: {}", e),
                })?;

            let mut links = Vec::new();
            for entity in entities {
                links.push(entity_to_link(entity)?);
            }

            Ok(links)
        }

        async fn create_location(&self, location: &Location) -> DataStoreResult<Location> {
            let active_location = locations::ActiveModel {
                id: Set(location.id.to_string()),
                name: Set(location.name.clone()),
                location_type: Set(location.location_type.clone()),
                path: Set(location.path.clone()),
                parent_id: Set(location.parent_id.map(|id| id.to_string())),
                description: Set(location.description.clone()),
                address: Set(location.address.clone()),
                coordinates: Set(None), // Not in Location model yet
                custom_data: Set(Some(
                    serde_json::to_string(&location.custom_data).unwrap_or_default(),
                )),
                created_at: Set(Utc::now().to_rfc3339()),
                updated_at: Set(Utc::now().to_rfc3339()),
            };

            active_location
                .insert(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to create location: {}", e),
                })?;

            self.get_location(&location.id)
                .await?
                .ok_or_else(|| DataStoreError::NotFound {
                    entity_type: "Location".to_string(),
                    id: location.id.to_string(),
                })
        }

        async fn get_location(&self, id: &Uuid) -> DataStoreResult<Option<Location>> {
            let entity = locations::Entity::find_by_id(id.to_string())
                .one(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to query location: {}", e),
                })?;

            match entity {
                Some(e) => Ok(Some(entity_to_location(e)?)),
                None => Ok(None),
            }
        }

        async fn list_locations(
            &self,
            options: &QueryOptions,
        ) -> DataStoreResult<PagedResult<Location>> {
            let mut query = locations::Entity::find();

            // Apply filters
            for filter in &options.filters {
                match filter.field.as_str() {
                    "name" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(locations::Column::Name.contains(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Name filter must be a string".to_string(),
                            });
                        }
                    },
                    "location_type" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(locations::Column::LocationType.eq(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Location type filter must be a string".to_string(),
                            });
                        }
                    },
                    "parent_id" => match &filter.value {
                        FilterValue::String(s) => {
                            query = query.filter(locations::Column::ParentId.eq(s));
                        }
                        _ => {
                            return Err(DataStoreError::ValidationError {
                                message: "Parent ID filter must be a string".to_string(),
                            });
                        }
                    },
                    _ => {
                        return Err(DataStoreError::ValidationError {
                            message: format!("Unsupported filter field: {}", filter.field),
                        });
                    }
                }
            }

            // Apply sorting
            for sort in &options.sort {
                match sort.field.as_str() {
                    "name" => {
                        query = match sort.direction {
                            SortDirection::Ascending => query.order_by_asc(locations::Column::Name),
                            SortDirection::Descending => {
                                query.order_by_desc(locations::Column::Name)
                            }
                        };
                    }
                    "path" => {
                        query = match sort.direction {
                            SortDirection::Ascending => query.order_by_asc(locations::Column::Path),
                            SortDirection::Descending => {
                                query.order_by_desc(locations::Column::Path)
                            }
                        };
                    }
                    "created_at" => {
                        query = match sort.direction {
                            SortDirection::Ascending => {
                                query.order_by_asc(locations::Column::CreatedAt)
                            }
                            SortDirection::Descending => {
                                query.order_by_desc(locations::Column::CreatedAt)
                            }
                        };
                    }
                    _ => {
                        return Err(DataStoreError::ValidationError {
                            message: format!("Unsupported sort field: {}", sort.field),
                        });
                    }
                }
            }

            // Get total count
            let total_count =
                query
                    .clone()
                    .count(&self.db)
                    .await
                    .map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to count locations: {}", e),
                    })?;

            // Apply pagination
            if let Some(pagination) = &options.pagination {
                query = query
                    .offset(pagination.offset as u64)
                    .limit(pagination.limit as u64);
            }

            // Execute query
            let entities =
                query
                    .all(&self.db)
                    .await
                    .map_err(|e| DataStoreError::InternalError {
                        message: format!("Failed to query locations: {}", e),
                    })?;

            // Convert entities to Location models
            let mut locations = Vec::new();
            for entity in entities {
                locations.push(entity_to_location(entity)?);
            }

            Ok(PagedResult::new(
                locations,
                total_count as usize,
                options.pagination.as_ref(),
            ))
        }

        async fn update_location(&self, location: &Location) -> DataStoreResult<Location> {
            let active_location = locations::ActiveModel {
                id: Set(location.id.to_string()),
                name: Set(location.name.clone()),
                location_type: Set(location.location_type.clone()),
                path: Set(location.path.clone()),
                parent_id: Set(location.parent_id.map(|id| id.to_string())),
                description: Set(location.description.clone()),
                address: Set(location.address.clone()),
                coordinates: Set(None), // Not in Location model yet
                custom_data: Set(Some(
                    serde_json::to_string(&location.custom_data).unwrap_or_default(),
                )),
                created_at: Set(Utc::now().to_rfc3339()), // This should ideally preserve original
                updated_at: Set(Utc::now().to_rfc3339()),
            };

            active_location
                .update(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to update location: {}", e),
                })?;

            self.get_location(&location.id)
                .await?
                .ok_or_else(|| DataStoreError::NotFound {
                    entity_type: "Location".to_string(),
                    id: location.id.to_string(),
                })
        }

        async fn delete_location(&self, id: &Uuid) -> DataStoreResult<()> {
            let result = locations::Entity::delete_by_id(id.to_string())
                .exec(&self.db)
                .await
                .map_err(|e| DataStoreError::InternalError {
                    message: format!("Failed to delete location: {}", e),
                })?;

            if result.rows_affected == 0 {
                return Err(DataStoreError::NotFound {
                    entity_type: "Location".to_string(),
                    id: id.to_string(),
                });
            }

            Ok(())
        }

        async fn get_child_locations(&self, _parent_id: &Uuid) -> DataStoreResult<Vec<Location>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_child_locations not yet implemented - awaiting migrations"
                    .to_string(),
            })
        }

        async fn get_location_tree(&self) -> DataStoreResult<Vec<Location>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_location_tree not yet implemented - awaiting migrations"
                    .to_string(),
            })
        }

        async fn validate_location_hierarchy(
            &self,
            _child_id: &Uuid,
            _new_parent_id: &Uuid,
        ) -> DataStoreResult<()> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "validate_location_hierarchy not yet implemented - awaiting migrations"
                    .to_string(),
            })
        }

        async fn batch_nodes(
            &self,
            _operations: &[BatchOperation<Node>], // Trait method signature requires this parameter
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "batch_nodes not yet implemented - awaiting migrations".to_string(),
            })
        }

        async fn batch_links(
            &self,
            _operations: &[BatchOperation<Link>], // Trait method signature requires this parameter
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "batch_links not yet implemented - awaiting migrations".to_string(),
            })
        }

        async fn batch_locations(
            &self,
            _operations: &[BatchOperation<Location>], // Trait method signature requires this parameter
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "batch_locations not yet implemented - awaiting migrations".to_string(),
            })
        }

        async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
            Err(DataStoreError::UnsupportedOperation {
                operation: "get_entity_counts not yet implemented - awaiting migrations"
                    .to_string(),
            })
        }

        async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
            let mut stats = HashMap::new();
            stats.insert(
                "type".to_string(),
                serde_json::Value::String("SQLite".to_string()),
            );
            stats.insert(
                "status".to_string(),
                serde_json::Value::String("awaiting migrations".to_string()),
            );
            Ok(stats)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_new() {
        // Valid pagination
        let pagination = Pagination::new(10, 0).unwrap();
        assert_eq!(pagination.limit, 10);
        assert_eq!(pagination.offset, 0);

        // Invalid limit (0)
        assert!(Pagination::new(0, 0).is_err());

        // Invalid limit (too large)
        assert!(Pagination::new(1001, 0).is_err());

        // Valid edge cases
        assert!(Pagination::new(1, 0).is_ok());
        assert!(Pagination::new(1000, 0).is_ok());
    }

    #[test]
    fn test_pagination_page() {
        // Valid page pagination
        let pagination = Pagination::page(1, 10).unwrap();
        assert_eq!(pagination.limit, 10);
        assert_eq!(pagination.offset, 0);

        let pagination = Pagination::page(3, 20).unwrap();
        assert_eq!(pagination.limit, 20);
        assert_eq!(pagination.offset, 40);

        // Invalid page (0)
        assert!(Pagination::page(0, 10).is_err());
    }

    #[test]
    fn test_paged_result_new() {
        let items = vec![1, 2, 3];

        // Without pagination
        let result = PagedResult::new(items.clone(), 3, None);
        assert_eq!(result.items, items);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.page_size, 3);
        assert_eq!(result.page, 1);
        assert_eq!(result.total_pages, 1);
        assert!(!result.has_next);
        assert!(!result.has_previous);

        // With pagination - first page
        let pagination = Pagination::new(2, 0).unwrap();
        let result = PagedResult::new(vec![1, 2], 5, Some(&pagination));
        assert_eq!(result.items, vec![1, 2]);
        assert_eq!(result.total_count, 5);
        assert_eq!(result.page_size, 2);
        assert_eq!(result.page, 1);
        assert_eq!(result.total_pages, 3);
        assert!(result.has_next);
        assert!(!result.has_previous);

        // With pagination - middle page
        let pagination = Pagination::new(2, 2).unwrap();
        let result = PagedResult::new(vec![3, 4], 5, Some(&pagination));
        assert_eq!(result.page, 2);
        assert!(result.has_next);
        assert!(result.has_previous);

        // With pagination - last page
        let pagination = Pagination::new(2, 4).unwrap();
        let result = PagedResult::new(vec![5], 5, Some(&pagination));
        assert_eq!(result.page, 3);
        assert!(!result.has_next);
        assert!(result.has_previous);
    }

    #[test]
    fn test_filter_helpers() {
        let filter = filter_equals_string("name", "test");
        assert_eq!(filter.field, "name");
        assert!(matches!(filter.operation, FilterOperation::Equals));
        assert!(matches!(filter.value, FilterValue::String(ref s) if s == "test"));

        let uuid = Uuid::new_v4();
        let filter = filter_equals_uuid("id", uuid);
        assert_eq!(filter.field, "id");
        assert!(matches!(filter.operation, FilterOperation::Equals));
        assert!(matches!(filter.value, FilterValue::Uuid(u) if u == uuid));

        let filter = filter_contains("description", "partial");
        assert_eq!(filter.field, "description");
        assert!(matches!(filter.operation, FilterOperation::Contains));
        assert!(matches!(filter.value, FilterValue::String(ref s) if s == "partial"));
    }

    #[test]
    fn test_sort_helpers() {
        let sort = sort_asc("name");
        assert_eq!(sort.field, "name");
        assert!(matches!(sort.direction, SortDirection::Ascending));

        let sort = sort_desc("created_at");
        assert_eq!(sort.field, "created_at");
        assert!(matches!(sort.direction, SortDirection::Descending));
    }

    #[test]
    fn test_query_options_default() {
        let options = QueryOptions::default();
        assert!(options.filters.is_empty());
        assert!(options.sort.is_empty());
        assert!(options.pagination.is_none());
    }

    #[test]
    fn test_batch_result() {
        let result = BatchResult {
            success_count: 5,
            error_count: 2,
            errors: vec![
                (
                    1,
                    DataStoreError::ValidationError {
                        message: "Test error".to_string(),
                    },
                ),
                (
                    3,
                    DataStoreError::NotFound {
                        entity_type: "Node".to_string(),
                        id: "test-id".to_string(),
                    },
                ),
            ],
        };

        assert_eq!(result.success_count, 5);
        assert_eq!(result.error_count, 2);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_datastore_error_display() {
        let error = DataStoreError::NotFound {
            entity_type: "Node".to_string(),
            id: "123".to_string(),
        };
        assert!(error.to_string().contains("Entity not found"));
        assert!(error.to_string().contains("Node"));
        assert!(error.to_string().contains("123"));

        let error = DataStoreError::ValidationError {
            message: "Invalid input".to_string(),
        };
        assert!(error.to_string().contains("Validation error"));
        assert!(error.to_string().contains("Invalid input"));

        let error = DataStoreError::Timeout { seconds: 30 };
        assert!(error.to_string().contains("timeout"));
        assert!(error.to_string().contains("30"));
    }
}
