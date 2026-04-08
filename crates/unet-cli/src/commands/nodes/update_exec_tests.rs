/// Execution tests for node update command
#[cfg(test)]
mod tests {
    use super::super::types::UpdateNodeArgs;
    use super::super::update::update_node;
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use uuid::Uuid;

    fn make_node() -> unet_core::models::Node {
        use unet_core::models::*;
        let id = Uuid::new_v4();
        NodeBuilder::new()
            .id(id)
            .name("core-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .build()
            .unwrap()
    }

    fn store_for_updates(
        original: unet_core::models::Node,
        last_updated: std::sync::Arc<std::sync::Mutex<Option<unet_core::models::Node>>>,
    ) -> MockDataStore {
        let current_node = std::sync::Arc::new(std::sync::Mutex::new(original));
        let get_node_state = current_node.clone();
        let update_state = current_node.clone();
        let update_capture = last_updated.clone();

        let mut store = MockDataStore::new();
        store.expect_get_node_required().returning(move |_| {
            ready_ok(
                get_node_state
                    .lock()
                    .expect("lock current_node in get_node")
                    .clone(),
            )
        });
        store.expect_update_node().returning(move |node| {
            *update_capture
                .lock()
                .expect("lock last_updated in update_node") = Some(node.clone());
            *update_state
                .lock()
                .expect("lock current_node in update_node") = node.clone();
            ready_ok(node.clone())
        });
        store
    }

    #[tokio::test]
    async fn test_update_node_changes_fields() {
        let node = make_node();
        let last_updated = std::sync::Arc::new(std::sync::Mutex::new(None));
        let store = store_for_updates(node.clone(), last_updated);
        let args = UpdateNodeArgs {
            id: node.id,
            name: Some("core-1b".to_string()),
            domain: Some("corp.local".to_string()),
            vendor: Some("cisco".to_string()),
            model: Some("ISR4331".to_string()),
            role: Some("router".to_string()),
            lifecycle: Some("live".to_string()),
            location_id: None,
            management_ip: Some("192.0.2.20".to_string()),
            custom_data: Some("{\"site\":\"dc1\"}".to_string()),
        };

        let result = update_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_node_invalid_vendor_error() {
        let node = make_node();
        let last_updated = std::sync::Arc::new(std::sync::Mutex::new(None));
        let store = store_for_updates(node.clone(), last_updated);
        let args = UpdateNodeArgs {
            id: node.id,
            name: None,
            domain: None,
            vendor: Some("invalid".to_string()),
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let err = update_node(args, &store, crate::OutputFormat::Json)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Invalid vendor"));
    }

    #[tokio::test]
    async fn test_update_node_fqdn_recomputed() {
        let node = make_node();
        let last_updated = std::sync::Arc::new(std::sync::Mutex::new(None));
        let store = store_for_updates(node.clone(), last_updated.clone());

        let args = UpdateNodeArgs {
            id: node.id,
            name: None,
            domain: Some(String::new()),
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };
        update_node(args, &store, crate::OutputFormat::Json)
            .await
            .unwrap();
        let updated = last_updated
            .lock()
            .expect("lock last_updated after first update")
            .clone()
            .expect("updated node after first update");
        assert_eq!(updated.fqdn, updated.name);

        let args2 = UpdateNodeArgs {
            id: updated.id,
            name: Some("newname".to_string()),
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };
        update_node(args2, &store, crate::OutputFormat::Json)
            .await
            .unwrap();
        let updated2 = last_updated
            .lock()
            .expect("lock last_updated after second update")
            .clone()
            .expect("updated node after second update");
        assert_eq!(updated2.fqdn, updated2.name);
    }
}
