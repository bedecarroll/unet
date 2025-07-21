//! Tests for transaction helper functions

#[cfg(test)]
mod transaction_helper_tests {
    use super::super::super::DataStore;
    use super::super::super::types::{DataStoreError, DataStoreResult, Transaction};
    use super::super::core::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // Mock transaction for testing
    struct MockTransaction {
        committed: Arc<Mutex<bool>>,
        rolled_back: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl Transaction for MockTransaction {
        async fn commit(self: Box<Self>) -> DataStoreResult<()> {
            *self.committed.lock().unwrap() = true;
            Ok(())
        }

        async fn rollback(self: Box<Self>) -> DataStoreResult<()> {
            *self.rolled_back.lock().unwrap() = true;
            Ok(())
        }
    }

    // Mock datastore for testing - minimal implementation
    struct MockDataStore {
        should_fail: bool,
    }

    #[async_trait]
    impl DataStore for MockDataStore {
        fn name(&self) -> &'static str {
            "MockDataStore"
        }

        async fn health_check(&self) -> DataStoreResult<()> {
            if self.should_fail {
                Err(DataStoreError::ConnectionError {
                    message: "Health check failed".to_string(),
                })
            } else {
                Ok(())
            }
        }

        async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> {
            if self.should_fail {
                return Err(DataStoreError::ConnectionError {
                    message: "Failed to begin transaction".to_string(),
                });
            }
            Ok(Box::new(MockTransaction {
                committed: Arc::new(Mutex::new(false)),
                rolled_back: Arc::new(Mutex::new(false)),
            }))
        }

        // Node operations
        async fn create_node(
            &self,
            _node: &crate::models::node::Node,
        ) -> DataStoreResult<crate::models::node::Node> {
            unimplemented!()
        }

        async fn get_node(
            &self,
            _id: &uuid::Uuid,
        ) -> DataStoreResult<Option<crate::models::node::Node>> {
            unimplemented!()
        }

        async fn list_nodes(
            &self,
            _options: &crate::datastore::types::QueryOptions,
        ) -> DataStoreResult<crate::datastore::types::PagedResult<crate::models::node::Node>>
        {
            unimplemented!()
        }

        async fn update_node(
            &self,
            _node: &crate::models::node::Node,
        ) -> DataStoreResult<crate::models::node::Node> {
            unimplemented!()
        }

        async fn delete_node(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
            unimplemented!()
        }

        async fn get_nodes_by_location(
            &self,
            _location_id: &uuid::Uuid,
        ) -> DataStoreResult<Vec<crate::models::node::Node>> {
            unimplemented!()
        }

        async fn search_nodes_by_name(
            &self,
            _name: &str,
        ) -> DataStoreResult<Vec<crate::models::node::Node>> {
            unimplemented!()
        }

        // Link operations
        async fn create_link(
            &self,
            _link: &crate::models::link::Link,
        ) -> DataStoreResult<crate::models::link::Link> {
            unimplemented!()
        }

        async fn get_link(
            &self,
            _id: &uuid::Uuid,
        ) -> DataStoreResult<Option<crate::models::link::Link>> {
            unimplemented!()
        }

        async fn list_links(
            &self,
            _options: &crate::datastore::types::QueryOptions,
        ) -> DataStoreResult<crate::datastore::types::PagedResult<crate::models::link::Link>>
        {
            unimplemented!()
        }

        async fn update_link(
            &self,
            _link: &crate::models::link::Link,
        ) -> DataStoreResult<crate::models::link::Link> {
            unimplemented!()
        }

        async fn delete_link(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
            unimplemented!()
        }

        async fn get_links_for_node(
            &self,
            _node_id: &uuid::Uuid,
        ) -> DataStoreResult<Vec<crate::models::link::Link>> {
            unimplemented!()
        }

        async fn get_links_between_nodes(
            &self,
            _first_node_id: &uuid::Uuid,
            _second_node_id: &uuid::Uuid,
        ) -> DataStoreResult<Vec<crate::models::link::Link>> {
            unimplemented!()
        }

        // Location operations
        async fn create_location(
            &self,
            _location: &crate::models::location::Location,
        ) -> DataStoreResult<crate::models::location::Location> {
            unimplemented!()
        }

        async fn get_location(
            &self,
            _id: &uuid::Uuid,
        ) -> DataStoreResult<Option<crate::models::location::Location>> {
            unimplemented!()
        }

