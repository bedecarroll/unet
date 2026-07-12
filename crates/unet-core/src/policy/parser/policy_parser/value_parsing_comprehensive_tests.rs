//! Comprehensive tests for value parsing functionality
//!
//! Tests are organized into focused modules by functionality.

#[cfg(test)]
#[path = "value_parsing/operator_tests.rs"]
mod operator_tests;

#[cfg(test)]
#[path = "value_parsing/field_ref_tests.rs"]
mod field_ref_tests;

#[cfg(test)]
#[path = "value_parsing/string_value_tests.rs"]
mod string_value_tests;

#[cfg(test)]
#[path = "value_parsing/number_value_tests.rs"]
mod number_value_tests;

#[cfg(test)]
#[path = "value_parsing/boolean_value_tests.rs"]
mod boolean_value_tests;

#[cfg(test)]
#[path = "value_parsing/other_value_tests.rs"]
mod other_value_tests;

#[cfg(test)]
#[path = "value_parsing/edge_case_tests.rs"]
mod edge_case_tests;
