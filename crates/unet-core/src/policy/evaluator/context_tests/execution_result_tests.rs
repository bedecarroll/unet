//! Tests for `PolicyExecutionResult` and execution flow
//!
//! Tests are organized into focused modules by functionality.

#[cfg(test)]
#[path = "execution_result/construction_tests.rs"]
mod construction_tests;

#[cfg(test)]
#[path = "execution_result/status_tests.rs"]
mod status_tests;

#[cfg(test)]
#[path = "execution_result/utility_tests.rs"]
mod utility_tests;
