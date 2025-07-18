//! Tests for `SQLite` link operations

use super::setup::{create_test_node, setup_test_db};
use crate::datastore::sqlite::links::*;
use crate::datastore::types::{
    BatchOperation, Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort,
    SortDirection,
};
use crate::models::Link;
use serde_json::json;
use uuid::Uuid;

/// Helper function to create a test link
fn create_test_link(name: &str, source_node_id: Uuid, dest_node_id: Option<Uuid>) -> Link {
    Link {
        id: Uuid::new_v4(),
        name: name.to_string(),
        source_node_id,
        node_a_interface: "eth0".to_string(),
        dest_node_id,
        node_z_interface: dest_node_id.map(|_| "eth1".to_string()),
        description: Some("Test link".to_string()),
        bandwidth: Some(1_000_000_000), // 1 Gbps
        link_type: Some("ethernet".to_string()),
        is_internet_circuit: false,
        custom_data: json!({"test": "data"}),
    }
}

/// Helper function to create an internet circuit link
fn create_internet_circuit_link(name: &str, source_node_id: Uuid) -> Link {
    Link {
        id: Uuid::new_v4(),
        name: name.to_string(),
        source_node_id,
        node_a_interface: "wan0".to_string(),
        dest_node_id: None,
        node_z_interface: None,
        description: Some("Internet circuit".to_string()),
        bandwidth: Some(100_000_000), // 100 Mbps
        link_type: Some("fiber".to_string()),
        is_internet_circuit: true,
        custom_data: json!({"provider": "ISP1"}),
    }
}

#[tokio::test]
async fn test_create_link_success() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create the nodes first to satisfy foreign key constraints
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest-node")
        .await
        .unwrap();

    let original_link = create_test_link("test-link", source_node_id, Some(dest_node_id));

    let result = create_link(&test_db.store, &original_link).await;
    if let Err(e) = &result {
        panic!("Failed to create link: {e:?}");
    }

    let created_link = result.unwrap();
    assert_eq!(created_link.id, original_link.id);
    assert_eq!(created_link.name, original_link.name);
    assert_eq!(created_link.source_node_id, original_link.source_node_id);
    assert_eq!(created_link.dest_node_id, original_link.dest_node_id);
    assert_eq!(created_link.bandwidth, original_link.bandwidth);
    assert_eq!(
        created_link.is_internet_circuit,
        original_link.is_internet_circuit
    );
}

#[tokio::test]
async fn test_create_internet_circuit_link() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create the source node first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    let original_link = create_internet_circuit_link("internet-circuit", source_node_id);

    let result = create_link(&test_db.store, &original_link).await;
    assert!(result.is_ok());

    let created_link = result.unwrap();
    assert_eq!(created_link.dest_node_id, None);
    assert_eq!(created_link.node_z_interface, None);
    assert!(created_link.is_internet_circuit);
}

#[tokio::test]
async fn test_get_link_existing() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create the nodes first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest-node")
        .await
        .unwrap();

    let original_link = create_test_link("get-test-link", source_node_id, Some(dest_node_id));
    create_link(&test_db.store, &original_link).await.unwrap();

    let result = get_link(&test_db.store, &original_link.id).await;
    assert!(result.is_ok());

    let retrieved_link = result.unwrap();
    assert!(retrieved_link.is_some());
    let link = retrieved_link.unwrap();
    assert_eq!(link.id, original_link.id);
    assert_eq!(link.name, original_link.name);
}

#[tokio::test]
async fn test_get_link_nonexistent() {
    let test_db = setup_test_db().await;
    let nonexistent_id = Uuid::new_v4();

    let result = get_link(&test_db.store, &nonexistent_id).await;
    assert!(result.is_ok());

    let retrieved_link = result.unwrap();
    assert!(retrieved_link.is_none());
}

