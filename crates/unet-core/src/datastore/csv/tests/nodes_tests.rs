//! Tests for CSV nodes operations

use super::super::nodes::*;
use super::super::store::CsvStore;
use crate::datastore::types::{
    BatchOperation, DataStoreError, Filter, FilterOperation, FilterValue, Pagination, QueryOptions,
    Sort, SortDirection,
};
use crate::models::{DeviceRole, Lifecycle, Node, NodeBuilder, Vendor};
use tempfile::TempDir;
use uuid::Uuid;

async fn setup_test_store() -> (CsvStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let store = CsvStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    (store, temp_dir)
}

fn create_test_node_with_name(name: &str) -> Node {
    NodeBuilder::new()
        .name(name.to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("Test Model".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build()
        .unwrap()
}

fn create_test_node_with_location(name: &str, location_id: Option<Uuid>) -> Node {
    let mut builder = NodeBuilder::new()
        .name(name.to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("Test Model".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live);

    if let Some(loc_id) = location_id {
        builder = builder.location_id(loc_id);
    }

    builder.build().unwrap()
}

#[tokio::test]
async fn test_create_node_success() {
    let (store, _temp_dir) = setup_test_store().await;
    let node = create_test_node_with_name("test-node");

    let result = create_node(&store, &node).await;
    assert!(result.is_ok());

    let created_node = result.unwrap();
    assert_eq!(created_node.name, "test-node");
    assert_eq!(created_node.id, node.id);
}

#[tokio::test]
async fn test_create_node_duplicate_id() {
    let (store, _temp_dir) = setup_test_store().await;
    let node = create_test_node_with_name("test-node");

    // Create first node
    let result1 = create_node(&store, &node).await;
    assert!(result1.is_ok());

    // Try to create node with same ID
    let result2 = create_node(&store, &node).await;
    assert!(result2.is_err());

    match result2.unwrap_err() {
        DataStoreError::ConstraintViolation { message } => {
            assert!(message.contains("already exists"));
        }
        _ => panic!("Expected ConstraintViolation"),
    }
}

#[tokio::test]
async fn test_get_node_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let node = create_test_node_with_name("test-node");

    // Create node first
    create_node(&store, &node).await.unwrap();

    // Get node
    let result = get_node(&store, &node.id).await;
    assert!(result.is_ok());

    let found_node = result.unwrap();
    assert!(found_node.is_some());
    assert_eq!(found_node.unwrap().name, "test-node");
}

#[tokio::test]
async fn test_get_node_not_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let non_existent_id = Uuid::new_v4();

    let result = get_node(&store, &non_existent_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_list_nodes_empty() {
    let (store, _temp_dir) = setup_test_store().await;
    let options = QueryOptions::default();

    let result = list_nodes(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 0);
    assert_eq!(paged_result.total_count, 0);
}

#[tokio::test]
async fn test_list_nodes_multiple() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create multiple nodes
    let node1 = create_test_node_with_name("node-1");
    let node2 = create_test_node_with_name("node-2");
    let node3 = create_test_node_with_name("node-3");

    create_node(&store, &node1).await.unwrap();
    create_node(&store, &node2).await.unwrap();
    create_node(&store, &node3).await.unwrap();

    let options = QueryOptions::default();
    let result = list_nodes(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.total_count, 3);
}

#[tokio::test]
async fn test_list_nodes_with_pagination() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create multiple nodes
    for i in 1..=5 {
        let node = create_test_node_with_name(&format!("node-{i}"));
        create_node(&store, &node).await.unwrap();
    }

    let options = QueryOptions {
        pagination: Some(Pagination {
            offset: 1,
            limit: 2,
        }),
        ..Default::default()
    };

    let result = list_nodes(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2);
    assert_eq!(paged_result.total_count, 5);
}

#[tokio::test]
async fn test_list_nodes_with_name_filter() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create nodes with different names
    let node1 = create_test_node_with_name("router-1");
    let node2 = create_test_node_with_name("switch-1");
    let node3 = create_test_node_with_name("router-2");

    create_node(&store, &node1).await.unwrap();
    create_node(&store, &node2).await.unwrap();
    create_node(&store, &node3).await.unwrap();

    let options = QueryOptions {
        filters: vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("router".to_string()),
        }],
        ..Default::default()
    };

    let result = list_nodes(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2);
    assert!(paged_result.items.iter().all(|n| n.name.contains("router")));
}

#[tokio::test]
async fn test_list_nodes_with_sorting() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create nodes with names in non-alphabetical order
    let node1 = create_test_node_with_name("charlie");
    let node2 = create_test_node_with_name("alpha");
    let node3 = create_test_node_with_name("bravo");

    create_node(&store, &node1).await.unwrap();
    create_node(&store, &node2).await.unwrap();
    create_node(&store, &node3).await.unwrap();

    let options = QueryOptions {
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }],
        ..Default::default()
    };

    let result = list_nodes(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.items[0].name, "alpha");
    assert_eq!(paged_result.items[1].name, "bravo");
    assert_eq!(paged_result.items[2].name, "charlie");
}

#[tokio::test]
async fn test_list_nodes_with_descending_sort() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create nodes with names in non-alphabetical order
    let node1 = create_test_node_with_name("alpha");
    let node2 = create_test_node_with_name("bravo");
    let node3 = create_test_node_with_name("charlie");

    create_node(&store, &node1).await.unwrap();
    create_node(&store, &node2).await.unwrap();
    create_node(&store, &node3).await.unwrap();

    let options = QueryOptions {
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Descending,
        }],
        ..Default::default()
    };

    let result = list_nodes(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.items[0].name, "charlie");
    assert_eq!(paged_result.items[1].name, "bravo");
    assert_eq!(paged_result.items[2].name, "alpha");
}

