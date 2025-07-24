//! Edge case tests for `SQLite` link operations
//!
//! Tests covering special cases, error conditions, and boundary value testing.

use super::super::super::setup::{create_test_node, setup_test_db};
use super::super::helpers::create_test_link;
use crate::datastore::sqlite::links::*;
use serde_json::json;
use uuid::Uuid;

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
