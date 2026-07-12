//! Comprehensive error path tests for `ActionExecutor`
//!
//! Tests are organized into focused modules by functionality.

#[cfg(test)]
#[path = "assert_action_tests.rs"]
mod assert_action_tests;

#[cfg(test)]
#[path = "set_action_tests.rs"]
mod set_action_tests;

#[cfg(test)]
#[path = "apply_template_tests.rs"]
mod apply_template_tests;

#[cfg(test)]
#[path = "resolve_value_tests.rs"]
mod resolve_value_tests;

#[cfg(test)]
#[path = "nested_field_tests.rs"]
mod nested_field_tests;

#[cfg(test)]
#[path = "utility_tests.rs"]
mod utility_tests;
