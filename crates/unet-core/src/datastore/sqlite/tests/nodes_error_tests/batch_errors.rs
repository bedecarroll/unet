//! Batch operation error tests

use super::super::setup::{create_test_node, setup_test_db};
use crate::datastore::sqlite::nodes::*;
use crate::datastore::types::{BatchOperation, DataStoreError};
use crate::models::{DeviceRole, Lifecycle, Node, Vendor};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_batch_nodes_mixed_operations() {
    let test_db = setup_test_db().await;

    // Create initial node for update/delete operations
    let existing_node_id = Uuid::new_v4();
    create_test_node(&test_db.store, existing_node_id, "existing-node")
        .await
        .unwrap();

    let new_node = Node {
        id: Uuid::new_v4(),
        name: "new-batch-node".to_string(),
        domain: "test.local".to_string(),
        fqdn: "new-batch-node.test.local".to_string(),
        vendor: Vendor::Cisco,
        model: "Batch Device".to_string(),
        role: DeviceRole::Router,
        lifecycle: Lifecycle::Live,
        management_ip: None,
        location_id: None,
        platform: None,
        version: None,
        serial_number: None,
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: json!({}),
    };

    let updated_node = Node {
        id: existing_node_id,
        name: "updated-existing-node".to_string(),
        domain: "test.local".to_string(),
        fqdn: "updated-existing-node.test.local".to_string(),
        vendor: Vendor::Juniper,
        model: "Updated Device".to_string(),
        role: DeviceRole::Switch,
        lifecycle: Lifecycle::Live,
        management_ip: None,
        location_id: None,
        platform: None,
        version: None,
        serial_number: None,
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: json!({}),
    };

    let non_existent_id = Uuid::new_v4();

    let operations = vec![
        BatchOperation::Insert(new_node.clone()),
        BatchOperation::Update(updated_node.clone()),
        BatchOperation::Delete(existing_node_id),
        BatchOperation::Delete(non_existent_id), // This should fail
    ];

    let result = batch_nodes(&test_db.store, &operations).await;

    assert!(result.is_ok());
    let batch_result = result.unwrap();

    // Should have 3 successes and 1 error
    assert_eq!(batch_result.success_count, 3);
    assert_eq!(batch_result.error_count, 1);
    assert_eq!(batch_result.errors.len(), 1);

    // The error should be for the delete of non-existent node
    let (error_index, error) = &batch_result.errors[0];
    assert_eq!(*error_index, 3); // Fourth operation (index 3)
    match error {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "Node");
            assert_eq!(id, &non_existent_id.to_string());
        }
        other => panic!("Expected NotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_batch_nodes_all_failures() {
    let test_db = setup_test_db().await;

    // Try to update and delete non-existent nodes
    let non_existent_id_1 = Uuid::new_v4();
    let non_existent_id_2 = Uuid::new_v4();

    let fake_node = Node {
        id: non_existent_id_1,
        name: "fake-node".to_string(),
        domain: "test.local".to_string(),
        fqdn: "fake-node.test.local".to_string(),
        vendor: Vendor::Cisco,
        model: "Fake Device".to_string(),
        role: DeviceRole::Router,
        lifecycle: Lifecycle::Live,
        management_ip: None,
        location_id: None,
        platform: None,
        version: None,
        serial_number: None,
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: json!({}),
    };

    let operations = vec![
        BatchOperation::Update(fake_node), // Should fail - node doesn't exist
        BatchOperation::Delete(non_existent_id_2), // Should fail - node doesn't exist
    ];

    let result = batch_nodes(&test_db.store, &operations).await;

    assert!(result.is_ok());
    let batch_result = result.unwrap();

    // All operations should fail
    assert_eq!(batch_result.success_count, 0);
    assert_eq!(batch_result.error_count, 2);
    assert_eq!(batch_result.errors.len(), 2);
}
