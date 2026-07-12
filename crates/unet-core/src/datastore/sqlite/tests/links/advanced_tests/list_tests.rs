//! List operation tests for `SQLite` link operations
//!
//! Tests covering list, filtering, pagination, and sorting functionality.

use super::super::super::setup::{create_test_node, setup_test_db};
use super::super::helpers::{create_internet_circuit_link, create_test_link};
use crate::datastore::sqlite::links::*;
use crate::datastore::types::{
    Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort, SortDirection,
};
use uuid::Uuid;

#[tokio::test]
async fn test_list_links_empty() {
    let test_db = setup_test_db().await;
    let options = QueryOptions::default();

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 0);
    assert_eq!(paged_result.total_count, 0);
}

#[tokio::test]
async fn test_list_links_with_data() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest-node")
        .await
        .unwrap();

    // Create multiple links
    for i in 1..=3 {
        let link = create_test_link(&format!("link-{i}"), source_node_id, Some(dest_node_id));
        create_link(&test_db.store, &link).await.unwrap();
    }

    let options = QueryOptions::default();
    let result = list_links(&test_db.store, &options).await;

    assert!(result.is_ok());
    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.total_count, 3);

    // Verify links are returned in order
    for (i, link) in paged_result.items.iter().enumerate() {
        assert_eq!(link.name, format!("link-{}", i + 1));
    }
}

#[tokio::test]
async fn test_list_links_with_pagination() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create a node
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    // Create 5 links
    for i in 1..=5 {
        let link = create_test_link(&format!("link-{i}"), source_node_id, None);
        create_link(&test_db.store, &link).await.unwrap();
    }

    // Test first page
    let pagination = Pagination {
        limit: 2,
        offset: 0,
    };
    let options = QueryOptions {
        pagination: Some(pagination),
        ..Default::default()
    };
    let result = list_links(&test_db.store, &options).await.unwrap();
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_count, 5);

    // Test second page
    let pagination = Pagination {
        limit: 2,
        offset: 2,
    };
    let options = QueryOptions {
        pagination: Some(pagination),
        ..Default::default()
    };
    let result = list_links(&test_db.store, &options).await.unwrap();
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_count, 5);

    // Test last page
    let pagination = Pagination {
        limit: 2,
        offset: 4,
    };
    let options = QueryOptions {
        pagination: Some(pagination),
        ..Default::default()
    };
    let result = list_links(&test_db.store, &options).await.unwrap();
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.total_count, 5);
}

#[tokio::test]
async fn test_list_links_with_filters() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest-node")
        .await
        .unwrap();

    // Create circuit and ethernet links
    let circuit_link = create_internet_circuit_link("circuit-1", source_node_id);
    create_link(&test_db.store, &circuit_link).await.unwrap();

    let ethernet_link = create_test_link("ethernet-1", source_node_id, Some(dest_node_id));
    create_link(&test_db.store, &ethernet_link).await.unwrap();

    // Filter by is_internet_circuit
    let filter = Filter {
        field: "is_internet_circuit".to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Boolean(true),
    };
    let options = QueryOptions {
        filters: vec![filter],
        ..Default::default()
    };

    let result = list_links(&test_db.store, &options).await.unwrap();
    assert_eq!(result.items.len(), 1);
    assert!(result.items[0].is_internet_circuit);
}

#[tokio::test]
async fn test_list_links_with_sorting() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create a node
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    // Create links with different names (out of order)
    let names = vec!["charlie", "alpha", "bravo"];
    for name in names {
        let link = create_test_link(name, source_node_id, None);
        create_link(&test_db.store, &link).await.unwrap();
    }

    // Test ascending sort
    let sort = Sort {
        field: "name".to_string(),
        direction: SortDirection::Ascending,
    };
    let options = QueryOptions {
        sort: vec![sort],
        ..Default::default()
    };

    let result = list_links(&test_db.store, &options).await.unwrap();
    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0].name, "alpha");
    assert_eq!(result.items[1].name, "bravo");
    assert_eq!(result.items[2].name, "charlie");

    // Test descending sort
    let sort = Sort {
        field: "name".to_string(),
        direction: SortDirection::Descending,
    };
    let options = QueryOptions {
        sort: vec![sort],
        ..Default::default()
    };

    let result = list_links(&test_db.store, &options).await.unwrap();
    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0].name, "charlie");
    assert_eq!(result.items[1].name, "bravo");
    assert_eq!(result.items[2].name, "alpha");
}
