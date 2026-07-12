//! Failing mock `DataStore` implementation for testing error conditions

use crate::datastore::{DataStore, DataStoreResult};
use async_trait::async_trait;

pub struct FailingMockDataStore;

#[async_trait]
impl DataStore for FailingMockDataStore {
    fn name(&self) -> &'static str {
        "failing_mock"
    }

    async fn health_check(&self) -> DataStoreResult<()> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock health check failed".to_string(),
        })
    }

    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn crate::datastore::Transaction>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock transaction failed".to_string(),
        })
    }

    async fn create_node(
        &self,
        _node: &crate::models::Node,
    ) -> DataStoreResult<crate::models::Node> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock create failed".to_string(),
        })
    }

    async fn get_node(&self, _id: &uuid::Uuid) -> DataStoreResult<Option<crate::models::Node>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn list_nodes(
        &self,
        _opts: &crate::datastore::QueryOptions,
    ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Node>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock list failed".to_string(),
        })
    }

    async fn update_node(
        &self,
        _node: &crate::models::Node,
    ) -> DataStoreResult<crate::models::Node> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock update failed".to_string(),
        })
    }

    async fn delete_node(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock delete failed".to_string(),
        })
    }

    async fn create_location(
        &self,
        _location: &crate::models::Location,
    ) -> DataStoreResult<crate::models::Location> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock create failed".to_string(),
        })
    }

    async fn get_location(
        &self,
        _id: &uuid::Uuid,
    ) -> DataStoreResult<Option<crate::models::Location>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn list_locations(
        &self,
        _opts: &crate::datastore::QueryOptions,
    ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Location>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock list failed".to_string(),
        })
    }

    async fn update_location(
        &self,
        _location: &crate::models::Location,
    ) -> DataStoreResult<crate::models::Location> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock update failed".to_string(),
        })
    }

    async fn delete_location(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock delete failed".to_string(),
        })
    }

    async fn create_link(
        &self,
        _link: &crate::models::Link,
    ) -> DataStoreResult<crate::models::Link> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock create failed".to_string(),
        })
    }

    async fn get_link(&self, _id: &uuid::Uuid) -> DataStoreResult<Option<crate::models::Link>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn list_links(
        &self,
        _opts: &crate::datastore::QueryOptions,
    ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Link>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock list failed".to_string(),
        })
    }

    async fn update_link(
        &self,
        _link: &crate::models::Link,
    ) -> DataStoreResult<crate::models::Link> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock update failed".to_string(),
        })
    }

    async fn delete_link(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock delete failed".to_string(),
        })
    }

    async fn get_node_status(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Option<crate::models::derived::NodeStatus>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn get_nodes_by_location(
        &self,
        _location_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::models::Node>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<crate::models::Node>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn get_links_for_node(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::models::Link>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn get_links_between_nodes(
        &self,
        _first_node_id: &uuid::Uuid,
        _second_node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::models::Link>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn create_vendor(&self, _name: &str) -> DataStoreResult<()> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock create failed".to_string(),
        })
    }

    async fn list_vendors(&self) -> DataStoreResult<Vec<String>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock list failed".to_string(),
        })
    }

    async fn delete_vendor(&self, _name: &str) -> DataStoreResult<()> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock delete failed".to_string(),
        })
    }

    async fn batch_nodes(
        &self,
        _operations: &[crate::datastore::BatchOperation<crate::models::Node>],
    ) -> DataStoreResult<crate::datastore::BatchResult> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock batch failed".to_string(),
        })
    }

    async fn batch_links(
        &self,
        _operations: &[crate::datastore::BatchOperation<crate::models::Link>],
    ) -> DataStoreResult<crate::datastore::BatchResult> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock batch failed".to_string(),
        })
    }

    async fn batch_locations(
        &self,
        _operations: &[crate::datastore::BatchOperation<crate::models::Location>],
    ) -> DataStoreResult<crate::datastore::BatchResult> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock batch failed".to_string(),
        })
    }

    async fn get_entity_counts(&self) -> DataStoreResult<std::collections::HashMap<String, usize>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn get_statistics(
        &self,
    ) -> DataStoreResult<std::collections::HashMap<String, serde_json::Value>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn store_policy_result(
        &self,
        _node_id: &uuid::Uuid,
        _rule_id: &str,
        _result: &crate::policy::PolicyExecutionResult,
    ) -> DataStoreResult<()> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock policy result storage error".to_string(),
        })
    }

    async fn get_policy_results(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::policy::PolicyExecutionResult>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn get_latest_policy_results(
        &self,
        _node_id: &uuid::Uuid,
    ) -> DataStoreResult<Vec<crate::policy::PolicyExecutionResult>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock get failed".to_string(),
        })
    }

    async fn get_nodes_for_policy_evaluation(&self) -> DataStoreResult<Vec<crate::models::Node>> {
        Err(crate::datastore::types::DataStoreError::InternalError {
            message: "Mock policy evaluation error".to_string(),
        })
    }
}
