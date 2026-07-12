//! Tests for `EmptyMockDataStore` implementation

use super::super::mock_datastores::mocks::EmptyMockDataStore;
use super::shared::*;
use crate::datastore::{BatchOperation, DataStore};
use uuid::Uuid;

#[tokio::test]
async fn test_empty_mock_datastore_name() {
    let store = EmptyMockDataStore;
    assert_eq!(store.name(), "empty_mock");
}

#[tokio::test]
async fn test_empty_mock_datastore_health_check() {
    let store = EmptyMockDataStore;
    let result = store.health_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_begin_transaction() {
    let store = EmptyMockDataStore;
    let result = store.begin_transaction().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_mock_datastore_create_node() {
    let store = EmptyMockDataStore;
    let node = create_test_node();
    let result = store.create_node(&node).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_get_node() {
    let store = EmptyMockDataStore;
    let id = Uuid::new_v4();
    let result = store.get_node(&id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_empty_mock_datastore_list_nodes() {
    let store = EmptyMockDataStore;
    let options = create_query_options();
    let result = store.list_nodes(&options).await;
    assert!(result.is_ok());
    let paged = result.unwrap();
    assert_eq!(paged.items.len(), 0);
    assert_eq!(paged.total_count, 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_update_node() {
    let store = EmptyMockDataStore;
    let node = create_test_node();
    let result = store.update_node(&node).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_delete_node() {
    let store = EmptyMockDataStore;
    let id = Uuid::new_v4();
    let result = store.delete_node(&id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_get_nodes_by_location() {
    let store = EmptyMockDataStore;
    let location_id = Uuid::new_v4();
    let result = store.get_nodes_by_location(&location_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_search_nodes_by_name() {
    let store = EmptyMockDataStore;
    let result = store.search_nodes_by_name("test").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_create_link() {
    let store = EmptyMockDataStore;
    let link = create_test_link();
    let result = store.create_link(&link).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_get_link() {
    let store = EmptyMockDataStore;
    let id = Uuid::new_v4();
    let result = store.get_link(&id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_empty_mock_datastore_list_links() {
    let store = EmptyMockDataStore;
    let options = create_query_options();
    let result = store.list_links(&options).await;
    assert!(result.is_ok());
    let paged = result.unwrap();
    assert_eq!(paged.items.len(), 0);
    assert_eq!(paged.total_count, 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_update_link() {
    let store = EmptyMockDataStore;
    let link = create_test_link();
    let result = store.update_link(&link).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_delete_link() {
    let store = EmptyMockDataStore;
    let id = Uuid::new_v4();
    let result = store.delete_link(&id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_get_links_for_node() {
    let store = EmptyMockDataStore;
    let node_id = Uuid::new_v4();
    let result = store.get_links_for_node(&node_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_get_links_between_nodes() {
    let store = EmptyMockDataStore;
    let first_node = Uuid::new_v4();
    let second_node = Uuid::new_v4();
    let result = store
        .get_links_between_nodes(&first_node, &second_node)
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_create_location() {
    let store = EmptyMockDataStore;
    let location = create_test_location();
    let result = store.create_location(&location).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_get_location() {
    let store = EmptyMockDataStore;
    let id = Uuid::new_v4();
    let result = store.get_location(&id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_empty_mock_datastore_list_locations() {
    let store = EmptyMockDataStore;
    let options = create_query_options();
    let result = store.list_locations(&options).await;
    assert!(result.is_ok());
    let paged = result.unwrap();
    assert_eq!(paged.items.len(), 0);
    assert_eq!(paged.total_count, 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_update_location() {
    let store = EmptyMockDataStore;
    let location = create_test_location();
    let result = store.update_location(&location).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_delete_location() {
    let store = EmptyMockDataStore;
    let id = Uuid::new_v4();
    let result = store.delete_location(&id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_create_vendor() {
    let store = EmptyMockDataStore;
    let result = store.create_vendor("cisco").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_list_vendors() {
    let store = EmptyMockDataStore;
    let result = store.list_vendors().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_delete_vendor() {
    let store = EmptyMockDataStore;
    let result = store.delete_vendor("cisco").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_batch_nodes() {
    let store = EmptyMockDataStore;
    let operations = vec![BatchOperation::Insert(create_test_node())];
    let result = store.batch_nodes(&operations).await;
    assert!(result.is_ok());
    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 0);
    assert_eq!(batch_result.error_count, 0);
    assert_eq!(batch_result.errors.len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_batch_links() {
    let store = EmptyMockDataStore;
    let operations = vec![BatchOperation::Insert(create_test_link())];
    let result = store.batch_links(&operations).await;
    assert!(result.is_ok());
    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 0);
    assert_eq!(batch_result.error_count, 0);
    assert_eq!(batch_result.errors.len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_batch_locations() {
    let store = EmptyMockDataStore;
    let operations = vec![BatchOperation::Insert(create_test_location())];
    let result = store.batch_locations(&operations).await;
    assert!(result.is_ok());
    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 0);
    assert_eq!(batch_result.error_count, 0);
    assert_eq!(batch_result.errors.len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_get_entity_counts() {
    let store = EmptyMockDataStore;
    let result = store.get_entity_counts().await;
    assert!(result.is_ok());
    let counts = result.unwrap();
    assert_eq!(counts.len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_get_statistics() {
    let store = EmptyMockDataStore;
    let result = store.get_statistics().await;
    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.len(), 0);
}

#[tokio::test]
async fn test_empty_mock_datastore_store_policy_result() {
    let store = EmptyMockDataStore;
    let node_id = Uuid::new_v4();
    let result = store
        .store_policy_result(&node_id, "test-rule", &create_test_policy_result())
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_mock_datastore_get_nodes_for_policy_evaluation() {
    let store = EmptyMockDataStore;
    let result = store.get_nodes_for_policy_evaluation().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}
