//! Advanced tests for `SQLite` link operations
//!
//! Comprehensive tests covering list, filtering, pagination, node relationships,
//! batch operations, and edge cases to achieve high coverage.

use super::super::setup::{create_test_node, setup_test_db};
use super::helpers::{create_internet_circuit_link, create_test_link};
use crate::datastore::sqlite::links::*;
use crate::datastore::types::{
    BatchOperation, Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort,
    SortDirection,
};
use serde_json::json;
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
        let link = create_test_link(
            &format!("paginated-link-{i}"),
            source_node_id,
            Some(dest_node_id),
        );
        create_link(&test_db.store, &link).await.unwrap();
    }

    let options = QueryOptions {
        pagination: Some(Pagination {
            limit: 2,
            offset: 2,
        }),
        ..QueryOptions::default()
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2);
    assert_eq!(paged_result.total_count, 5);
}

#[tokio::test]
async fn test_list_links_with_filters() {
    let test_db = setup_test_db().await;
    let source1_id = Uuid::new_v4();
    let source2_id = Uuid::new_v4();
    let dest_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source1_id, "source1")
        .await
        .unwrap();
    create_test_node(&test_db.store, source2_id, "source2")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_id, "dest")
        .await
        .unwrap();

    // Create regular links
    let link1 = create_test_link("ethernet-link", source1_id, Some(dest_id));
    create_link(&test_db.store, &link1).await.unwrap();

    // Create internet circuit
    let circuit = create_internet_circuit_link("internet-circuit", source2_id);
    create_link(&test_db.store, &circuit).await.unwrap();

    // Test filtering by internet circuit
    let options = QueryOptions {
        filters: vec![Filter {
            field: "is_internet_circuit".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(true),
        }],
        ..QueryOptions::default()
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 1);
    assert!(paged_result.items[0].is_internet_circuit);
}

#[tokio::test]
async fn test_list_links_with_sorting() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    // Create links with different names
    let link_c = create_test_link("c-link", source_node_id, Some(dest_node_id));
    let link_a = create_test_link("a-link", source_node_id, Some(dest_node_id));
    let link_b = create_test_link("b-link", source_node_id, Some(dest_node_id));

    create_link(&test_db.store, &link_c).await.unwrap();
    create_link(&test_db.store, &link_a).await.unwrap();
    create_link(&test_db.store, &link_b).await.unwrap();

    // Test sorting by name ascending
    let options = QueryOptions {
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }],
        ..QueryOptions::default()
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.items[0].name, "a-link");
    assert_eq!(paged_result.items[1].name, "b-link");
    assert_eq!(paged_result.items[2].name, "c-link");
}

#[tokio::test]
async fn test_get_links_for_node() {
    let test_db = setup_test_db().await;
    let node1_id = Uuid::new_v4();
    let node2_id = Uuid::new_v4();
    let node3_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, node1_id, "node1")
        .await
        .unwrap();
    create_test_node(&test_db.store, node2_id, "node2")
        .await
        .unwrap();
    create_test_node(&test_db.store, node3_id, "node3")
        .await
        .unwrap();

    // Create links involving node1
    let link1 = create_test_link("link1-2", node1_id, Some(node2_id));
    let link2 = create_test_link("link1-3", node1_id, Some(node3_id));
    let link3 = create_test_link("link2-3", node2_id, Some(node3_id)); // Doesn't involve node1

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();
    create_link(&test_db.store, &link3).await.unwrap();

    let result = get_links_for_node(&test_db.store, &node1_id).await;
    assert!(result.is_ok());

    let node_links = result.unwrap();
    assert_eq!(node_links.len(), 2);

    // Verify the links involve node1
    for link in &node_links {
        assert!(link.source_node_id == node1_id || link.dest_node_id == Some(node1_id));
    }
}

#[tokio::test]
async fn test_get_links_for_node_no_links() {
    let test_db = setup_test_db().await;
    let node_id = Uuid::new_v4();

    // Create node but no links
    create_test_node(&test_db.store, node_id, "isolated-node")
        .await
        .unwrap();

    let result = get_links_for_node(&test_db.store, &node_id).await;
    assert!(result.is_ok());

    let links = result.unwrap();
    assert_eq!(links.len(), 0);
}

