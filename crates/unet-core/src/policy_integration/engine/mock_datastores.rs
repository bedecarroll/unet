//! Mock `DataStore` implementations for testing

#[cfg(test)]
pub mod mocks {
    use crate::datastore::{
        BatchOperation, BatchResult, DataStore, DataStoreError, DataStoreResult, PagedResult,
        QueryOptions,
    };
    use crate::models::{Link, Location, Node};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Mock `DataStore` that always returns errors for testing error paths
    pub struct FailingMockDataStore;

    #[async_trait]
    impl DataStore for FailingMockDataStore {
        fn name(&self) -> &'static str {
            "FailingMockDataStore"
        }

        async fn health_check(&self) -> DataStoreResult<()> {
            Err(DataStoreError::ConnectionError {
                message: "Mock error".to_string(),
            })
        }

        async fn begin_transaction(
            &self,
        ) -> DataStoreResult<Box<dyn crate::datastore::Transaction>> {
            Err(DataStoreError::TransactionError {
                message: "Mock error".to_string(),
            })
        }

        async fn create_node(&self, _node: &Node) -> DataStoreResult<Node> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_node(&self, _id: &Uuid) -> DataStoreResult<Option<Node>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn list_nodes(&self, _options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn update_node(&self, _node: &Node) -> DataStoreResult<Node> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn delete_node(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_nodes_by_location(&self, _location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<Node>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn create_link(&self, _link: &Link) -> DataStoreResult<Link> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_link(&self, _id: &Uuid) -> DataStoreResult<Option<Link>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn list_links(&self, _options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn update_link(&self, _link: &Link) -> DataStoreResult<Link> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn delete_link(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_links_for_node(&self, _node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_links_between_nodes(
            &self,
            _first_node_id: &Uuid,
            _second_node_id: &Uuid,
        ) -> DataStoreResult<Vec<Link>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn create_location(&self, _location: &Location) -> DataStoreResult<Location> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_location(&self, _id: &Uuid) -> DataStoreResult<Option<Location>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn list_locations(
            &self,
            _options: &QueryOptions,
        ) -> DataStoreResult<PagedResult<Location>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn update_location(&self, _location: &Location) -> DataStoreResult<Location> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn delete_location(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn create_vendor(&self, _name: &str) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn list_vendors(&self) -> DataStoreResult<Vec<String>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn delete_vendor(&self, _name: &str) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn batch_nodes(
            &self,
            _operations: &[BatchOperation<Node>],
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn batch_links(
            &self,
            _operations: &[BatchOperation<Link>],
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn batch_locations(
            &self,
            _operations: &[BatchOperation<Location>],
        ) -> DataStoreResult<BatchResult> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
            Err(DataStoreError::InternalError {
                message: "Mock error".to_string(),
            })
        }

        async fn store_policy_result(
            &self,
            _node_id: &Uuid,
            _rule_id: &str,
            _result: &crate::policy::PolicyExecutionResult,
        ) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "Mock policy result storage error".to_string(),
            })
        }

        async fn get_nodes_for_policy_evaluation(&self) -> DataStoreResult<Vec<Node>> {
            Err(DataStoreError::InternalError {
                message: "Mock policy evaluation error".to_string(),
            })
        }
    }

    /// Mock `DataStore` that returns empty results for successful operations
    pub struct EmptyMockDataStore;

    #[async_trait]
    impl DataStore for EmptyMockDataStore {
        fn name(&self) -> &'static str {
            "EmptyMockDataStore"
        }

        async fn health_check(&self) -> DataStoreResult<()> {
            Ok(())
        }

        async fn begin_transaction(
            &self,
        ) -> DataStoreResult<Box<dyn crate::datastore::Transaction>> {
            // This is a simplified mock - would need a proper Transaction impl for complete testing
            Err(DataStoreError::UnsupportedOperation {
                operation: "Transaction mock".to_string(),
            })
        }

        async fn create_node(&self, node: &Node) -> DataStoreResult<Node> {
            Ok(node.clone())
        }

        async fn get_node(&self, _id: &Uuid) -> DataStoreResult<Option<Node>> {
            Ok(None)
        }

        async fn list_nodes(&self, _options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> {
            Ok(PagedResult::new(
                Vec::new(),
                0,
                _options.pagination.as_ref(),
            ))
        }

        async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
            Ok(node.clone())
        }

        async fn delete_node(&self, _id: &Uuid) -> DataStoreResult<()> {
            Ok(())
        }

        async fn get_nodes_by_location(&self, _location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
            Ok(Vec::new())
        }

        async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<Node>> {
            Ok(Vec::new())
        }

        async fn create_link(&self, link: &Link) -> DataStoreResult<Link> {
            Ok(link.clone())
        }

        async fn get_link(&self, _id: &Uuid) -> DataStoreResult<Option<Link>> {
            Ok(None)
        }

        async fn list_links(&self, _options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> {
            Ok(PagedResult::new(
                Vec::new(),
                0,
                _options.pagination.as_ref(),
            ))
        }

        async fn update_link(&self, link: &Link) -> DataStoreResult<Link> {
            Ok(link.clone())
        }

        async fn delete_link(&self, _id: &Uuid) -> DataStoreResult<()> {
            Ok(())
        }

        async fn get_links_for_node(&self, _node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
            Ok(Vec::new())
        }

        async fn get_links_between_nodes(
            &self,
            _first_node_id: &Uuid,
            _second_node_id: &Uuid,
        ) -> DataStoreResult<Vec<Link>> {
            Ok(Vec::new())
        }

        async fn create_location(&self, location: &Location) -> DataStoreResult<Location> {
            Ok(location.clone())
        }

        async fn get_location(&self, _id: &Uuid) -> DataStoreResult<Option<Location>> {
            Ok(None)
        }

        async fn list_locations(
            &self,
            _options: &QueryOptions,
        ) -> DataStoreResult<PagedResult<Location>> {
            Ok(PagedResult::new(
                Vec::new(),
                0,
                _options.pagination.as_ref(),
            ))
        }

        async fn update_location(&self, location: &Location) -> DataStoreResult<Location> {
            Ok(location.clone())
        }

        async fn delete_location(&self, _id: &Uuid) -> DataStoreResult<()> {
            Ok(())
        }

        async fn create_vendor(&self, _name: &str) -> DataStoreResult<()> {
            Ok(())
        }

        async fn list_vendors(&self) -> DataStoreResult<Vec<String>> {
            Ok(Vec::new())
        }

        async fn delete_vendor(&self, _name: &str) -> DataStoreResult<()> {
            Ok(())
        }

        async fn batch_nodes(
            &self,
            _operations: &[BatchOperation<Node>],
        ) -> DataStoreResult<BatchResult> {
            Ok(BatchResult {
                success_count: 0,
                error_count: 0,
                errors: Vec::new(),
            })
        }

        async fn batch_links(
            &self,
            _operations: &[BatchOperation<Link>],
        ) -> DataStoreResult<BatchResult> {
            Ok(BatchResult {
                success_count: 0,
                error_count: 0,
                errors: Vec::new(),
            })
        }

        async fn batch_locations(
            &self,
            _operations: &[BatchOperation<Location>],
        ) -> DataStoreResult<BatchResult> {
            Ok(BatchResult {
                success_count: 0,
                error_count: 0,
                errors: Vec::new(),
            })
        }

        async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
            Ok(HashMap::new())
        }

        async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
            Ok(HashMap::new())
        }

        async fn store_policy_result(
            &self,
            _node_id: &Uuid,
            _rule_id: &str,
            _result: &crate::policy::PolicyExecutionResult,
        ) -> DataStoreResult<()> {
            Ok(())
        }

        async fn get_nodes_for_policy_evaluation(&self) -> DataStoreResult<Vec<Node>> {
            Ok(Vec::new())
        }
    }
}
