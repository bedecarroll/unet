//! Tests for mock datastore implementations used in policy integration testing
//!
//! This module contains comprehensive tests for both `FailingMockDataStore`
//! and `EmptyMockDataStore` implementations to ensure proper error handling
//! and empty state behavior in policy integration scenarios.

pub mod empty_mock_tests;
pub mod failing_mock_tests;
pub mod shared;