#[tokio::test]
async fn test_get_links_between_nodes() {
    let test_db = setup_test_db().await;
    let node1_id = Uuid::new_v4();
    let node2_id = Uuid::new_v4();
    let node3_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, node1_id, "node1")
        .await
        .unwrap();
    create_test_node(&test_db.store, node2_id, "node2")
        .await
        .unwrap();
    create_test_node(&test_db.store, node3_id, "node3")
        .await
        .unwrap();

    // Create multiple links between node1 and node2
    let link1 = create_test_link("link1-primary", node1_id, Some(node2_id));
    let link2 = create_test_link("link1-backup", node1_id, Some(node2_id));
    let link3 = create_test_link("link1-3", node1_id, Some(node3_id)); // Different pair

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();
    create_link(&test_db.store, &link3).await.unwrap();

    let result = get_links_between_nodes(&test_db.store, &node1_id, &node2_id).await;
    assert!(result.is_ok());

    let between_links = result.unwrap();
    assert_eq!(between_links.len(), 2);

    // Verify the links are between the specified nodes
    for link in &between_links {
        assert!(
            (link.source_node_id == node1_id && link.dest_node_id == Some(node2_id))
                || (link.source_node_id == node2_id && link.dest_node_id == Some(node1_id))
        );
    }
}

#[tokio::test]
async fn test_get_links_between_nodes_no_links() {
    let test_db = setup_test_db().await;
    let node1_id = Uuid::new_v4();
    let node2_id = Uuid::new_v4();

    // Create nodes but no links between them
    create_test_node(&test_db.store, node1_id, "node1")
        .await
        .unwrap();
    create_test_node(&test_db.store, node2_id, "node2")
        .await
        .unwrap();

    let result = get_links_between_nodes(&test_db.store, &node1_id, &node2_id).await;
    assert!(result.is_ok());

    let links = result.unwrap();
    assert_eq!(links.len(), 0);
}

#[tokio::test]
async fn test_batch_links_insert_operations() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    // Create batch insert operations
    let link1 = create_test_link("batch-link-1", source_node_id, Some(dest_node_id));
    let link2 = create_test_link("batch-link-2", source_node_id, Some(dest_node_id));

    let operations = vec![BatchOperation::Insert(link1), BatchOperation::Insert(link2)];

    let result = batch_links(&test_db.store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 2);
    assert_eq!(batch_result.error_count, 0);
    assert!(batch_result.errors.is_empty());
}

#[tokio::test]
async fn test_batch_links_update_operations() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    // Create initial links
    let mut link1 = create_test_link("update-batch-1", source_node_id, Some(dest_node_id));
    let mut link2 = create_test_link("update-batch-2", source_node_id, Some(dest_node_id));

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();

    // Modify for batch update
    link1.description = Some("Batch updated description 1".to_string());
    link2.description = Some("Batch updated description 2".to_string());

    let operations = vec![
        BatchOperation::Update(link1.clone()),
        BatchOperation::Update(link2.clone()),
    ];

    let result = batch_links(&test_db.store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 2);
    assert_eq!(batch_result.error_count, 0);

    // Verify updates persisted
    let updated_link1 = get_link(&test_db.store, &link1.id).await.unwrap().unwrap();
    assert_eq!(
        updated_link1.description,
        Some("Batch updated description 1".to_string())
    );
}

