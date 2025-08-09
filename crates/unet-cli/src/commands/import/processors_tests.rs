/// Execution tests for import processors
#[cfg(test)]
mod tests {
    use super::super::processors::{import_links, import_locations, import_nodes};
    use super::super::stats::ImportStats;
    use crate::commands::import::ImportArgs;
    use async_trait::async_trait;
    use tempfile::TempDir;
    use unet_core::datastore::DataStore;
    use unet_core::models::{DeviceRole, Location, Vendor};
    use uuid::Uuid;

    #[derive(Default)]
    struct Store {
        loc: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        nod: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        lnk: std::sync::Arc<std::sync::atomic::AtomicUsize>,
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
            node: &unet_core::models::Node,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> {
            self.nod.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
            unimplemented!("not needed")
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
        async fn create_link(
            &self,
            link: &unet_core::models::Link,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> {
            self.lnk.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(link.clone())
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
        async fn create_location(
            &self,
            location: &unet_core::models::Location,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> {
            self.loc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(location.clone())
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
    }

    fn write_json_files(dir: &std::path::Path) {
        // locations.json
        let loc = Location {
            id: Uuid::new_v4(),
            name: "loc1".to_string(),
            location_type: "dc".to_string(),
            parent_id: None,
            path: "loc1".to_string(),
            description: None,
            address: None,
            custom_data: serde_json::Value::Null,
        };
        std::fs::write(
            dir.join("locations.json"),
            serde_json::to_string(&vec![loc]).unwrap(),
        )
        .unwrap();

        // nodes.json
        let mut node = unet_core::models::Node::new(
            "n1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "m1".to_string();
        std::fs::write(
            dir.join("nodes.json"),
            serde_json::to_string(&vec![node]).unwrap(),
        )
        .unwrap();

        // links.json
        let link = unet_core::models::Link::new(
            "l1".to_string(),
            Uuid::new_v4(),
            "e0".to_string(),
            Uuid::new_v4(),
            "e1".to_string(),
        );
        std::fs::write(
            dir.join("links.json"),
            serde_json::to_string(&vec![link]).unwrap(),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_import_processors_happy_path() {
        let dir = TempDir::new().unwrap();
        write_json_files(dir.path());

        let store = Store::default();
        let mut stats = ImportStats::new();
        let args = ImportArgs {
            from: dir.path().to_path_buf(),
            format: None,
            dry_run: false,
            continue_on_error: false,
        };

        import_locations(&args, &store, &mut stats).await.unwrap();
        import_nodes(&args, &store, &mut stats).await.unwrap();
        import_links(&args, &store, &mut stats).await.unwrap();

        assert!(store.loc.load(std::sync::atomic::Ordering::SeqCst) >= 1);
        assert!(store.nod.load(std::sync::atomic::Ordering::SeqCst) >= 1);
        assert!(store.lnk.load(std::sync::atomic::Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn test_import_processors_dry_run() {
        let dir = TempDir::new().unwrap();
        write_json_files(dir.path());

        let store = Store::default();
        let mut stats = ImportStats::new();
        let args = ImportArgs {
            from: dir.path().to_path_buf(),
            format: None,
            dry_run: true,
            continue_on_error: false,
        };

        import_locations(&args, &store, &mut stats).await.unwrap();
        import_nodes(&args, &store, &mut stats).await.unwrap();
        import_links(&args, &store, &mut stats).await.unwrap();

        assert_eq!(store.loc.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(store.nod.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(store.lnk.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
}
