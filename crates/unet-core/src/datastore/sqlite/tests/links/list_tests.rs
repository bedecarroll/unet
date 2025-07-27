//! Tests for `SQLite` link list operations

use crate::datastore::sqlite::links::*;
use crate::datastore::sqlite::tests::links::helpers::{
    create_internet_circuit_link, create_test_link,
};
use crate::datastore::sqlite::tests::setup::{create_test_node, setup_test_db};
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
    assert_eq!(paged_result.page, 1);
    assert_eq!(paged_result.page_size, 3);
}

#[tokio::test]
async fn test_list_links_with_pagination() {
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

    // Create 5 links
    for i in 1..=5 {
        let link = create_test_link(&format!("link-{i}"), source_node_id, Some(dest_node_id));
        create_link(&test_db.store, &link).await.unwrap();
    }

    // Test pagination
    let options = QueryOptions {
        pagination: Some(Pagination::page(1, 2).unwrap()), // Page 1, 2 items per page
        filters: vec![],
        sort: vec![],
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2); // Should get 2 items
    assert_eq!(paged_result.total_count, 5); // Total is still 5
    assert_eq!(paged_result.page, 1);
    assert_eq!(paged_result.page_size, 2);
    assert_eq!(paged_result.total_pages, 3); // ceil(5/2) = 3

    // Test second page
    let options_page2 = QueryOptions {
        pagination: Some(Pagination::page(2, 2).unwrap()), // Page 2, 2 items per page
        filters: vec![],
        sort: vec![],
    };

    let result2 = list_links(&test_db.store, &options_page2).await;
    assert!(result2.is_ok());

    let paged_result2 = result2.unwrap();
    assert_eq!(paged_result2.items.len(), 2);
    assert_eq!(paged_result2.total_count, 5);
    assert_eq!(paged_result2.page, 2);
    assert_eq!(paged_result2.page_size, 2);
}

#[tokio::test]
async fn test_list_links_with_filters() {
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

    // Create links with different circuit types
    let fiber_link = create_internet_circuit_link("fiber-link", source_node_id);
    let copper_link = create_test_link("copper-link", source_node_id, Some(dest_node_id));

    create_link(&test_db.store, &fiber_link).await.unwrap();
    create_link(&test_db.store, &copper_link).await.unwrap();

    // Filter by is_internet_circuit = true
    let options = QueryOptions {
        pagination: None,
        filters: vec![Filter {
            field: "is_internet_circuit".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(true),
        }],
        sort: vec![],
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 1); // Only the fiber link should match
    assert_eq!(paged_result.items[0].name, "fiber-link");

    // Filter by name contains "copper"
    let options2 = QueryOptions {
        pagination: None,
        filters: vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("copper".to_string()),
        }],
        sort: vec![],
    };

    let result2 = list_links(&test_db.store, &options2).await;
    assert!(result2.is_ok());

    let paged_result2 = result2.unwrap();
    assert_eq!(paged_result2.items.len(), 1);
    assert_eq!(paged_result2.items[0].name, "copper-link");
}

#[tokio::test]
async fn test_list_links_with_sorting() {
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

    // Create links with different names
    let link_c = create_test_link("c-link", source_node_id, Some(dest_node_id));
    let link_a = create_test_link("a-link", source_node_id, Some(dest_node_id));
    let link_b = create_test_link("b-link", source_node_id, Some(dest_node_id));

    // Insert in random order
    create_link(&test_db.store, &link_c).await.unwrap();
    create_link(&test_db.store, &link_a).await.unwrap();
    create_link(&test_db.store, &link_b).await.unwrap();

    // Sort by name ascending
    let options = QueryOptions {
        pagination: None,
        filters: vec![],
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }],
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.items[0].name, "a-link");
    assert_eq!(paged_result.items[1].name, "b-link");
    assert_eq!(paged_result.items[2].name, "c-link");

    // Sort by name descending
    let options_desc = QueryOptions {
        pagination: None,
        filters: vec![],
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Descending,
        }],
    };

    let result_desc = list_links(&test_db.store, &options_desc).await;
    assert!(result_desc.is_ok());

    let paged_result_desc = result_desc.unwrap();
    assert_eq!(paged_result_desc.items.len(), 3);
    assert_eq!(paged_result_desc.items[0].name, "c-link");
    assert_eq!(paged_result_desc.items[1].name, "b-link");
    assert_eq!(paged_result_desc.items[2].name, "a-link");
}
