/// Tests for location commands
///
/// Tests are organized into focused modules by functionality.
#[cfg(test)]
#[path = "args_tests.rs"]
mod args_tests;

#[cfg(test)]
#[path = "types_address_tests.rs"]
mod types_address_tests;

#[cfg(test)]
#[path = "json_parsing_tests.rs"]
mod json_parsing_tests;

#[cfg(test)]
#[path = "query_operations_tests.rs"]
mod query_operations_tests;

#[cfg(test)]
#[path = "command_structure_tests.rs"]
mod command_structure_tests;

#[cfg(test)]
#[path = "crud_business_logic_tests.rs"]
mod crud_business_logic_tests;
