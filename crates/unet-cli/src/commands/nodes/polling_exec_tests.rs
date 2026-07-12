/// Execution tests for node polling command
#[cfg(test)]
mod tests {
    use super::super::polling::polling_node;
    use super::super::types::{PollingAction, PollingNodeArgs};
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
    async fn test_polling_actions_all_variants() {
        let node = make_node();
        let store = store_with_node(node.clone());

        for action in [
            PollingAction::Status,
            PollingAction::Start,
            PollingAction::Stop,
            PollingAction::Restart,
            PollingAction::History,
        ] {
            let args = PollingNodeArgs {
                id: node.id,
                action: action.clone(),
                detailed: true,
            };
            let result = polling_node(args, &store, crate::OutputFormat::Json).await;
            assert!(result.is_ok());
        }
    }
}
