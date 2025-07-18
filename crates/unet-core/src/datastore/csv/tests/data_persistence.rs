//! Data persistence tests for CSV datastore

#[cfg(test)]
mod data_persistence_tests {
    use super::super::super::store::CsvStore;
    use crate::models::{DeviceRole, LinkBuilder, LocationBuilder, NodeBuilder, Vendor};

    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_csv_store_save_and_load_data() {
        let temp_dir = TempDir::new().unwrap();
        let store = CsvStore::new(temp_dir.path().join("test_store"))
            .await
            .unwrap();

        // Add some test data
        let node_id = Uuid::new_v4();
        let dest_node_id = Uuid::new_v4();
        let link_id = Uuid::new_v4();
        let location_id = Uuid::new_v4();

        {
            let mut data = store.data.lock().await;

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

            let location = LocationBuilder::new()
                .name("test-location")
                .id(location_id)
                .location_type("datacenter")
                .build()
                .unwrap();
            data.locations.insert(location_id, location);
        }

        // Save the data
        let result = store.save_data().await;
        assert!(result.is_ok());

        // Verify files were created
        assert!(store.base_path.join("nodes.json").exists());
        assert!(store.base_path.join("links.json").exists());
        assert!(store.base_path.join("locations.json").exists());

        // Clear data and reload
        {
            let mut data = store.data.lock().await;
            data.nodes.clear();
            data.links.clear();
            data.locations.clear();
        }

        let result = store.load_data().await;
        assert!(result.is_ok());

        // Verify data was loaded correctly
        let data = store.data.lock().await;
        assert_eq!(data.nodes.len(), 1);
        assert_eq!(data.links.len(), 1);
        assert_eq!(data.locations.len(), 1);

        assert_eq!(data.nodes.get(&node_id).unwrap().name, "test-node");
        assert_eq!(data.links.get(&link_id).unwrap().name, "test-link");
        assert_eq!(
            data.locations.get(&location_id).unwrap().name,
            "test-location"
        );
        drop(data);
    }

    #[tokio::test]
    async fn test_csv_store_save_empty_data() {
        let temp_dir = TempDir::new().unwrap();
        let store = CsvStore::new(temp_dir.path().join("test_store"))
            .await
            .unwrap();

        // Save empty data
        let result = store.save_data().await;
        assert!(result.is_ok());

        // Verify files were created with empty arrays
        assert!(store.base_path.join("nodes.json").exists());
        assert!(store.base_path.join("links.json").exists());
        assert!(store.base_path.join("locations.json").exists());

        // Read and verify content
        let nodes_content = tokio::fs::read_to_string(store.base_path.join("nodes.json"))
            .await
            .unwrap();
        let links_content = tokio::fs::read_to_string(store.base_path.join("links.json"))
            .await
            .unwrap();
        let locations_content = tokio::fs::read_to_string(store.base_path.join("locations.json"))
            .await
            .unwrap();

        assert_eq!(nodes_content.trim(), "[]");
        assert_eq!(links_content.trim(), "[]");
        assert_eq!(locations_content.trim(), "[]");
    }

    #[tokio::test]
    async fn test_csv_store_load_malformed_data() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store");

        // Create directory first
        tokio::fs::create_dir_all(&store_path).await.unwrap();

        // Create malformed JSON files
        tokio::fs::write(store_path.join("nodes.json"), "invalid json")
            .await
            .unwrap();

        let store = CsvStore::new(&store_path).await;
        assert!(store.is_err());

        // Verify error type
        match store.unwrap_err() {
            crate::datastore::DataStoreError::InternalError { message } => {
                assert!(message.contains("Failed to parse nodes"));
            }
            _ => panic!("Expected InternalError"),
        }
    }

    #[tokio::test]
    async fn test_csv_store_multiple_instances() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store");

        // Create first store and add data
        let store1 = CsvStore::new(&store_path).await.unwrap();
        let node_id = Uuid::new_v4();
        {
            let mut data = store1.data.lock().await;
            let node = NodeBuilder::new()
                .name("store1-node")
                .id(node_id)
                .vendor(Vendor::Cisco)
                .model("test-model")
                .role(DeviceRole::Router)
                .domain("example.com")
                .build()
                .unwrap();
            data.nodes.insert(node_id, node);
        }
        store1.save_data().await.unwrap();

        // Create second store and verify it loads the data
        let store2 = CsvStore::new(&store_path).await.unwrap();
        let data2 = store2.data.lock().await;
        assert_eq!(data2.nodes.len(), 1);
        assert_eq!(data2.nodes.get(&node_id).unwrap().name, "store1-node");
        drop(data2);
    }
}
