/// Execution tests for node update command
#[cfg(test)]
mod tests {
    use super::super::types::UpdateNodeArgs;
    use super::super::update::update_node;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use unet_core::datastore::DataStore;
    use uuid::Uuid;

    #[derive(Clone)]
    struct Store {
        node: unet_core::models::Node,
        last_updated: std::sync::Arc<std::sync::Mutex<Option<unet_core::models::Node>>>,
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
            let updated = self.last_updated.lock().unwrap().clone();
            Ok(Some(updated.unwrap_or_else(|| self.node.clone())))
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
            *self.last_updated.lock().unwrap() = Some(node.clone());
            *self.last_updated.lock().unwrap() = Some(node.clone());
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

    fn make_node() -> unet_core::models::Node {
        use unet_core::models::*;
        let id = Uuid::new_v4();
        NodeBuilder::new()
            .id(id)
            .name("core-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_update_node_changes_fields() {
        let node = make_node();
        let store = Store {
            node: node.clone(),
            last_updated: Default::default(),
        };
        let args = UpdateNodeArgs {
            id: node.id,
            name: Some("core-1b".to_string()),
            domain: Some("corp.local".to_string()),
            vendor: Some("cisco".to_string()),
            model: Some("ISR4331".to_string()),
            role: Some("router".to_string()),
            lifecycle: Some("live".to_string()),
            location_id: None,
            management_ip: Some("192.0.2.20".to_string()),
            custom_data: Some("{\"site\":\"dc1\"}".to_string()),
        };

        let result = update_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_node_invalid_vendor_error() {
        let node = make_node();
        let store = Store {
            node: node.clone(),
            last_updated: Default::default(),
        };
        let args = UpdateNodeArgs {
            id: node.id,
            name: None,
            domain: None,
            vendor: Some("invalid".to_string()),
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };
        let err = update_node(args, &store, crate::OutputFormat::Json)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Invalid vendor"));
    }

    #[tokio::test]
    async fn test_update_node_fqdn_recomputed() {
        let node = make_node();
        let store = Store {
            node: node.clone(),
            last_updated: Default::default(),
        };

        // Domain -> empty, fqdn should equal name
        let args = UpdateNodeArgs {
            id: node.id,
            name: None,
            domain: Some(String::new()),
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };
        update_node(args, &store, crate::OutputFormat::Json)
            .await
            .unwrap();
        let updated = store.last_updated.lock().unwrap().clone().unwrap();
        assert_eq!(updated.fqdn, updated.name);

        // Name change should recompute fqdn (domain remains empty)
        let args2 = UpdateNodeArgs {
            id: updated.id,
            name: Some("newname".to_string()),
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };
        update_node(args2, &store, crate::OutputFormat::Json)
            .await
            .unwrap();
        let updated2 = store.last_updated.lock().unwrap().clone().unwrap();
        assert_eq!(updated2.fqdn, updated2.name);
    }
}
