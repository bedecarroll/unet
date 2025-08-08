/// Execution tests for link CRUD commands
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
        link: Option<unet_core::models::Link>,
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
        async fn create_link(&self, link: &unet_core::models::Link) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> { Ok(link.clone()) }
        async fn get_link(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Link>> { Ok(self.link.clone()) }
        async fn list_links(&self, options: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::PagedResult<unet_core::models::Link>> {
            *self.last_options.lock().unwrap() = Some(options.clone());
            Ok(unet_core::datastore::types::PagedResult::new(vec![], 0, options.pagination.as_ref()))
        }
        async fn update_link(&self, link: &unet_core::models::Link) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> { Ok(link.clone()) }
        async fn delete_link(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> { self.deleted.store(true, std::sync::atomic::Ordering::SeqCst); Ok(()) }
        async fn get_links_for_node(&self, _node_id: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> { unimplemented!("not needed") }
        async fn get_links_between_nodes(&self, _first_node_id: &Uuid, _second_node_id: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> { unimplemented!("not needed") }
        async fn create_location(&self, _location: &unet_core::models::Location) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> { unimplemented!("not needed") }
        async fn get_location(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Location>> { Ok(None) }
        async fn list_locations(&self, _options: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::PagedResult<unet_core::models::Location>> { unimplemented!("not needed") }
        async fn update_location(&self, _location: &unet_core::models::Location) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> { unimplemented!("not needed") }
        async fn delete_location(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> { unimplemented!("not needed") }
        async fn create_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> { unimplemented!("not needed") }
        async fn list_vendors(&self) -> unet_core::datastore::DataStoreResult<Vec<String>> { unimplemented!("not needed") }
        async fn delete_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> { unimplemented!("not needed") }
        async fn batch_nodes(&self, _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Node>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult> { unimplemented!("not needed") }
        async fn batch_links(&self, _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Link>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult> { unimplemented!("not needed") }
        async fn batch_locations(&self, _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Location>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult> { unimplemented!("not needed") }
        async fn get_entity_counts(&self) -> unet_core::datastore::DataStoreResult<HashMap<String, usize>> { unimplemented!("not needed") }
        async fn get_statistics(&self) -> unet_core::datastore::DataStoreResult<HashMap<String, serde_json::Value>> { unimplemented!("not needed") }
    }

    fn example_link() -> unet_core::models::Link {
        unet_core::models::Link::new("l1".to_string(), Uuid::new_v4(), "e0".to_string(), Uuid::new_v4(), "e1".to_string())
    }

    #[tokio::test]
    async fn test_add_and_show_update_delete_link() {
        let store = Store::default();
        // Add
        let args = AddLinkArgs { name: "l1".to_string(), node_a_id: Uuid::new_v4(), node_a_interface: "e0".to_string(), node_z_id: Some(Uuid::new_v4()), node_z_interface: Some("e1".to_string()), bandwidth_bps: Some(1_000_000), description: Some("desc".to_string()), custom_data: Some("{\"prio\":1}".to_string()) };
        assert!(add_link(args, &store, crate::OutputFormat::Json).await.is_ok());

        // Show
        let link = example_link();
        let store = Store { link: Some(link.clone()), ..Default::default() };
        let show_args = ShowLinkArgs { id: link.id };
        assert!(show_link(show_args, &store, crate::OutputFormat::Json).await.is_ok());

        // Update
        let upd_args = UpdateLinkArgs { id: link.id, name: Some("l2".to_string()), node_a_id: None, node_a_interface: Some("e2".to_string()), node_z_id: None, node_z_interface: None, bandwidth_bps: Some(2_000_000), description: Some("d2".to_string()), custom_data: Some("{\"prio\":2}".to_string()) };
        assert!(update_link(upd_args, &store, crate::OutputFormat::Json).await.is_ok());

        // Delete with yes
        let del_store = Store { link: Some(link.clone()), ..Default::default() };
        let del_args = DeleteLinkArgs { id: link.id, yes: true };
        assert!(delete_link(del_args, &del_store, crate::OutputFormat::Json).await.is_ok());
        assert!(del_store.deleted.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_list_links_captures_filters() {
        let store = Store::default();
        let args = ListLinkArgs { node_id: Some(Uuid::new_v4()), page: 2, per_page: 10 };
        assert!(list_links(args, &store, crate::OutputFormat::Json).await.is_ok());
        let opts = store.last_options.lock().unwrap().clone().unwrap();
        assert!(opts.pagination.is_some());
    }
}

