//! Comprehensive tests for policy AST functionality
//!
//! Tests are organized into focused modules by functionality.

#[cfg(test)]
#[path = "ast/display_tests.rs"]
mod display_tests;

#[cfg(test)]
#[path = "ast/structure_tests.rs"]
mod structure_tests;

#[cfg(test)]
#[path = "ast/complex_tests.rs"]
mod complex_tests;

#[cfg(test)]
#[path = "ast/traits_tests.rs"]
mod traits_tests;

#[cfg(test)]
#[path = "ast/edge_case_tests.rs"]
mod edge_case_tests;
