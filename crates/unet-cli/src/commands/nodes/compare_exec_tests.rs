/// Execution tests for node compare command
#[cfg(test)]
mod tests {
    use super::super::compare::compare_nodes;
    use super::super::types::{CompareNodeArgs, CompareType};
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use uuid::Uuid;

    fn store_with_node(node: unet_core::models::Node) -> MockDataStore {
        let mut store = MockDataStore::new();
        store
            .expect_get_node_required()
            .returning(move |_| ready_ok(node.clone()));
        store
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
    async fn test_compare_two_nodes_all() {
        let node = make_node();
        let store = store_with_node(node.clone());
        let args = CompareNodeArgs {
            node_a: node.id,
            node_b: Some(Uuid::new_v4()),
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        let result = compare_nodes(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compare_historical() {
        let node = make_node();
        let store = store_with_node(node);
        let args = CompareNodeArgs {
            node_a: Uuid::new_v4(),
            node_b: None,
            compare_type: vec![CompareType::Interfaces],
            diff_only: true,
        };

        let result = compare_nodes(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
    }
}