#[tokio::test]
async fn test_update_node_success() {
    let (store, _temp_dir) = setup_test_store().await;
    let mut node = create_test_node_with_name("original-name");

    // Create node first
    create_node(&store, &node).await.unwrap();

    // Update node
    node.name = "updated-name".to_string();
    node.fqdn = format!("{}.{}", node.name, node.domain); // Update FQDN to match name.domain
    let result = update_node(&store, &node).await;
    assert!(result.is_ok());

    let updated_node = result.unwrap();
    assert_eq!(updated_node.name, "updated-name");

    // Verify in store
    let stored_node = get_node(&store, &node.id).await.unwrap().unwrap();
    assert_eq!(stored_node.name, "updated-name");
}

#[tokio::test]
async fn test_update_node_not_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let node = create_test_node_with_name("test-node");

    // Try to update non-existent node
    let result = update_node(&store, &node).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "node");
            assert_eq!(id, node.id.to_string());
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_delete_node_success() {
    let (store, _temp_dir) = setup_test_store().await;
    let node = create_test_node_with_name("test-node");

    // Create node first
    create_node(&store, &node).await.unwrap();

    // Delete node
    let result = delete_node(&store, &node.id).await;
    assert!(result.is_ok());

    // Verify node is gone
    let stored_node = get_node(&store, &node.id).await.unwrap();
    assert!(stored_node.is_none());
}

#[tokio::test]
async fn test_delete_node_not_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let non_existent_id = Uuid::new_v4();

    let result = delete_node(&store, &non_existent_id).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "node");
            assert_eq!(id, non_existent_id.to_string());
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_get_nodes_by_location() {
    let (store, _temp_dir) = setup_test_store().await;
    let location_id = Uuid::new_v4();

    // Create nodes with and without location
    let node1 = create_test_node_with_location("node-1", Some(location_id));
    let node2 = create_test_node_with_location("node-2", None);
    let node3 = create_test_node_with_location("node-3", Some(location_id));

    create_node(&store, &node1).await.unwrap();
    create_node(&store, &node2).await.unwrap();
    create_node(&store, &node3).await.unwrap();

    let result = get_nodes_by_location(&store, &location_id).await;
    assert!(result.is_ok());

    let location_nodes = result.unwrap();
    assert_eq!(location_nodes.len(), 2);
    assert!(
        location_nodes
            .iter()
            .all(|n| n.location_id == Some(location_id))
    );
}

#[tokio::test]
async fn test_search_nodes_by_name() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create nodes with different names
    let node1 = create_test_node_with_name("router-primary");
    let node2 = create_test_node_with_name("switch-backup");
    let node3 = create_test_node_with_name("router-secondary");
    let node4 = create_test_node_with_name("firewall-main");

    create_node(&store, &node1).await.unwrap();
    create_node(&store, &node2).await.unwrap();
    create_node(&store, &node3).await.unwrap();
    create_node(&store, &node4).await.unwrap();

    // Search for "router"
    let result = search_nodes_by_name(&store, "router").await;
    assert!(result.is_ok());

    let search_results = result.unwrap();
    assert_eq!(search_results.len(), 2);
    assert!(search_results.iter().all(|n| n.name.contains("router")));
}

#[tokio::test]
async fn test_search_nodes_by_name_case_insensitive() {
    let (store, _temp_dir) = setup_test_store().await;

    let node = create_test_node_with_name("Router-Primary");
    create_node(&store, &node).await.unwrap();

    // Search with different case
    let result = search_nodes_by_name(&store, "router").await;
    assert!(result.is_ok());

    let nodes = result.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].name, "Router-Primary");
}

#[tokio::test]
async fn test_batch_nodes_mixed_operations() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create initial node
    let existing_node = create_test_node_with_name("existing-node");
    create_node(&store, &existing_node).await.unwrap();

    // Prepare batch operations
    let new_node = create_test_node_with_name("new-node");
    let mut updated_node = existing_node.clone();
    updated_node.name = "updated-existing-node".to_string();
    updated_node.fqdn = format!("{}.{}", updated_node.name, updated_node.domain); // Update FQDN to match name.domain

    let operations = vec![
        BatchOperation::Insert(new_node.clone()),
        BatchOperation::Update(updated_node.clone()),
        BatchOperation::Delete(Uuid::new_v4()), // This will fail
    ];

    let result = batch_nodes(&store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 2);
    assert_eq!(batch_result.error_count, 1);
    assert_eq!(batch_result.errors.len(), 1);

    // Verify successful operations
    assert!(get_node(&store, &new_node.id).await.unwrap().is_some());
    let stored_updated = get_node(&store, &existing_node.id).await.unwrap().unwrap();
    assert_eq!(stored_updated.name, "updated-existing-node");
}

#[tokio::test]
async fn test_batch_nodes_all_success() {
    let (store, _temp_dir) = setup_test_store().await;

    let node1 = create_test_node_with_name("node-1");
    let node2 = create_test_node_with_name("node-2");

    let operations = vec![
        BatchOperation::Insert(node1.clone()),
        BatchOperation::Insert(node2.clone()),
    ];

    let result = batch_nodes(&store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 2);
    assert_eq!(batch_result.error_count, 0);
    assert!(batch_result.errors.is_empty());
}

#[tokio::test]
async fn test_batch_nodes_empty_operations() {
    let (store, _temp_dir) = setup_test_store().await;

    let operations = vec![];
    let result = batch_nodes(&store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 0);
    assert_eq!(batch_result.error_count, 0);
}