        async fn list_locations(
            &self,
            _options: &crate::datastore::types::QueryOptions,
        ) -> DataStoreResult<crate::datastore::types::PagedResult<crate::models::location::Location>>
        {
            unimplemented!()
        }

        async fn update_location(
            &self,
            _location: &crate::models::location::Location,
        ) -> DataStoreResult<crate::models::location::Location> {
            unimplemented!()
        }

        async fn delete_location(&self, _id: &uuid::Uuid) -> DataStoreResult<()> {
            unimplemented!()
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

        // Batch operations
        async fn batch_nodes(
            &self,
            _operations: &[crate::datastore::types::BatchOperation<crate::models::node::Node>],
        ) -> DataStoreResult<crate::datastore::types::BatchResult> {
            unimplemented!()
        }

        async fn batch_links(
            &self,
            _operations: &[crate::datastore::types::BatchOperation<crate::models::link::Link>],
        ) -> DataStoreResult<crate::datastore::types::BatchResult> {
            unimplemented!()
        }

        async fn batch_locations(
            &self,
            _operations: &[crate::datastore::types::BatchOperation<
                crate::models::location::Location,
            >],
        ) -> DataStoreResult<crate::datastore::types::BatchResult> {
            unimplemented!()
        }

        // Statistics and metadata
        async fn get_entity_counts(
            &self,
        ) -> DataStoreResult<std::collections::HashMap<String, usize>> {
            unimplemented!()
        }

        async fn get_statistics(
            &self,
        ) -> DataStoreResult<std::collections::HashMap<String, serde_json::Value>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_with_transaction_success() {
        let datastore = MockDataStore { should_fail: false };

        let result = with_transaction(
            &datastore,
            |_tx| async move { Ok::<i32, DataStoreError>(42) },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_transaction_operation_failure() {
        let datastore = MockDataStore { should_fail: false };

        let result = with_transaction(&datastore, |_tx| async move {
            Err::<i32, DataStoreError>(DataStoreError::ValidationError {
                message: "test error".to_string(),
            })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_transaction_begin_failure() {
        let datastore = MockDataStore { should_fail: true };

        let result = with_transaction(
            &datastore,
            |_tx| async move { Ok::<i32, DataStoreError>(42) },
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_transaction_control_success() {
        let datastore = MockDataStore { should_fail: false };

        let result = with_transaction_control(&datastore, |_tx| async move {
            Ok::<(i32, bool), DataStoreError>((42, true))
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_transaction_control_failure() {
        let datastore = MockDataStore { should_fail: false };

        let result = with_transaction_control(&datastore, |_tx| async move {
            Err::<(i32, bool), DataStoreError>(DataStoreError::ValidationError {
                message: "test error".to_string(),
            })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_with_transaction_success() {
        let datastore = MockDataStore { should_fail: false };

        // Test with empty operations to verify the function signature works
        let empty_operations: Vec<fn() -> std::future::Ready<DataStoreResult<i32>>> = vec![];
        let result = batch_with_transaction(&datastore, empty_operations).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_batch_with_transaction_failure() {
        let datastore = MockDataStore { should_fail: true };

        // Test with transaction creation failure
        let empty_operations: Vec<fn() -> std::future::Ready<DataStoreResult<i32>>> = vec![];
        let result = batch_with_transaction(&datastore, empty_operations).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_with_transaction_empty() {
        let datastore = MockDataStore { should_fail: false };

        let operations: Vec<fn() -> std::future::Ready<DataStoreResult<i32>>> = vec![];

        let result = batch_with_transaction(&datastore, operations).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_retry_transaction_success_first_try() {
        let datastore = MockDataStore { should_fail: false };

        let result =
            retry_transaction(&datastore, 3, || async { Ok::<i32, DataStoreError>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_transaction_all_failures() {
        let datastore = MockDataStore { should_fail: false };

        let result = retry_transaction(&datastore, 2, || async {
            Err::<i32, DataStoreError>(DataStoreError::ValidationError {
                message: "persistent error".to_string(),
            })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_transaction_success_after_failures() {
        let datastore = MockDataStore { should_fail: false };
        let attempt_count = Arc::new(Mutex::new(0));

        let attempt_counter = attempt_count.clone();
        let result = retry_transaction(&datastore, 3, move || {
            let counter = attempt_counter.clone();
            async move {
                let mut count = counter.lock().unwrap();
                *count += 1;
                if *count < 3 {
                    Err::<i32, DataStoreError>(DataStoreError::ValidationError {
                        message: "temporary error".to_string(),
                    })
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempt_count.lock().unwrap(), 3);
    }
}
