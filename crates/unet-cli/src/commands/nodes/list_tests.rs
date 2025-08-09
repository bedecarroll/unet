/// Tests for node list functionality
#[cfg(test)]
mod tests {
    use super::super::list::list_nodes;
    use super::super::types::ListNodeArgs;
    use async_trait::async_trait;
    use unet_core::datastore::{DataStore, QueryOptions};
    use uuid::Uuid;

    #[derive(Default)]
    struct CaptureStore {
        last_options: std::sync::Mutex<Option<QueryOptions>>,
    }

    #[async_trait]
    impl DataStore for CaptureStore {
        fn name(&self) -> &'static str {
            "capture"
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
            Ok(None)
        }

        async fn list_nodes(
            &self,
            options: &QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Node>,
        > {
            *self.last_options.lock().expect("lock last_options") = Some(options.clone());
            Ok(unet_core::datastore::types::PagedResult::new(
                vec![],
                0,
                options.pagination.as_ref(),
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
        ) -> unet_core::datastore::DataStoreResult<std::collections::HashMap<String, usize>>
        {
            unimplemented!("not needed")
        }

        async fn get_statistics(
            &self,
        ) -> unet_core::datastore::DataStoreResult<
            std::collections::HashMap<String, serde_json::Value>,
        > {
            unimplemented!("not needed")
        }
    }

    #[tokio::test]
    async fn test_list_nodes_builds_filters_and_pagination() {
        let store = CaptureStore::default();
        let args = ListNodeArgs {
            lifecycle: Some("live".to_string()),
            role: Some("router".to_string()),
            vendor: Some("cisco".to_string()),
            page: 2,
            per_page: 5,
        };

        let result = list_nodes(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());

        let opts = store
            .last_options
            .lock()
            .expect("lock last_options")
            .clone()
            .expect("options recorded");
        assert_eq!(opts.filters.len(), 3);
        assert!(opts.pagination.is_some());
        let p = opts.pagination.unwrap();
        assert_eq!(p.limit, 5);
        assert_eq!(p.offset, 5); // (page-1)*per_page
    }
}
