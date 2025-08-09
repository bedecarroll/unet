/// Tests for node add functionality
#[cfg(test)]
mod tests {
    use super::super::add::add_node;
    use super::super::types::AddNodeArgs;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use unet_core::datastore::DataStore;
    use uuid::Uuid;

    // Minimal mock datastore for exercising add_node
    struct MockStore {
        last_node: Arc<Mutex<Option<unet_core::models::Node>>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                last_node: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[async_trait]
    impl DataStore for MockStore {
        fn name(&self) -> &'static str {
            "mock"
        }

        async fn health_check(&self) -> unet_core::datastore::DataStoreResult<()> {
            Ok(())
        }

        async fn begin_transaction(
            &self,
        ) -> unet_core::datastore::DataStoreResult<Box<dyn unet_core::datastore::Transaction>>
        {
            unimplemented!("not needed")
        }

        async fn create_node(
            &self,
            node: &unet_core::models::Node,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> {
            let mut guard = self
                .last_node
                .lock()
                .expect("lock last_node in create_node");
            *guard = Some(node.clone());
            Ok(node.clone())
        }

        async fn get_node(
            &self,
            _id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Node>> {
            Ok(None)
        }

        async fn list_nodes(
            &self,
            _options: &unet_core::datastore::QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Node>,
        > {
            Ok(unet_core::datastore::types::PagedResult::new(
                vec![],
                0,
                None,
            ))
        }

        async fn update_node(
            &self,
            node: &unet_core::models::Node,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> {
            Ok(node.clone())
        }

        async fn delete_node(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }

        async fn get_nodes_by_location(
            &self,
            _location_id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Node>> {
            unimplemented!("not needed")
        }

        async fn search_nodes_by_name(
            &self,
            _name: &str,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Node>> {
            unimplemented!("not needed")
        }

        async fn create_link(
            &self,
            _link: &unet_core::models::Link,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> {
            unimplemented!("not needed")
        }

        async fn get_link(
            &self,
            _id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Link>> {
            Ok(None)
        }

        async fn list_links(
            &self,
            _options: &unet_core::datastore::QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Link>,
        > {
            unimplemented!("not needed")
        }

        async fn update_link(
            &self,
            _link: &unet_core::models::Link,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> {
            unimplemented!("not needed")
        }

        async fn delete_link(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }

        async fn get_links_for_node(
            &self,
            _node_id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> {
            unimplemented!("not needed")
        }

        async fn get_links_between_nodes(
            &self,
            _first_node_id: &Uuid,
            _second_node_id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> {
            unimplemented!("not needed")
        }

        async fn create_location(
            &self,
            _location: &unet_core::models::Location,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> {
            unimplemented!("not needed")
        }

        async fn get_location(
            &self,
            _id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Location>> {
            Ok(None)
        }

        async fn list_locations(
            &self,
            _options: &unet_core::datastore::QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Location>,
        > {
            unimplemented!("not needed")
        }

        async fn update_location(
            &self,
            _location: &unet_core::models::Location,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> {
            unimplemented!("not needed")
        }

        async fn delete_location(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }

        async fn create_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }

        async fn list_vendors(&self) -> unet_core::datastore::DataStoreResult<Vec<String>> {
            unimplemented!("not needed")
        }

        async fn delete_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }

        async fn batch_nodes(
            &self,
            _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Node>],
        ) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult>
        {
            unimplemented!("not needed")
        }

        async fn batch_links(
            &self,
            _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Link>],
        ) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult>
        {
            unimplemented!("not needed")
        }

        async fn batch_locations(
            &self,
            _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Location>],
        ) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult>
        {
            unimplemented!("not needed")
        }

        async fn get_entity_counts(
            &self,
        ) -> unet_core::datastore::DataStoreResult<HashMap<String, usize>> {
            unimplemented!("not needed")
        }

        async fn get_statistics(
            &self,
        ) -> unet_core::datastore::DataStoreResult<HashMap<String, serde_json::Value>> {
            unimplemented!("not needed")
        }
    }

    #[tokio::test]
    async fn test_add_node_success_with_optional_fields() {
        let store = MockStore::new();
        let args = AddNodeArgs {
            name: "edge-1".to_string(),
            domain: "example.com".to_string(),
            vendor: "cisco".to_string(),
            model: "ISR4321".to_string(),
            role: "router".to_string(),
            lifecycle: "live".to_string(),
            location_id: Some(Uuid::new_v4()),
            management_ip: Some("192.0.2.10".to_string()),
            custom_data: Some("{\"region\":\"us-east\"}".to_string()),
        };

        let result = add_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());

        // Verify node was passed to datastore
        let saved = store
            .last_node
            .lock()
            .expect("lock last_node after call")
            .clone()
            .expect("node should be saved");
        assert_eq!(saved.name, "edge-1");
        assert_eq!(saved.vendor, unet_core::models::Vendor::Cisco);
        assert!(saved.custom_data.get("region").is_some());
    }

    #[tokio::test]
    async fn test_add_node_invalid_vendor_returns_error() {
        let store = MockStore::new();
        let args = AddNodeArgs {
            name: "edge-1".to_string(),
            domain: "example.com".to_string(),
            vendor: "invalid".to_string(),
            model: "X".to_string(),
            role: "router".to_string(),
            lifecycle: "planned".to_string(),
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let err = add_node(args, &store, crate::OutputFormat::Json)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Invalid vendor"));
    }
}
