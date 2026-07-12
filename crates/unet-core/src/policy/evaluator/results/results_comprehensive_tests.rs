//! Comprehensive tests for policy evaluation results
//!
//! Tests are organized into focused modules by functionality.

#[path = "test_helpers.rs"]
mod test_helpers;

#[path = "aggregated_results_tests.rs"]
mod aggregated_results_tests;

#[path = "summary_tests.rs"]
mod summary_tests;

#[path = "error_compliance_tests.rs"]
mod error_compliance_tests;

#[path = "filtering_tests.rs"]
mod filtering_tests;

#[path = "serialization_tests.rs"]
mod serialization_tests;

#[path = "priority_tests.rs"]
mod priority_tests;
