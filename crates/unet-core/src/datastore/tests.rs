//! Tests for the datastore module

#[cfg(test)]
mod datastore_tests {
    use crate::datastore::helpers::{
        filter_contains, filter_equals_string, filter_equals_uuid, sort_asc, sort_desc,
    };
    use crate::datastore::types::{
        BatchResult, DataStoreError, FilterOperation, FilterValue, PagedResult, Pagination,
        QueryOptions, SortDirection,
    };
    use uuid::Uuid;

    #[test]
    fn test_pagination_new() {
        // Valid pagination
        let pagination = Pagination::new(10, 0).unwrap();
        assert_eq!(pagination.limit, 10);
        assert_eq!(pagination.offset, 0);

        // Invalid limit (0)
        assert!(Pagination::new(0, 0).is_err());

        // Invalid limit (too large)
        assert!(Pagination::new(1001, 0).is_err());

        // Valid edge cases
        assert!(Pagination::new(1, 0).is_ok());
        assert!(Pagination::new(1000, 0).is_ok());
    }

    #[test]
    fn test_pagination_page() {
        // Valid page pagination
        let pagination = Pagination::page(1, 10).unwrap();
        assert_eq!(pagination.limit, 10);
        assert_eq!(pagination.offset, 0);

        let pagination = Pagination::page(3, 20).unwrap();
        assert_eq!(pagination.limit, 20);
        assert_eq!(pagination.offset, 40);

        // Invalid page (0)
        assert!(Pagination::page(0, 10).is_err());
    }

    #[test]
    fn test_paged_result_new() {
        let items = vec![1, 2, 3];

        // Without pagination
        let result = PagedResult::new(items.clone(), 3, None);
        assert_eq!(result.items, items);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.page_size, 3);
        assert_eq!(result.page, 1);
        assert_eq!(result.total_pages, 1);
        assert!(!result.has_next);
        assert!(!result.has_previous);

        // With pagination - first page
        let pagination = Pagination::new(2, 0).unwrap();
        let result = PagedResult::new(vec![1, 2], 5, Some(&pagination));
        assert_eq!(result.items, vec![1, 2]);
        assert_eq!(result.total_count, 5);
        assert_eq!(result.page_size, 2);
        assert_eq!(result.page, 1);
        assert_eq!(result.total_pages, 3);
        assert!(result.has_next);
        assert!(!result.has_previous);

        // With pagination - middle page
        let pagination = Pagination::new(2, 2).unwrap();
        let result = PagedResult::new(vec![3, 4], 5, Some(&pagination));
        assert_eq!(result.page, 2);
        assert!(result.has_next);
        assert!(result.has_previous);

        // With pagination - last page
        let pagination = Pagination::new(2, 4).unwrap();
        let result = PagedResult::new(vec![5], 5, Some(&pagination));
        assert_eq!(result.page, 3);
        assert!(!result.has_next);
        assert!(result.has_previous);
    }

    #[test]
    fn test_filter_helpers() {
        let filter = filter_equals_string("name", "test");
        assert_eq!(filter.field, "name");
        assert!(matches!(filter.operation, FilterOperation::Equals));
        assert!(matches!(filter.value, FilterValue::String(ref s) if s == "test"));

        let uuid = Uuid::new_v4();
        let filter = filter_equals_uuid("id", uuid);
        assert_eq!(filter.field, "id");
        assert!(matches!(filter.operation, FilterOperation::Equals));
        assert!(matches!(filter.value, FilterValue::Uuid(u) if u == uuid));

        let filter = filter_contains("description", "partial");
        assert_eq!(filter.field, "description");
        assert!(matches!(filter.operation, FilterOperation::Contains));
        assert!(matches!(filter.value, FilterValue::String(ref s) if s == "partial"));
    }

    #[test]
    fn test_sort_helpers() {
        let sort = sort_asc("name");
        assert_eq!(sort.field, "name");
        assert!(matches!(sort.direction, SortDirection::Ascending));

        let sort = sort_desc("created_at");
        assert_eq!(sort.field, "created_at");
        assert!(matches!(sort.direction, SortDirection::Descending));
    }

    #[test]
    fn test_query_options_default() {
        let options = QueryOptions::default();
        assert!(options.filters.is_empty());
        assert!(options.sort.is_empty());
        assert!(options.pagination.is_none());
    }

    #[test]
    fn test_batch_result() {
        let result = BatchResult {
            success_count: 5,
            error_count: 2,
            errors: vec![
                (
                    1,
                    DataStoreError::ValidationError {
                        message: "Test error".to_string(),
                    },
                ),
                (
                    3,
                    DataStoreError::NotFound {
                        entity_type: "Node".to_string(),
                        id: "test-id".to_string(),
                    },
                ),
            ],
        };

        assert_eq!(result.success_count, 5);
        assert_eq!(result.error_count, 2);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_datastore_error_display() {
        let error = DataStoreError::NotFound {
            entity_type: "Node".to_string(),
            id: "123".to_string(),
        };
        assert!(error.to_string().contains("Entity not found"));
        assert!(error.to_string().contains("Node"));
        assert!(error.to_string().contains("123"));

        let error = DataStoreError::ValidationError {
            message: "Invalid input".to_string(),
        };
        assert!(error.to_string().contains("Validation error"));
        assert!(error.to_string().contains("Invalid input"));

        let error = DataStoreError::Timeout { seconds: 30 };
        assert!(error.to_string().contains("timeout"));
        assert!(error.to_string().contains("30"));
    }
}