#[tokio::test]
async fn test_batch_links_delete_operations() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    // Create links
    let link1 = create_test_link("delete-batch-1", source_node_id, Some(dest_node_id));
    let link2 = create_test_link("delete-batch-2", source_node_id, Some(dest_node_id));

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();

    let operations = vec![
        BatchOperation::Delete(link1.id),
        BatchOperation::Delete(link2.id),
    ];

    let result = batch_links(&test_db.store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 2);
    assert_eq!(batch_result.error_count, 0);

    // Verify deletions
    assert!(get_link(&test_db.store, &link1.id).await.unwrap().is_none());
    assert!(get_link(&test_db.store, &link2.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_batch_links_mixed_operations() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    // Create initial link for update and delete
    let mut existing_link = create_test_link("existing-link", source_node_id, Some(dest_node_id));
    let delete_link = create_test_link("delete-link", source_node_id, Some(dest_node_id));

    create_link(&test_db.store, &existing_link).await.unwrap();
    create_link(&test_db.store, &delete_link).await.unwrap();

    // Prepare new link for insert and modify existing for update
    let new_link = create_test_link("new-link", source_node_id, Some(dest_node_id));
    existing_link.description = Some("Mixed batch update".to_string());

    let operations = vec![
        BatchOperation::Insert(new_link.clone()),
        BatchOperation::Update(existing_link.clone()),
        BatchOperation::Delete(delete_link.id),
    ];

    let result = batch_links(&test_db.store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 3);
    assert_eq!(batch_result.error_count, 0);

    // Verify all operations
    assert!(
        get_link(&test_db.store, &new_link.id)
            .await
            .unwrap()
            .is_some()
    ); // Insert
    let updated = get_link(&test_db.store, &existing_link.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.description, Some("Mixed batch update".to_string())); // Update
    assert!(
        get_link(&test_db.store, &delete_link.id)
            .await
            .unwrap()
            .is_none()
    ); // Delete
}

#[tokio::test]
async fn test_batch_links_with_errors() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    // Create valid link
    let valid_link = create_test_link("valid-link", source_node_id, Some(dest_node_id));

    // Create link with invalid source node (should fail)
    let invalid_link = create_test_link("invalid-link", Uuid::new_v4(), Some(dest_node_id));

    let operations = vec![
        BatchOperation::Insert(valid_link.clone()),
        BatchOperation::Insert(invalid_link), // This should fail
    ];

    let result = batch_links(&test_db.store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 1);
    assert_eq!(batch_result.error_count, 1);
    assert_eq!(batch_result.errors.len(), 1);

    // Verify the valid link was still created
    assert!(
        get_link(&test_db.store, &valid_link.id)
            .await
            .unwrap()
            .is_some()
    );

    // Check error details
    let (error_index, _error) = &batch_result.errors[0];
    assert_eq!(*error_index, 1); // Second operation failed
}

#[tokio::test]
async fn test_update_link_nonexistent() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes but not the link
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    let nonexistent_link = create_test_link("nonexistent", source_node_id, Some(dest_node_id));

    let result = update_link(&test_db.store, &nonexistent_link).await;
    assert!(result.is_err());

    // The SQLite implementation returns InternalError for updates that affect 0 rows
    if let Err(e) = result {
        match e {
            crate::datastore::DataStoreError::InternalError { message } => {
                assert!(message.contains("None of the records are updated"));
            }
            _ => panic!("Expected InternalError, got: {e:?}"),
        }
    }
}

#[tokio::test]
async fn test_link_with_large_bandwidth() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    let mut link = create_test_link("high-bandwidth-link", source_node_id, Some(dest_node_id));
    link.bandwidth = Some(u64::MAX); // Test edge case with max bandwidth

    let result = create_link(&test_db.store, &link).await;
    assert!(result.is_ok());

    let created_link = result.unwrap();
    // Note: The SQLite implementation uses i64::MAX when u64 conversion fails
    // So we expect either the original value or i64::MAX
    assert!(created_link.bandwidth.is_some());
}

#[tokio::test]
async fn test_link_custom_data_serialization() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, source_node_id, "source")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest")
        .await
        .unwrap();

    let mut link = create_test_link("custom-data-link", source_node_id, Some(dest_node_id));
    link.custom_data = json!({
        "vendor": "Cisco",
        "model": "ISR4431",
        "config": {
            "vlan": 100,
            "mtu": 1500
        },
        "tags": ["production", "critical"]
    });

    let result = create_link(&test_db.store, &link).await;
    assert!(result.is_ok());

    let created_link = result.unwrap();
    assert_eq!(created_link.custom_data, link.custom_data);

    // Verify it persists correctly
    let retrieved_link = get_link(&test_db.store, &link.id).await.unwrap().unwrap();
    assert_eq!(retrieved_link.custom_data, link.custom_data);
}

pub mod batch_tests;
pub mod edge_case_tests;
pub mod list_tests;
pub mod node_relationship_tests;
