//! Advanced tests for `SQLite` link operations
//!
//! Comprehensive tests covering list, filtering, pagination, node relationships,
//! batch operations, and edge cases to achieve high coverage.
//!
//! Tests are organized into focused modules by functionality.

#[path = "list_tests.rs"]
mod list_tests;

#[path = "node_relationship_tests.rs"]
mod node_relationship_tests;

#[path = "batch_operations_tests.rs"]
mod batch_operations_tests;

#[path = "edge_case_tests.rs"]
mod edge_case_tests;
