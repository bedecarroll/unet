//! Location operations for `SQLite` datastore
//!
//! This module organizes location operations into focused submodules:
//! - `crud_operations`: Basic CRUD operations (create, read, update, delete)
//! - `query_operations`: List operations with filtering, sorting, and pagination
//! - `batch_operations`: Batch processing functionality
//! - `tests`: Comprehensive test suite

pub mod batch_operations;
pub mod crud_operations;
pub mod query_operations;

#[cfg(test)]
mod tests;

// Re-export all public functions for backward compatibility
pub use batch_operations::batch_locations;
pub use crud_operations::{create_location, delete_location, get_location, update_location};
pub use query_operations::list_locations;
