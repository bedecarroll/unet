//! Tests to validate schema parity between migrations and entities.
//!
//! Tests are organized into focused modules by responsibility.

#[path = "schema_parity_tests/comparison.rs"]
mod comparison;
#[path = "schema_parity_tests/database.rs"]
mod database;
#[path = "schema_parity_tests/formatting.rs"]
mod formatting;
#[path = "schema_parity_tests/parity_tests.rs"]
mod parity_tests;
#[path = "schema_parity_tests/types.rs"]
mod types;
#[path = "schema_parity_tests/utility_tests.rs"]
mod utility_tests;
