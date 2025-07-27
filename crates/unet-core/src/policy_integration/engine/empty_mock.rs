//! Empty mock `DataStore` implementation for testing

use crate::datastore::{DataStore, DataStoreResult};
use async_trait::async_trait;

pub struct EmptyMockDataStore;

#[async_trait]
impl DataStore for EmptyMockDataStore {
    fn name(&self) -> &'static str {
        "empty_mock"
    }

    async fn health_check(&self) -> DataStoreResult<()> {
        Ok(())
    }

    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn crate::datastore::Transaction>> {
        Err(
            crate::datastore::types::DataStoreError::UnsupportedOperation {
                operation: "transaction".to_string(),
            },
        )
    }

    async fn create_node(
        &self,
        node: &crate::models::Node,
    ) -> DataStoreResult<crate::models::Node> {
        Ok(node.clone())
    }

    async fn get_node(&self, _id: &uuid::Uuid) -> DataStoreResult<Option<crate::models::Node>> {
        Ok(None)
    }

    async fn list_nodes(
        &self,
        _opts: &crate::datastore::QueryOptions,
    ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Node>> {
        Ok(crate::datastore::PagedResult {
            items: vec![],
            total_count: 0,
            page: 1,
            page_size: 10,
            total_pages: 0,
            has_next: false,
            has_previous: false,
        })
    }

    async fn update_node(
        &self,
        node: &crate::models::Node,
    ) -> DataStoreResult<crate::models::Node> {
        Ok(node.clone())
    }

    async fn delete_node(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
        Ok(())
    }

    async fn create_location(
        &self,
        location: &crate::models::Location,
    ) -> DataStoreResult<crate::models::Location> {
        Ok(location.clone())
    }

    async fn get_location(
        &self,
        _id: &uuid::Uuid,
    ) -> DataStoreResult<Option<crate::models::Location>> {
        Ok(None)
    }

    async fn list_locations(
        &self,
        _opts: &crate::datastore::QueryOptions,
    ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Location>> {
        Ok(crate::datastore::PagedResult {
            items: vec![],
            total_count: 0,
            page: 1,
            page_size: 10,
            total_pages: 0,
            has_next: false,
            has_previous: false,
        })
    }

    async fn update_location(
        &self,
        location: &crate::models::Location,
    ) -> DataStoreResult<crate::models::Location> {
        Ok(location.clone())
    }

    async fn delete_location(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
        Ok(())
    }

    async fn create_link(
        &self,
        link: &crate::models::Link,
    ) -> DataStoreResult<crate::models::Link> {
        Ok(link.clone())
    }

    async fn get_link(&self, _id: &uuid::Uuid) -> DataStoreResult<Option<crate::models::Link>> {
        Ok(None)
    }

    async fn list_links(
        &self,
        _opts: &crate::datastore::QueryOptions,
    ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Link>> {
        Ok(crate::datastore::PagedResult {
            items: vec![],
            total_count: 0,
            page: 1,
            page_size: 10,
            total_pages: 0,
            has_next: false,
            has_previous: false,
        })
    }

    async fn update_link(
        &self,
        link: &crate::models::Link,
    ) -> DataStoreResult<crate::models::Link> {
        Ok(link.clone())
    }

    async fn delete_link(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
        Ok(())
    }

    async fn get_node_status(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Option<crate::models::derived::NodeStatus>> {
        Ok(None)
    }

    async fn get_nodes_by_location(
        &self,
        _location_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::models::Node>> {
        Ok(vec![])
    }

    async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<crate::models::Node>> {
        Ok(vec![])
    }

    async fn get_links_for_node(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::models::Link>> {
        Ok(vec![])
    }

    async fn get_links_between_nodes(
        &self,
        _first_node_id: &uuid::Uuid,
        _second_node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::models::Link>> {
        Ok(vec![])
    }

    async fn create_vendor(&self, _name: &str) -> DataStoreResult<()> {
        Ok(())
    }

    async fn list_vendors(&self) -> DataStoreResult<Vec<String>> {
        Ok(vec![])
    }

    async fn delete_vendor(&self, _name: &str) -> DataStoreResult<()> {
        Ok(())
    }

    async fn batch_nodes(
        &self,
        _operations: &[crate::datastore::BatchOperation<crate::models::Node>],
    ) -> DataStoreResult<crate::datastore::BatchResult> {
        Ok(crate::datastore::BatchResult {
            success_count: 0,
            error_count: 0,
            errors: vec![],
        })
    }

    async fn batch_links(
        &self,
        _operations: &[crate::datastore::BatchOperation<crate::models::Link>],
    ) -> DataStoreResult<crate::datastore::BatchResult> {
        Ok(crate::datastore::BatchResult {
            success_count: 0,
            error_count: 0,
            errors: vec![],
        })
    }

    async fn batch_locations(
        &self,
        _operations: &[crate::datastore::BatchOperation<crate::models::Location>],
    ) -> DataStoreResult<crate::datastore::BatchResult> {
        Ok(crate::datastore::BatchResult {
            success_count: 0,
            error_count: 0,
            errors: vec![],
        })
    }

    async fn get_entity_counts(&self) -> DataStoreResult<std::collections::HashMap<String, usize>> {
        Ok(std::collections::HashMap::new())
    }

    async fn get_statistics(
        &self,
    ) -> DataStoreResult<std::collections::HashMap<String, serde_json::Value>> {
        Ok(std::collections::HashMap::new())
    }

    async fn store_policy_result(
        &self,
        _node_id: &uuid::Uuid,
        _rule_id: &str,
        _result: &crate::policy::PolicyExecutionResult,
    ) -> DataStoreResult<()> {
        Ok(())
    }

    async fn get_policy_results(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::policy::PolicyExecutionResult>> {
        Ok(vec![])
    }

    async fn get_latest_policy_results(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::policy::PolicyExecutionResult>> {
        Ok(vec![])
    }

    async fn get_nodes_for_policy_evaluation(&self) -> DataStoreResult<Vec<crate::models::Node>> {
        Ok(vec![])
    }
}
