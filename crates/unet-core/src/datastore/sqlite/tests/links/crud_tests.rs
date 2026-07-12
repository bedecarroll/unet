//! Tests for basic CRUD operations on links

use super::super::setup::{create_test_node, setup_test_db};
use super::helpers::{create_internet_circuit_link, create_test_link};
use crate::datastore::sqlite::links::*;
use uuid::Uuid;

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

    // Should return NotFound error for nonexistent link
    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            crate::datastore::DataStoreError::NotFound { entity_type, id } => {
                assert_eq!(entity_type, "Link");
                assert_eq!(id, nonexistent_id.to_string());
            }
            _ => panic!("Expected NotFound error, got: {e:?}"),
        }
    }
}