/// Tests for `DataStore` trait default implementations
#[cfg(test)]
mod datastore_defaults_tests {
    use crate::datastore::{DataStore, DataStoreError, DataStoreResult, QueryOptions};
    use crate::models::{DeviceRole, Lifecycle, Node, NodeBuilder, Vendor};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Mock `DataStore` implementation for testing default trait methods
    struct MockDataStore {
        nodes: HashMap<Uuid, Node>,
    }

    impl MockDataStore {
        fn new() -> Self {
            Self {
                nodes: HashMap::new(),
            }
        }

        fn with_node(mut self, node: Node) -> Self {
            self.nodes.insert(node.id, node);
            self
        }
    }

    #[async_trait]
    impl DataStore for MockDataStore {
        fn name(&self) -> &'static str {
            "MockDataStore"
        }

        async fn health_check(&self) -> DataStoreResult<()> {
            Ok(())
        }

        async fn begin_transaction(
            &self,
        ) -> DataStoreResult<Box<dyn crate::datastore::Transaction>> {
            Err(DataStoreError::InternalError {
                message: "begin_transaction not implemented in mock".to_string(),
            })
        }

        async fn create_node(&self, _node: &Node) -> DataStoreResult<Node> {
            Err(DataStoreError::InternalError {
                message: "create_node not implemented in mock".to_string(),
            })
        }

        async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>> {
            Ok(self.nodes.get(id).cloned())
        }

        async fn list_nodes(
            &self,
            _options: &QueryOptions,
        ) -> DataStoreResult<crate::datastore::PagedResult<Node>> {
            let nodes: Vec<Node> = self.nodes.values().cloned().collect();
            Ok(crate::datastore::PagedResult::new(
                nodes,
                self.nodes.len(),
                None,
            ))
        }

        async fn update_node(&self, _node: &Node) -> DataStoreResult<Node> {
            Err(DataStoreError::InternalError {
                message: "update_node not implemented in mock".to_string(),
            })
        }

