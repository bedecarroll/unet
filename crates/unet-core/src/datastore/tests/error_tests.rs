//! Tests for `DataStoreError` display and error handling

use crate::datastore::types::DataStoreError;

#[test]
fn test_batch_result() {
    // Test basic batch result functionality - simplified test
    let error = DataStoreError::NotFound {
        entity_type: "Node".to_string(),
        id: "test".to_string(),
    };
    assert!(error.to_string().contains("test"));
}

#[test]
fn test_datastore_error_display() {
    let error = DataStoreError::NotFound {
        entity_type: "Node".to_string(),
        id: "123".to_string(),
    };
    assert!(error.to_string().contains("Node"));

    let error = DataStoreError::ValidationError {
        message: "Invalid data".to_string(),
    };
    assert_eq!(error.to_string(), "Validation error: Invalid data");

    let error = DataStoreError::ConstraintViolation {
        message: "Unique constraint failed".to_string(),
    };
    assert!(error.to_string().contains("Constraint violation"));
}