#[tokio::test]
async fn test_update_link_success() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create the nodes first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest-node")
        .await
        .unwrap();

    let mut original_link =
        create_test_link("update-test-link", source_node_id, Some(dest_node_id));
    create_link(&test_db.store, &original_link).await.unwrap();

    // Modify the link
    original_link.name = "updated-link-name".to_string();
    original_link.bandwidth = Some(10_000_000_000); // 10 Gbps
    original_link.description = Some("Updated description".to_string());

    let result = update_link(&test_db.store, &original_link).await;
    assert!(result.is_ok());

    let updated_link = result.unwrap();
    assert_eq!(updated_link.name, "updated-link-name");
    assert_eq!(updated_link.bandwidth, Some(10_000_000_000));
    assert_eq!(
        updated_link.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
async fn test_delete_link_success() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    // Create the nodes first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();
    create_test_node(&test_db.store, dest_node_id, "dest-node")
        .await
        .unwrap();

    let original_link = create_test_link("delete-test-link", source_node_id, Some(dest_node_id));
    create_link(&test_db.store, &original_link).await.unwrap();

    let result = delete_link(&test_db.store, &original_link.id).await;
    assert!(result.is_ok());

    // Verify the link is deleted
    let get_result = get_link(&test_db.store, &original_link.id).await;
    assert!(get_result.is_ok());
    assert!(get_result.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_link_nonexistent() {
    let test_db = setup_test_db().await;
    let nonexistent_id = Uuid::new_v4();

    let result = delete_link(&test_db.store, &nonexistent_id).await;
    assert!(result.is_err());
}

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
async fn test_list_links_with_pagination() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create the source node first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    // Create multiple links
    for i in 0..5 {
        let dest_node_id = Uuid::new_v4();
        create_test_node(&test_db.store, dest_node_id, &format!("dest-node-{i}"))
            .await
            .unwrap();
        let link = create_test_link(&format!("link-{i}"), source_node_id, Some(dest_node_id));
        create_link(&test_db.store, &link).await.unwrap();
    }

    let options = QueryOptions {
        pagination: Some(Pagination {
            offset: 1,
            limit: 2,
        }),
        ..Default::default()
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2);
    assert_eq!(paged_result.total_count, 5);
}

#[tokio::test]
async fn test_list_links_with_sorting() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create the source node first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    // Create links with different names to test sorting
    let link_names = ["zebra-link", "alpha-link", "beta-link"];
    for (i, name) in link_names.iter().enumerate() {
        let dest_node_id = Uuid::new_v4();
        create_test_node(&test_db.store, dest_node_id, &format!("dest-node-{i}"))
            .await
            .unwrap();
        let link = create_test_link(name, source_node_id, Some(dest_node_id));
        create_link(&test_db.store, &link).await.unwrap();
    }

    let options = QueryOptions {
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }],
        ..Default::default()
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    let sorted_names: Vec<String> = paged_result.items.iter().map(|l| l.name.clone()).collect();
    assert_eq!(sorted_names, vec!["alpha-link", "beta-link", "zebra-link"]);
}

#[tokio::test]
async fn test_get_links_for_node() {
    let test_db = setup_test_db().await;
    let primary_node_id = Uuid::new_v4();
    let secondary_node_id = Uuid::new_v4();
    let tertiary_node_id = Uuid::new_v4();

    // Create all nodes first
    create_test_node(&test_db.store, primary_node_id, "node-a")
        .await
        .unwrap();
    create_test_node(&test_db.store, secondary_node_id, "node-b")
        .await
        .unwrap();
    create_test_node(&test_db.store, tertiary_node_id, "node-c")
        .await
        .unwrap();

    // Create links involving primary_node
    let link1 = create_test_link("link1", primary_node_id, Some(secondary_node_id));
    let link2 = create_test_link("link2", secondary_node_id, Some(primary_node_id));
    let link3 = create_test_link("link3", secondary_node_id, Some(tertiary_node_id)); // Should not be included

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();
    create_link(&test_db.store, &link3).await.unwrap();

    let result = get_links_for_node(&test_db.store, &primary_node_id).await;
    assert!(result.is_ok());

    let node_links = result.unwrap();
    assert_eq!(node_links.len(), 2);

    let link_ids: std::collections::HashSet<Uuid> = node_links.iter().map(|l| l.id).collect();
    assert!(link_ids.contains(&link1.id));
    assert!(link_ids.contains(&link2.id));
    assert!(!link_ids.contains(&link3.id));
}

#[tokio::test]
async fn test_get_links_between_nodes() {
    let test_db = setup_test_db().await;
    let first_node_id = Uuid::new_v4();
    let second_node_id = Uuid::new_v4();
    let third_node_id = Uuid::new_v4();

    // Create all nodes first
    create_test_node(&test_db.store, first_node_id, "node-a")
        .await
        .unwrap();
    create_test_node(&test_db.store, second_node_id, "node-b")
        .await
        .unwrap();
    create_test_node(&test_db.store, third_node_id, "node-c")
        .await
        .unwrap();

    // Create links between different node pairs
    let link1 = create_test_link("link1", first_node_id, Some(second_node_id));
    let link2 = create_test_link("link2", second_node_id, Some(first_node_id));
    let link3 = create_test_link("link3", first_node_id, Some(third_node_id)); // Different pair

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();
    create_link(&test_db.store, &link3).await.unwrap();

    let result = get_links_between_nodes(&test_db.store, &first_node_id, &second_node_id).await;
    assert!(result.is_ok());

    let connecting_links = result.unwrap();
    assert_eq!(connecting_links.len(), 2);

    let link_ids: std::collections::HashSet<Uuid> = connecting_links.iter().map(|l| l.id).collect();
    assert!(link_ids.contains(&link1.id));
    assert!(link_ids.contains(&link2.id));
    assert!(!link_ids.contains(&link3.id));
}

