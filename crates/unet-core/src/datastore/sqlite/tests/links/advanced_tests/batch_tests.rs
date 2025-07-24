//! Batch operation tests for `SQLite` link operations
//!
//! Tests covering batch inserts, updates, deletes, mixed operations, and error handling.

use super::super::super::setup::{create_test_node, setup_test_db};
use super::super::helpers::create_test_link;
use crate::datastore::sqlite::links::*;
use crate::datastore::types::BatchOperation;
use uuid::Uuid;

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
