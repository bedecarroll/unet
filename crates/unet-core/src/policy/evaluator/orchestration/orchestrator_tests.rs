// Tests for policy orchestration components
//
// This module has been split into focused test modules:
// - `basic_orchestration_tests` - Basic orchestrator functionality
// - `caching_tests` - Cache-related tests
// - `batch_execution_tests` - Batch processing and timeout tests
// - `orchestrator_test_helpers` - Shared test helper functions

#[cfg(test)]
#[path = "basic_orchestration_tests.rs"]
mod basic_orchestration_tests;
#[cfg(test)]
#[path = "batch_execution_tests.rs"]
mod batch_execution_tests;
#[cfg(test)]
#[path = "caching_tests.rs"]
mod caching_tests;
#[cfg(test)]
#[path = "orchestrator_test_helpers.rs"]
pub mod orchestrator_test_helpers;
