/// Execution tests for node history command
#[cfg(test)]
mod tests {
    use super::super::history::history_node;
    use super::super::types::{HistoryNodeArgs, HistoryType};
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
    async fn test_history_all_variants() {
        let node = make_node();
        let store = store_with_node(node.clone());

        for history_type in [
            HistoryType::Status,
            HistoryType::Interfaces,
            HistoryType::Metrics,
            HistoryType::System,
            HistoryType::All,
        ] {
            let args = HistoryNodeArgs {
                id: node.id,
                history_type: history_type.clone(),
                limit: 5,
                last_hours: Some(24),
                detailed: true,
            };
            let result = history_node(args, &store, crate::OutputFormat::Json).await;
            assert!(result.is_ok());
        }
    }
}