        async fn delete_node(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "delete_node not implemented in mock".to_string(),
            })
        }

        async fn get_nodes_by_location(&self, _location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
            Ok(Vec::new())
        }

        async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<Node>> {
            Ok(Vec::new())
        }

        async fn create_link(
            &self,
            _link: &crate::models::Link,
        ) -> DataStoreResult<crate::models::Link> {
            Err(DataStoreError::InternalError {
                message: "create_link not implemented in mock".to_string(),
            })
        }

        async fn get_link(&self, _id: &Uuid) -> DataStoreResult<Option<crate::models::Link>> {
            Ok(None)
        }

        async fn list_links(
            &self,
            _options: &QueryOptions,
        ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Link>> {
            Ok(crate::datastore::PagedResult::new(Vec::new(), 0, None))
        }

        async fn update_link(
            &self,
            _link: &crate::models::Link,
        ) -> DataStoreResult<crate::models::Link> {
            Err(DataStoreError::InternalError {
                message: "update_link not implemented in mock".to_string(),
            })
        }

        async fn delete_link(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "delete_link not implemented in mock".to_string(),
            })
        }

        async fn get_links_for_node(
            &self,
            _node_id: &Uuid,
        ) -> DataStoreResult<Vec<crate::models::Link>> {
            Ok(Vec::new())
        }

        async fn get_links_between_nodes(
            &self,
            _first_node_id: &Uuid,
            _second_node_id: &Uuid,
        ) -> DataStoreResult<Vec<crate::models::Link>> {
            Ok(Vec::new())
        }

        async fn create_location(
            &self,
            _location: &crate::models::Location,
        ) -> DataStoreResult<crate::models::Location> {
            Err(DataStoreError::InternalError {
                message: "create_location not implemented in mock".to_string(),
            })
        }

        async fn get_location(
            &self,
            _id: &Uuid,
        ) -> DataStoreResult<Option<crate::models::Location>> {
            Ok(None)
        }

        async fn list_locations(
            &self,
            _options: &QueryOptions,
        ) -> DataStoreResult<crate::datastore::PagedResult<crate::models::Location>> {
            Ok(crate::datastore::PagedResult::new(Vec::new(), 0, None))
        }

        async fn update_location(
            &self,
            _location: &crate::models::Location,
        ) -> DataStoreResult<crate::models::Location> {
            Err(DataStoreError::InternalError {
                message: "update_location not implemented in mock".to_string(),
            })
        }

        async fn delete_location(&self, _id: &Uuid) -> DataStoreResult<()> {
            Err(DataStoreError::InternalError {
                message: "delete_location not implemented in mock".to_string(),
            })
        }

        async fn batch_nodes(
            &self,
            _operations: &[crate::datastore::BatchOperation<Node>],
        ) -> DataStoreResult<crate::datastore::BatchResult> {
            Ok(crate::datastore::BatchResult {
                success_count: 0,
                error_count: 0,
                errors: Vec::new(),
            })
        }

        async fn batch_links(
            &self,
            _operations: &[crate::datastore::BatchOperation<crate::models::Link>],
        ) -> DataStoreResult<crate::datastore::BatchResult> {
            Ok(crate::datastore::BatchResult {
                success_count: 0,
                error_count: 0,
                errors: Vec::new(),
            })
        }

        async fn batch_locations(
            &self,
            _operations: &[crate::datastore::BatchOperation<crate::models::Location>],
        ) -> DataStoreResult<crate::datastore::BatchResult> {
            Ok(crate::datastore::BatchResult {
                success_count: 0,
                error_count: 0,
                errors: Vec::new(),
            })
        }

        async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
            let mut counts = HashMap::new();
            counts.insert("nodes".to_string(), self.nodes.len());
            counts.insert("links".to_string(), 0);
            counts.insert("locations".to_string(), 0);
            Ok(counts)
        }

        async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
            Ok(HashMap::new())
        }
    }

    fn create_test_node() -> Node {
        NodeBuilder::new()
            .name("test-node".to_string())
            .domain("example.com".to_string())
            .vendor(Vendor::Cisco)
            .model("Test Model".to_string())
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_get_node_required_exists() {
        let node = create_test_node();
        let store = MockDataStore::new().with_node(node.clone());

        let result = store.get_node_required(&node.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, node.id);
    }

    #[tokio::test]
    async fn test_get_node_required_not_exists() {
        let store = MockDataStore::new();
        let non_existent_id = Uuid::new_v4();

        let result = store.get_node_required(&non_existent_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DataStoreError::NotFound { entity_type, id } => {
                assert_eq!(entity_type, "Node");
                assert_eq!(id, non_existent_id.to_string());
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_link_required_not_exists() {
        let store = MockDataStore::new();
        let non_existent_id = Uuid::new_v4();

        let result = store.get_link_required(&non_existent_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DataStoreError::NotFound { entity_type, id } => {
                assert_eq!(entity_type, "Link");
                assert_eq!(id, non_existent_id.to_string());
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_location_required_not_exists() {
        let store = MockDataStore::new();
        let non_existent_id = Uuid::new_v4();

        let result = store.get_location_required(&non_existent_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DataStoreError::NotFound { entity_type, id } => {
                assert_eq!(entity_type, "Location");
                assert_eq!(id, non_existent_id.to_string());
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_node_status_default() {
        let store = MockDataStore::new();
        let node_id = Uuid::new_v4();

        let result = store.get_node_status(&node_id).await;
        assert!(result.is_ok());

        let status = result.unwrap();
        assert!(status.is_some());
        assert_eq!(status.unwrap().node_id, node_id);
    }

    #[tokio::test]
    async fn test_get_node_interfaces_default() {
        let store = MockDataStore::new();
        let node_id = Uuid::new_v4();

        let result = store.get_node_interfaces(&node_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_node_metrics_default() {
        let store = MockDataStore::new();
        let node_id = Uuid::new_v4();

        let result = store.get_node_metrics(&node_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // Note: test_store_policy_result_default skipped due to complex PolicyExecutionResult structure

    #[tokio::test]
    async fn test_get_policy_results_default() {
        let store = MockDataStore::new();
        let node_id = Uuid::new_v4();

        let result = store.get_policy_results(&node_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_latest_policy_results_default() {
        let store = MockDataStore::new();
        let node_id = Uuid::new_v4();

        let result = store.get_latest_policy_results(&node_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_rule_results_default() {
        let store = MockDataStore::new();
        let rule_id = "test-rule";

        let result = store.get_rule_results(rule_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_nodes_for_policy_evaluation() {
        let node = create_test_node();
        let store = MockDataStore::new().with_node(node.clone());

        let result = store.get_nodes_for_policy_evaluation().await;
        assert!(result.is_ok());

        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, node.id);
    }
}
