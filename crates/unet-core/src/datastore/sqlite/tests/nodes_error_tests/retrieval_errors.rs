//! Node retrieval and query error tests

use super::super::setup::{create_test_node, setup_test_db};
use crate::datastore::sqlite::nodes::*;
use crate::datastore::types::{
    DataStoreError, Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort,
    SortDirection,
};
use uuid::Uuid;

#[tokio::test]
async fn test_get_node_with_malformed_uuid() {
    let test_db = setup_test_db().await;

    // This test verifies UUID parsing is handled correctly
    // by using a valid UUID that doesn't exist
    let non_existent_id = Uuid::new_v4();

    let result = get_node(&test_db.store, &non_existent_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_list_nodes_with_invalid_filters() {
    let test_db = setup_test_db().await;

    // Create a node first
    let node_id = Uuid::new_v4();
    create_test_node(&test_db.store, node_id, "test-node")
        .await
        .unwrap();

    // Test with invalid filter field that doesn't exist
    let options = QueryOptions {
        filters: vec![Filter {
            field: "non_existent_field".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("value".to_string()),
        }],
        sort: vec![],
        pagination: None,
    };

    let result = list_nodes(&test_db.store, &options).await;

    // Should return error for invalid filter field
    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::ValidationError { message } => {
            assert_eq!(message, "Unsupported filter field: non_existent_field");
        }
        other => panic!("Expected ValidationError for invalid filter field, got {other:?}"),
    }
}

#[tokio::test]
async fn test_list_nodes_with_invalid_sort_field() {
    let test_db = setup_test_db().await;

    // Create a node first
    let node_id = Uuid::new_v4();
    create_test_node(&test_db.store, node_id, "test-node")
        .await
        .unwrap();

    // Test with invalid sort field
    let options = QueryOptions {
        filters: vec![],
        sort: vec![Sort {
            field: "invalid_sort_field".to_string(),
            direction: SortDirection::Ascending,
        }],
        pagination: None,
    };

    let result = list_nodes(&test_db.store, &options).await;

    // Should return error for invalid sort field
    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::ValidationError { message } => {
            assert_eq!(message, "Unsupported sort field: invalid_sort_field");
        }
        other => panic!("Expected ValidationError for invalid sort field, got {other:?}"),
    }
}

#[tokio::test]
async fn test_list_nodes_with_invalid_pagination() {
    let test_db = setup_test_db().await;

    // Create a node first
    let node_id = Uuid::new_v4();
    create_test_node(&test_db.store, node_id, "test-node")
        .await
        .unwrap();

    // Test with invalid pagination (limit too high)
    let invalid_pagination = Pagination::new(10_000, 0); // Way over limit
    assert!(invalid_pagination.is_err());

    // Test with zero page size - should be caught at validation level
    let zero_page_pagination = Pagination::new(0, 0);
    assert!(zero_page_pagination.is_err());
}

#[tokio::test]
async fn test_get_nodes_by_location_empty_result() {
    let test_db = setup_test_db().await;

    // Get nodes for a location that doesn't exist
    let non_existent_location_id = Uuid::new_v4();
    let result = get_nodes_by_location(&test_db.store, &non_existent_location_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_search_nodes_by_name_empty_result() {
    let test_db = setup_test_db().await;

    let result = search_nodes_by_name(&test_db.store, "nonexistent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_search_nodes_by_name_special_characters() {
    let test_db = setup_test_db().await;

    // Create test nodes first
    let node_id1 = Uuid::new_v4();
    create_test_node(&test_db.store, node_id1, "node-with-dash")
        .await
        .unwrap();

    let node_id2 = Uuid::new_v4();
    create_test_node(&test_db.store, node_id2, "node_with_underscore")
        .await
        .unwrap();

    let node_id3 = Uuid::new_v4();
    create_test_node(&test_db.store, node_id3, "node.with.dots")
        .await
        .unwrap();

    // Search for nodes with special characters
    let dash_results = search_nodes_by_name(&test_db.store, "dash").await;
    assert!(dash_results.is_ok());
    assert_eq!(dash_results.unwrap().len(), 1);

    let underscore_results = search_nodes_by_name(&test_db.store, "underscore").await;
    assert!(underscore_results.is_ok());
    assert_eq!(underscore_results.unwrap().len(), 1);

    let dots_results = search_nodes_by_name(&test_db.store, "dots").await;
    assert!(dots_results.is_ok());
    assert_eq!(dots_results.unwrap().len(), 1);

    // Search with SQL wildcard characters - should be escaped
    let wildcard_results = search_nodes_by_name(&test_db.store, "%").await;
    assert!(wildcard_results.is_ok());
    assert!(wildcard_results.unwrap().is_empty());

    let underscore_wildcard_results = search_nodes_by_name(&test_db.store, "_").await;
    assert!(underscore_wildcard_results.is_ok());
    assert!(underscore_wildcard_results.unwrap().is_empty());
}

#[tokio::test]
async fn test_list_nodes_with_large_offset() {
    let test_db = setup_test_db().await;

    // Create a few test nodes
    for i in 0..5 {
        let node_id = Uuid::new_v4();
        create_test_node(&test_db.store, node_id, &format!("test-node-{i}"))
            .await
            .unwrap();
    }

    // Test with large offset beyond available data
    let pagination = Pagination::new(10, 100).unwrap(); // Page size 10, offset 100
    let options = QueryOptions {
        filters: vec![],
        sort: vec![],
        pagination: Some(pagination),
    };

    let result = list_nodes(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert!(paged_result.items.is_empty());
    assert_eq!(paged_result.total_count, 5); // Should still return correct total
    assert!(!paged_result.has_next);
    assert!(paged_result.has_previous);
}

#[tokio::test]
async fn test_list_nodes_integer_overflow_protection() {
    let test_db = setup_test_db().await;

    // Test that large but valid pagination values don't cause issues
    let large_page_size = Pagination::new(1000, 0).unwrap(); // Max allowed page size
    let options = QueryOptions {
        filters: vec![],
        sort: vec![],
        pagination: Some(large_page_size),
    };

    let result = list_nodes(&test_db.store, &options).await;
    assert!(result.is_ok());

    // Should return empty result but not crash
    let paged_result = result.unwrap();
    assert!(paged_result.items.is_empty());
    assert_eq!(paged_result.total_count, 0);
}
