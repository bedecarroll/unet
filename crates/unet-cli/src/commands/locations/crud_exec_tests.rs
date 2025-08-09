/// Execution tests for location CRUD commands
#[cfg(test)]
mod tests {
    use super::crud::*;
    use super::types::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use unet_core::datastore::DataStore;
    use uuid::Uuid;

    #[derive(Default, Clone)]
    struct Store {
        location: Option<unet_core::models::Location>,
        last_options: std::sync::Arc<std::sync::Mutex<Option<unet_core::datastore::QueryOptions>>>,
        deleted: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait]
    impl DataStore for Store {
        fn name(&self) -> &'static str { "store" }
        async fn health_check(&self) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn begin_transaction(&self) -> unet_core::datastore::DataStoreResult<Box<dyn unet_core::datastore::Transaction>> { unimplemented!("not needed") }
        async fn create_node(&self, _node: &unet_core::models::Node) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> { unimplemented!("not needed") }
        async fn get_node(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Node>> { Ok(None) }
        async fn list_nodes(&self, _options: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::PagedResult<unet_core::models::Node>> { unimplemented!("not needed") }
        async fn update_node(&self, _node: &unet_core::models::Node) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> { unimplemented!("not needed") }
        async fn delete_node(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> { unimplemented!("not needed") }
        async fn create_location(&self, location: &unet_core::models::Location) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> { Ok(location.clone()) }
        async fn get_location(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Location>> { Ok(self.location.clone()) }
        async fn list_locations(&self, options: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::PagedResult<unet_core::models::Location>> {
            *self.last_options.lock().unwrap() = Some(options.clone());
            Ok(unet_core::datastore::types::PagedResult::new(vec![], 0, options.pagination.as_ref()))
        }
        async fn update_location(&self, location: &unet_core::models::Location) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> { Ok(location.clone()) }
        async fn delete_location(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> { self.deleted.store(true, std::sync::atomic::Ordering::SeqCst); Ok(()) }
        async fn create_link(&self, _link: &unet_core::models::Link) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> { unimplemented!("not needed") }
        async fn get_link(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Link>> { Ok(None) }
        async fn list_links(&self, _options: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::PagedResult<unet_core::models::Link>> { unimplemented!("not needed") }
        async fn update_link(&self, _link: &unet_core::models::Link) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> { unimplemented!("not needed") }
        async fn delete_link(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> { unimplemented!("not needed") }
        async fn get_links_for_node(&self, _node_id: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> { unimplemented!("not needed") }
        async fn get_links_between_nodes(&self, _first_node_id: &Uuid, _second_node_id: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> { unimplemented!("not needed") }
        async fn create_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> { unimplemented!("not needed") }
        async fn list_vendors(&self) -> unet_core::datastore::DataStoreResult<Vec<String>> { unimplemented!("not needed") }
        async fn delete_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> { unimplemented!("not needed") }
        async fn batch_nodes(&self, _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Node>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult> { unimplemented!("not needed") }
        async fn batch_links(&self, _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Link>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult> { unimplemented!("not needed") }
        async fn batch_locations(&self, _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Location>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult> { unimplemented!("not needed") }
        async fn get_entity_counts(&self) -> unet_core::datastore::DataStoreResult<HashMap<String, usize>> { unimplemented!("not needed") }
        async fn get_statistics(&self) -> unet_core::datastore::DataStoreResult<HashMap<String, serde_json::Value>> { unimplemented!("not needed") }
    }

    fn example_location() -> unet_core::models::Location {
        unet_core::models::Location { id: Uuid::new_v4(), name: "loc1".to_string(), location_type: "dc".to_string(), parent_id: None, path: "loc1".to_string(), description: None, address: None, custom_data: serde_json::Value::Null }
    }

    #[tokio::test]
    async fn test_add_show_update_delete_location() {
        let store = Store::default();
        // Add
        let args = AddLocationArgs { name: "loc1".to_string(), location_type: "dc".to_string(), parent_id: None, address: Some("addr".to_string()), city: Some("city".to_string()), country: Some("cty".to_string()), custom_data: Some("{}".to_string()) };
        assert!(add_location(args, &store, crate::OutputFormat::Json).await.is_ok());

        // Show
        let loc = example_location();
        let show_store = Store { location: Some(loc.clone()), ..Default::default() };
        let show_args = ShowLocationArgs { id: loc.id };
        assert!(show_location(show_args, &show_store, crate::OutputFormat::Json).await.is_ok());

        // Update
        let upd_args = UpdateLocationArgs { id: loc.id, name: Some("L2".to_string()), location_type: Some("room".to_string()), parent_id: None, address: Some("a".to_string()), city: Some("b".to_string()), country: Some("c".to_string()), custom_data: Some("{}".to_string()) };
        assert!(update_location(upd_args, &show_store, crate::OutputFormat::Json).await.is_ok());

        // Delete with yes
        let del_store = Store { location: Some(loc.clone()), ..Default::default() };
        let del_args = DeleteLocationArgs { id: loc.id, yes: true };
        assert!(delete_location(del_args, &del_store, crate::OutputFormat::Json).await.is_ok());
        assert!(del_store.deleted.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_list_locations_captures_filters() {
        let store = Store::default();
        let args = ListLocationArgs { location_type: Some("dc".to_string()), parent_id: Some(Uuid::new_v4()), page: 1, per_page: 20 };
        assert!(list_locations(args, &store, crate::OutputFormat::Json).await.is_ok());
        let opts = store.last_options.lock().unwrap().clone().unwrap();
        assert!(opts.pagination.is_some());
        assert_eq!(opts.filters.len(), 2);
    }
}

