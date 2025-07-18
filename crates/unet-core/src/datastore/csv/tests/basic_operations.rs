//! Basic CSV datastore operations tests

#[cfg(test)]
mod basic_operations_tests {
    use super::super::super::store::{CsvData, CsvStore};
    use crate::models::{DeviceRole, LinkBuilder, LocationBuilder, NodeBuilder, Vendor};

    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_csv_store_new() {
        let temp_dir = TempDir::new().unwrap();
        let store = CsvStore::new(temp_dir.path().join("test.csv")).await;
        assert!(store.is_ok());
    }

    #[tokio::test]
    async fn test_csv_store_new_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("path").join("test.csv");
        let store = CsvStore::new(&nested_path).await;
        assert!(store.is_ok());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_csv_data_default() {
        let data = CsvData::default();
        assert!(data.nodes.is_empty());
        assert!(data.links.is_empty());
        assert!(data.locations.is_empty());
    }

    #[test]
    fn test_csv_data_operations() {
        let mut data = CsvData::default();

        // Add a node
        let node_id = Uuid::new_v4();
        let node = NodeBuilder::new()
            .name("test-node")
            .id(node_id)
            .vendor(Vendor::Cisco)
            .model("test-model")
            .role(DeviceRole::Router)
            .domain("example.com")
            .build()
            .unwrap();
        data.nodes.insert(node_id, node);

        // Add a link
        let link_id = Uuid::new_v4();
        let dest_node_id = Uuid::new_v4();
        let link = LinkBuilder::new()
            .id(link_id)
            .name("test-link")
            .source_node_id(node_id)
            .node_a_interface("eth0")
            .dest_node_id(dest_node_id)
            .node_z_interface("eth1")
            .build()
            .unwrap();
        data.links.insert(link_id, link);

        // Add a location
        let location_id = Uuid::new_v4();
        let location = LocationBuilder::new()
            .name("test-location")
            .id(location_id)
            .location_type("datacenter")
            .build()
            .unwrap();
        data.locations.insert(location_id, location);

        // Verify data
        assert_eq!(data.nodes.len(), 1);
        assert_eq!(data.links.len(), 1);
        assert_eq!(data.locations.len(), 1);

        assert_eq!(data.nodes.get(&node_id).unwrap().name, "test-node");
        assert_eq!(data.links.get(&link_id).unwrap().source_node_id, node_id);
        assert_eq!(
            data.locations.get(&location_id).unwrap().name,
            "test-location"
        );
    }

    #[tokio::test]
    async fn test_csv_store_properties() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test.csv");
        let store = CsvStore::new(&store_path).await.unwrap();

        // Verify store properties
        assert_eq!(store.base_path, store_path);
        assert!(store.data.lock().await.nodes.is_empty());
        assert!(store.data.lock().await.links.is_empty());
        assert!(store.data.lock().await.locations.is_empty());
    }

    #[tokio::test]
    async fn test_csv_store_path_handling() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("csv_store");

        // Test with directory that doesn't exist
        let store = CsvStore::new(&store_path).await;
        assert!(store.is_ok());

        // Verify the store has the correct path
        let store = store.unwrap();
        assert_eq!(store.base_path, store_path);
    }

    #[tokio::test]
    async fn test_csv_store_concurrent_access() {
        let temp_dir = TempDir::new().unwrap();
        let store = CsvStore::new(temp_dir.path().join("test.csv"))
            .await
            .unwrap();

        // Test concurrent access to the data
        let data_arc = store.data.clone();

        // Lock data and modify it
        {
            let mut data = data_arc.lock().await;
            let node_id = Uuid::new_v4();
            let node = NodeBuilder::new()
                .name("concurrent-test")
                .id(node_id)
                .vendor(Vendor::Cisco)
                .model("test-model")
                .role(DeviceRole::Router)
                .domain("example.com")
                .build()
                .unwrap();
            data.nodes.insert(node_id, node);
        }

        // Verify from another lock
        {
            let data = data_arc.lock().await;
            assert_eq!(data.nodes.len(), 1);
            drop(data);
        }
    }

    #[tokio::test]
    async fn test_csv_store_load_data_empty() {
        let temp_dir = TempDir::new().unwrap();
        let store = CsvStore::new(temp_dir.path().join("test.csv"))
            .await
            .unwrap();

        // Loading empty data should work
        let result = store.load_data().await;
        assert!(result.is_ok());

        // Data should be empty
        let data = store.data.lock().await;
        assert!(data.nodes.is_empty());
        assert!(data.links.is_empty());
        assert!(data.locations.is_empty());
        drop(data);
    }
}