#[tokio::test]
async fn test_batch_operations_mixed() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create the source node first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    // Create some initial links with nodes
    let link1_dest = Uuid::new_v4();
    let link2_dest = Uuid::new_v4();
    create_test_node(&test_db.store, link1_dest, "link1-dest")
        .await
        .unwrap();
    create_test_node(&test_db.store, link2_dest, "link2-dest")
        .await
        .unwrap();

    let link1 = create_test_link("batch-link1", source_node_id, Some(link1_dest));
    let link2 = create_test_link("batch-link2", source_node_id, Some(link2_dest));
    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();

    // Prepare batch operations
    let new_link_dest = Uuid::new_v4();
    create_test_node(&test_db.store, new_link_dest, "new-link-dest")
        .await
        .unwrap();
    let new_link = create_test_link("batch-new-link", source_node_id, Some(new_link_dest));
    let mut updated_link = link1.clone();
    updated_link.name = "batch-updated-link".to_string();

    let operations = vec![
        BatchOperation::Insert(new_link.clone()),
        BatchOperation::Update(updated_link.clone()),
        BatchOperation::Delete(link2.id),
    ];

    let result = batch_links(&test_db.store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 3);
    assert_eq!(batch_result.error_count, 0);

    // Verify operations were applied
    let new_link_result = get_link(&test_db.store, &new_link.id).await.unwrap();
    assert!(new_link_result.is_some());

    let updated_link_result = get_link(&test_db.store, &updated_link.id).await.unwrap();
    assert_eq!(updated_link_result.unwrap().name, "batch-updated-link");

    let deleted_link_result = get_link(&test_db.store, &link2.id).await.unwrap();
    assert!(deleted_link_result.is_none());
}

#[tokio::test]
async fn test_batch_operations_with_errors() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create the source node first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    // Create a link to update
    let existing_link_dest = Uuid::new_v4();
    create_test_node(&test_db.store, existing_link_dest, "existing-dest")
        .await
        .unwrap();
    let existing_link = create_test_link("existing-link", source_node_id, Some(existing_link_dest));
    create_link(&test_db.store, &existing_link).await.unwrap();

    let valid_new_link_dest = Uuid::new_v4();
    create_test_node(&test_db.store, valid_new_link_dest, "valid-dest")
        .await
        .unwrap();
    let valid_new_link =
        create_test_link("valid-new-link", source_node_id, Some(valid_new_link_dest));
    let nonexistent_id = Uuid::new_v4();

    let operations = vec![
        BatchOperation::Insert(valid_new_link.clone()),
        BatchOperation::Delete(nonexistent_id), // This should fail
    ];

    let result = batch_links(&test_db.store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 1);
    assert_eq!(batch_result.error_count, 1);
    assert_eq!(batch_result.errors.len(), 1);
    assert_eq!(batch_result.errors[0].0, 1); // Error occurred at index 1
}

#[tokio::test]
async fn test_list_links_with_filters() {
    let test_db = setup_test_db().await;
    let source_node_id = Uuid::new_v4();

    // Create the source node first
    create_test_node(&test_db.store, source_node_id, "source-node")
        .await
        .unwrap();

    // Create links with different properties
    let internet_link = create_internet_circuit_link("internet-circuit", source_node_id);
    let ethernet_link_dest = Uuid::new_v4();
    create_test_node(&test_db.store, ethernet_link_dest, "ethernet-dest")
        .await
        .unwrap();
    let mut ethernet_link =
        create_test_link("ethernet-link", source_node_id, Some(ethernet_link_dest));
    ethernet_link.is_internet_circuit = false;

    create_link(&test_db.store, &internet_link).await.unwrap();
    create_link(&test_db.store, &ethernet_link).await.unwrap();

    // Filter for internet circuits only
    let options = QueryOptions {
        filters: vec![Filter {
            field: "is_internet_circuit".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(true),
        }],
        ..Default::default()
    };

    let result = list_links(&test_db.store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 1);
    assert!(paged_result.items[0].is_internet_circuit);
    assert_eq!(paged_result.items[0].name, "internet-circuit");
}
