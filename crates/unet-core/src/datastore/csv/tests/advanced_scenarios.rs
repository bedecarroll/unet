//! Advanced scenarios and edge cases for CSV datastore

#[cfg(test)]
mod advanced_scenarios_tests {
    use super::super::super::store::CsvStore;
    use crate::models::{DeviceRole, LinkBuilder, LocationBuilder, NodeBuilder, Vendor};

    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_csv_store_large_data_set() {
        let temp_dir = TempDir::new().unwrap();
        let store = CsvStore::new(temp_dir.path().join("test_store"))
            .await
            .unwrap();

        // Add a large number of nodes
        let node_count = 1000;
        let mut node_ids = Vec::new();

        {
            let mut data = store.data.lock().await;
            for i in 0..node_count {
                let node_id = Uuid::new_v4();
                let node = NodeBuilder::new()
                    .name(format!("node-{i}"))
                    .id(node_id)
                    .vendor(Vendor::Cisco)
                    .model(format!("model-{i}"))
                    .role(DeviceRole::Router)
                    .domain("example.com")
                    .build()
                    .unwrap();
                data.nodes.insert(node_id, node);
                node_ids.push(node_id);
            }
            drop(data);
        }

        // Save and reload
        store.save_data().await.unwrap();

        // Clear and reload
        {
            let mut data = store.data.lock().await;
            data.nodes.clear();
        }

        store.load_data().await.unwrap();

        // Verify all nodes were loaded
        let data = store.data.lock().await;
        assert_eq!(data.nodes.len(), node_count);

        // Verify some specific nodes
        for (i, node_id) in node_ids.iter().enumerate() {
            let node = data.nodes.get(node_id).unwrap();
            assert_eq!(node.name, format!("node-{i}"));
            assert_eq!(node.model, format!("model-{i}"));
        }
        drop(data);
    }

    #[tokio::test]
    async fn test_csv_store_data_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let store = CsvStore::new(temp_dir.path().join("test_store"))
            .await
            .unwrap();

        // Create complex data relationships
        let location_id = Uuid::new_v4();
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();
        let link_id = Uuid::new_v4();

        {
            let mut data = store.data.lock().await;

            // Create location
            let location = LocationBuilder::new()
                .name("datacenter-1")
                .id(location_id)
                .location_type("datacenter")
                .build()
                .unwrap();
            data.locations.insert(location_id, location);

            // Create nodes
            let node1 = NodeBuilder::new()
                .name("router-1")
                .id(node1_id)
                .vendor(Vendor::Cisco)
                .model("ASR1000")
                .role(DeviceRole::Router)
                .domain("example.com")
                .location_id(location_id)
                .build()
                .unwrap();
            data.nodes.insert(node1_id, node1);

            let node2 = NodeBuilder::new()
                .name("router-2")
                .id(node2_id)
                .vendor(Vendor::Juniper)
                .model("MX960")
                .role(DeviceRole::Router)
                .domain("example.com")
                .location_id(location_id)
                .build()
                .unwrap();
            data.nodes.insert(node2_id, node2);

            // Create link between nodes
            let link = LinkBuilder::new()
                .id(link_id)
                .name("router-1-to-router-2")
                .source_node_id(node1_id)
                .node_a_interface("GigabitEthernet0/0/0")
                .dest_node_id(node2_id)
                .node_z_interface("ge-0/0/0")
                .build()
                .unwrap();
            data.links.insert(link_id, link);
        }

        // Save and reload
        store.save_data().await.unwrap();

        // Clear and reload
        {
            let mut data = store.data.lock().await;
            data.nodes.clear();
            data.links.clear();
            data.locations.clear();
        }

        store.load_data().await.unwrap();

        // Verify relationships are preserved
        let data = store.data.lock().await;
        assert_eq!(data.nodes.len(), 2);
        assert_eq!(data.links.len(), 1);
        assert_eq!(data.locations.len(), 1);

        let node1 = data.nodes.get(&node1_id).unwrap();
        let node2 = data.nodes.get(&node2_id).unwrap();
        let link = data.links.get(&link_id).unwrap();
        let location = data.locations.get(&location_id).unwrap();

        assert_eq!(node1.location_id, Some(location_id));
        assert_eq!(node2.location_id, Some(location_id));
        assert_eq!(link.source_node_id, node1_id);
        assert_eq!(link.dest_node_id, Some(node2_id));
        assert_eq!(location.name, "datacenter-1");
        drop(data);
    }
}
