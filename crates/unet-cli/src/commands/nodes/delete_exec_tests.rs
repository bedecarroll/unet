/// Execution tests for node delete command
#[cfg(test)]
mod tests {
    use super::super::delete::{delete_node, confirm_deletion};
    use super::super::types::DeleteNodeArgs;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use unet_core::datastore::DataStore;
    use uuid::Uuid;

    #[derive(Clone)]
    struct Store {
        node: unet_core::models::Node,
        deleted: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl DataStore for Store {
        fn name(&self) -> &'static str {
            "store"
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
            _node: &unet_core::models::Node,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> {
            unimplemented!("not needed")
        }
        async fn get_node(
            &self,
            _id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Node>> {
            Ok(Some(self.node.clone()))
        }
        async fn list_nodes(
            &self,
            _options: &unet_core::datastore::QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Node>,
        > {
            unimplemented!("not needed")
        }
        async fn update_node(
            &self,
            node: &unet_core::models::Node,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> {
            Ok(node.clone())
        }
        async fn delete_node(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> {
            *self.deleted.lock().unwrap() = true;
            Ok(())
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

    fn make_node() -> unet_core::models::Node {
        use unet_core::models::*;
        let id = Uuid::new_v4();
        NodeBuilder::new()
            .id(id)
            .name("edge-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_delete_node_yes_skips_prompt() {
        let node = make_node();
        let deleted = std::sync::Arc::new(std::sync::Mutex::new(false));
        let store = Store {
            node: node.clone(),
            deleted: deleted.clone(),
        };

        let args = DeleteNodeArgs {
            id: node.id,
            yes: true,
        };
        let result = delete_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
        assert!(*deleted.lock().unwrap());
    }
}
#[cfg(test)]
mod confirm_tests {
    use super::super::delete::confirm_deletion;

    #[test]
    fn test_confirm_deletion_negative_and_positive() {
        // Negative
        let mut cur = std::io::Cursor::new(b"n\n".to_vec());
        let res = confirm_deletion(false, "edge-1", &mut cur).unwrap();
        assert!(!res);

        // Positive
        let mut cur2 = std::io::Cursor::new(b"yes\n".to_vec());
        let res2 = confirm_deletion(false, "edge-1", &mut cur2).unwrap();
        assert!(res2);
    }
}
