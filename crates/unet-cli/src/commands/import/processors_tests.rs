/// Execution tests for import processors
#[cfg(test)]
mod tests {
    use super::super::processors::{import_links, import_locations, import_nodes};
    use super::super::stats::ImportStats;
    use crate::commands::import::ImportArgs;
    use tempfile::TempDir;
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use unet_core::models::{DeviceRole, Location, Vendor};
    use uuid::Uuid;

    fn build_store(
        locations: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        nodes: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        links: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> MockDataStore {
        let location_counter = locations.clone();
        let node_counter = nodes.clone();
        let link_counter = links.clone();

        let mut store = MockDataStore::new();
        store.expect_create_location().returning(move |location| {
            location_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            ready_ok(location.clone())
        });
        store.expect_create_node().returning(move |node| {
            node_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            ready_ok(node.clone())
        });
        store.expect_create_link().returning(move |link| {
            link_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            ready_ok(link.clone())
        });
        store
    }

    fn write_json_files(dir: &std::path::Path) {
        let location = Location {
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
            serde_json::to_string(&vec![location]).unwrap(),
        )
        .unwrap();

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

        let location_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let node_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let link_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let store = build_store(
            location_count.clone(),
            node_count.clone(),
            link_count.clone(),
        );
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

        assert!(location_count.load(std::sync::atomic::Ordering::SeqCst) >= 1);
        assert!(node_count.load(std::sync::atomic::Ordering::SeqCst) >= 1);
        assert!(link_count.load(std::sync::atomic::Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn test_import_processors_dry_run() {
        let dir = TempDir::new().unwrap();
        write_json_files(dir.path());

        let location_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let node_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let link_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let store = build_store(
            location_count.clone(),
            node_count.clone(),
            link_count.clone(),
        );
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

        assert_eq!(location_count.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(node_count.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(link_count.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
}
