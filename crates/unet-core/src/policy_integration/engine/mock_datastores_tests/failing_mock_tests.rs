//! Tests for `FailingMockDataStore` implementation

use super::super::mock_datastores::mocks::FailingMockDataStore;
use super::shared::*;
use crate::datastore::{BatchOperation, DataStore};
use uuid::Uuid;

#[tokio::test]
async fn test_failing_mock_datastore_name() {
    let store = FailingMockDataStore;
    assert_eq!(store.name(), "FailingMockDataStore");
}

#[tokio::test]
async fn test_failing_mock_datastore_health_check() {
    let store = FailingMockDataStore;
    let result = store.health_check().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_begin_transaction() {
    let store = FailingMockDataStore;
    let result = store.begin_transaction().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_create_node() {
    let store = FailingMockDataStore;
    let node = create_test_node();
    let result = store.create_node(&node).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_node() {
    let store = FailingMockDataStore;
    let id = Uuid::new_v4();
    let result = store.get_node(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_list_nodes() {
    let store = FailingMockDataStore;
    let options = create_query_options();
    let result = store.list_nodes(&options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_update_node() {
    let store = FailingMockDataStore;
    let node = create_test_node();
    let result = store.update_node(&node).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_delete_node() {
    let store = FailingMockDataStore;
    let id = Uuid::new_v4();
    let result = store.delete_node(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_nodes_by_location() {
    let store = FailingMockDataStore;
    let location_id = Uuid::new_v4();
    let result = store.get_nodes_by_location(&location_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_search_nodes_by_name() {
    let store = FailingMockDataStore;
    let result = store.search_nodes_by_name("test").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_create_link() {
    let store = FailingMockDataStore;
    let link = create_test_link();
    let result = store.create_link(&link).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_link() {
    let store = FailingMockDataStore;
    let id = Uuid::new_v4();
    let result = store.get_link(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_list_links() {
    let store = FailingMockDataStore;
    let options = create_query_options();
    let result = store.list_links(&options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_update_link() {
    let store = FailingMockDataStore;
    let link = create_test_link();
    let result = store.update_link(&link).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_delete_link() {
    let store = FailingMockDataStore;
    let id = Uuid::new_v4();
    let result = store.delete_link(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_links_for_node() {
    let store = FailingMockDataStore;
    let node_id = Uuid::new_v4();
    let result = store.get_links_for_node(&node_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_links_between_nodes() {
    let store = FailingMockDataStore;
    let first_node = Uuid::new_v4();
    let second_node = Uuid::new_v4();
    let result = store
        .get_links_between_nodes(&first_node, &second_node)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_create_location() {
    let store = FailingMockDataStore;
    let location = create_test_location();
    let result = store.create_location(&location).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_location() {
    let store = FailingMockDataStore;
    let id = Uuid::new_v4();
    let result = store.get_location(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_list_locations() {
    let store = FailingMockDataStore;
    let options = create_query_options();
    let result = store.list_locations(&options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_update_location() {
    let store = FailingMockDataStore;
    let location = create_test_location();
    let result = store.update_location(&location).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_delete_location() {
    let store = FailingMockDataStore;
    let id = Uuid::new_v4();
    let result = store.delete_location(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_create_vendor() {
    let store = FailingMockDataStore;
    let result = store.create_vendor("cisco").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_list_vendors() {
    let store = FailingMockDataStore;
    let result = store.list_vendors().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_delete_vendor() {
    let store = FailingMockDataStore;
    let result = store.delete_vendor("cisco").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_batch_nodes() {
    let store = FailingMockDataStore;
    let operations = vec![BatchOperation::Insert(create_test_node())];
    let result = store.batch_nodes(&operations).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_batch_links() {
    let store = FailingMockDataStore;
    let operations = vec![BatchOperation::Insert(create_test_link())];
    let result = store.batch_links(&operations).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_batch_locations() {
    let store = FailingMockDataStore;
    let operations = vec![BatchOperation::Insert(create_test_location())];
    let result = store.batch_locations(&operations).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_entity_counts() {
    let store = FailingMockDataStore;
    let result = store.get_entity_counts().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_statistics() {
    let store = FailingMockDataStore;
    let result = store.get_statistics().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_store_policy_result() {
    let store = FailingMockDataStore;
    let node_id = Uuid::new_v4();
    let result = store
        .store_policy_result(&node_id, "test-rule", &create_test_policy_result())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_failing_mock_datastore_get_nodes_for_policy_evaluation() {
    let store = FailingMockDataStore;
    let result = store.get_nodes_for_policy_evaluation().await;
    assert!(result.is_err());
}
