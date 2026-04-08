//! Tests for `ClientOperations` organized by behavior area.

#[path = "client_operations_tests/concurrency_tests.rs"]
mod concurrency_tests;
#[path = "client_operations_tests/construction_tests.rs"]
mod construction_tests;
#[path = "client_operations_tests/fixtures.rs"]
mod fixtures;
#[path = "client_operations_tests/mock_operation_tests.rs"]
mod mock_operation_tests;
#[path = "client_operations_tests/semaphore_tests.rs"]
mod semaphore_tests;
#[path = "client_operations_tests/session_pool_tests.rs"]
mod session_pool_tests;
