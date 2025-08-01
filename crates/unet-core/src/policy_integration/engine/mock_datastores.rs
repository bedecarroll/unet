//! Mock `DataStore` implementations for testing
//!
//! This module has been split into focused modules:
//! - `empty_mock` - `EmptyMockDataStore` implementation
//! - `failing_mock` - `FailingMockDataStore` implementation  
//! - `mock_utils` - Utility functions and re-exports

#[cfg(test)]
#[path = "empty_mock.rs"]
mod empty_mock;
#[cfg(test)]
#[path = "failing_mock.rs"]
mod failing_mock;
#[cfg(test)]
#[path = "mock_utils.rs"]
mod mock_utils;

#[cfg(test)]
pub mod mocks {
    pub use super::empty_mock::EmptyMockDataStore;
    pub use super::failing_mock::FailingMockDataStore;
}
