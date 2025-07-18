//! Type definitions for the `DataStore` abstraction layer

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    ///
    /// # Errors
    /// Returns an error if limit is 0 or greater than 1000
    pub fn new(limit: usize, offset: usize) -> DataStoreResult<Self> {
        if limit == 0 || limit > 1000 {
            return Err(DataStoreError::ValidationError {
                message: "Limit must be between 1 and 1000".to_string(),
            });
        }

        Ok(Self { limit, offset })
    }

    /// Creates pagination for a specific page
    ///
    /// # Errors
    /// Returns an error if page is 0 or `page_size` is invalid
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
    #[must_use]
    pub fn new(items: Vec<T>, total_count: usize, pagination: Option<&Pagination>) -> Self {
        let (page_size, page, total_pages, has_next, has_previous) =
            pagination.map_or((total_count, 1, 1, false, false), |p| {
                let page = (p.offset / p.limit) + 1;
                let total_pages = total_count.div_ceil(p.limit);
                let has_next = page < total_pages;
                let has_previous = page > 1;
                (p.limit, page, total_pages, has_next, has_previous)
            });

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
    ///
    /// # Errors
    /// Returns an error if the transaction cannot be committed
    async fn commit(self: Box<Self>) -> DataStoreResult<()>;

    /// Rolls back the transaction
    ///
    /// # Errors
    /// Returns an error if the transaction cannot be rolled back
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

#[cfg(test)]
mod tests;
