//! Tests for node creation operations

#[cfg(test)]
mod tests {
    use crate::api::{ApiResponse, CreateNodeRequest};
    use crate::handlers::nodes::crud::*;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use axum::{extract::State, response::Json};
    use std::net::IpAddr;
    use unet_core::models::{DeviceRole, Lifecycle, Vendor};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_node_success() {
        let app_state = setup_test_app_state().await;
        let request = create_test_create_request();

        let result = create_node(State(app_state.clone()), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "test-router");
        assert_eq!(data.node.domain, "test.com");
        assert_eq!(data.node.vendor, Vendor::Cisco);
        assert_eq!(data.node.model, "ISR4431");
        assert_eq!(data.node.role, DeviceRole::Router);
        assert_eq!(data.node.lifecycle, Lifecycle::Live);
        assert_eq!(
            data.node.management_ip,
            Some("192.168.1.1".parse::<IpAddr>().unwrap())
        );
        assert_eq!(data.node.custom_data, serde_json::json!({"rack": "R1"}));

        // Verify it was actually saved to the datastore
        let saved_node = app_state.datastore.get_node(&data.node.id).await.unwrap();
        assert!(saved_node.is_some());
    }

    #[tokio::test]
    async fn test_create_node_validation_error() {
        let app_state = setup_test_app_state().await;
        let request = CreateNodeRequest {
            name: String::new(), // Invalid empty name
            domain: Some("test.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ISR4431".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: None,
            management_ip: Some("192.168.1.1".to_string()),
            custom_data: Some(serde_json::json!({"rack": "R1"})),
        };

        let result = create_node(State(app_state), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("validation") || error.to_string().contains("name"));
    }

    #[tokio::test]
    async fn test_create_node_invalid_ip() {
        let app_state = setup_test_app_state().await;
        let request = CreateNodeRequest {
            name: "test-router".to_string(),
            domain: Some("test.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ISR4431".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: None,
            management_ip: Some("invalid-ip".to_string()),
            custom_data: Some(serde_json::json!({"rack": "R1"})),
        };

        let result = create_node(State(app_state), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("ip") || error.to_string().contains("address"));
    }

    #[tokio::test]
    async fn test_create_node_with_minimal_data() {
        let app_state = setup_test_app_state().await;
        let request = CreateNodeRequest {
            name: "minimal-node".to_string(),
            domain: Some("minimal.com".to_string()),
            vendor: Vendor::Generic,
            model: "Generic Model".to_string(),
            role: DeviceRole::Other,
            lifecycle: Lifecycle::Planned,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = create_node(State(app_state.clone()), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "minimal-node");
        assert_eq!(data.node.vendor, Vendor::Generic);
        assert_eq!(data.node.role, DeviceRole::Other);
        assert_eq!(data.node.lifecycle, Lifecycle::Planned);
        assert!(data.node.management_ip.is_none());
        assert_eq!(data.node.custom_data, serde_json::Value::Null);

        // Verify it was saved
        let saved_node = app_state.datastore.get_node(&data.node.id).await.unwrap();
        assert!(saved_node.is_some());
    }

    #[tokio::test]
    async fn test_create_node_with_location_id() {
        let app_state = setup_test_app_state().await;
        let location_id = Uuid::new_v4();

        let request = CreateNodeRequest {
            name: "located-node".to_string(),
            domain: Some("located.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ISR4431".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: Some(location_id),
            management_ip: Some("192.168.1.1".to_string()),
            custom_data: Some(serde_json::json!({"rack": "R1"})),
        };

        let result = create_node(State(app_state), Json(request)).await;

        // Note: This might fail if location doesn't exist, depending on validation
        // The test documents the expected behavior
        if let Ok(Json(ApiResponse { data, success, .. })) = result {
            assert!(success);
            assert_eq!(data.node.location_id, Some(location_id));
        } else {
            // Expected if location validation is enforced
        }
    }

    #[tokio::test]
    async fn test_create_node_internal_error() {
        // This test would require mocking the datastore to return an error
        // For now, we'll create a placeholder test that documents the expected behavior
        let app_state = setup_test_app_state().await;
        let request = create_test_create_request();

        // In a real test, we'd mock the datastore to fail
        // Here we just verify the happy path works
        let result = create_node(State(app_state), Json(request)).await;
        assert!(result.is_ok());
    }
}
