/// Tests for node add functionality
#[cfg(test)]
mod tests {
    use super::super::add::add_node;
    use super::super::types::AddNodeArgs;
    use unet_core::datastore::{MockDataStore, testing::ready_ok};

    #[tokio::test]
    async fn test_add_node_success_with_optional_fields() {
        let last_node = std::sync::Arc::new(std::sync::Mutex::new(None));
        let captured_node = last_node.clone();

        let mut store = MockDataStore::new();
        store.expect_create_node().returning(move |node| {
            *captured_node.lock().expect("lock last_node in create_node") = Some(node.clone());
            ready_ok(node.clone())
        });

        let args = AddNodeArgs {
            name: "edge-1".to_string(),
            domain: "example.com".to_string(),
            vendor: "cisco".to_string(),
            model: "ISR4321".to_string(),
            role: "router".to_string(),
            lifecycle: "live".to_string(),
            location_id: Some(uuid::Uuid::new_v4()),
            management_ip: Some("192.0.2.10".to_string()),
            custom_data: Some("{\"region\":\"us-east\"}".to_string()),
        };

        let result = add_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());

        let saved = last_node
            .lock()
            .expect("lock last_node after call")
            .clone()
            .expect("node should be saved");
        assert_eq!(saved.name, "edge-1");
        assert_eq!(saved.vendor, unet_core::models::Vendor::Cisco);
        assert!(saved.custom_data.get("region").is_some());
    }

    #[tokio::test]
    async fn test_add_node_invalid_vendor_returns_error() {
        let store = MockDataStore::new();
        let args = AddNodeArgs {
            name: "edge-1".to_string(),
            domain: "example.com".to_string(),
            vendor: "invalid".to_string(),
            model: "X".to_string(),
            role: "router".to_string(),
            lifecycle: "planned".to_string(),
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let err = add_node(args, &store, crate::OutputFormat::Json)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Invalid vendor"));
    }
}
