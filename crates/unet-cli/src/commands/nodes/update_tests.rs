// Tests for node update functionality
//
// This module has been split into focused test modules:
// - `field_update_tests` - Tests for individual field updates
// - `validation_tests` - Tests for validation and error cases
// - `output_format_tests` - Tests for different output formats

#[cfg(test)]
#[path = "field_update_tests.rs"]
mod field_update_tests;
#[cfg(test)]
#[path = "output_format_tests.rs"]
mod output_format_tests;
#[cfg(test)]
#[path = "validation_tests.rs"]
mod validation_tests;
