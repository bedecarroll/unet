// Tests for policy execution and evaluation logic
//
// This module has been split into focused test modules:
// - `node_evaluation_tests` - Individual node evaluation tests
// - `bulk_evaluation_tests` - Bulk evaluation functionality tests
// - `policy_loading_tests` - Policy loading and parsing tests
// - `test_helpers` - Shared test helper functions

#[cfg(test)]
#[path = "bulk_evaluation_tests.rs"]
mod bulk_evaluation_tests;
#[cfg(test)]
#[path = "node_evaluation_tests.rs"]
mod node_evaluation_tests;
#[cfg(test)]
#[path = "policy_loading_tests.rs"]
mod policy_loading_tests;
#[cfg(test)]
#[path = "test_helpers.rs"]
pub mod test_helpers;
