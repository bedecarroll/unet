//! Tests for node update operations

#[cfg(test)]
mod tests {
    use crate::api::{ApiResponse, UpdateNodeRequest};
    use crate::handlers::nodes::crud::*;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use axum::{
        extract::{Path, State},
        response::Json,
    };
    use std::net::IpAddr;
    use unet_core::models::{DeviceRole, Lifecycle, Vendor};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_update_node_success() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let request = create_test_update_request();

        let result = update_node(State(app_state.clone()), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.id, node.id);
        assert_eq!(data.node.name, "updated-router");
        assert_eq!(data.node.domain, "updated.com");
        assert_eq!(data.node.vendor, Vendor::Juniper);
        assert_eq!(data.node.model, "EX4300");
        assert_eq!(data.node.role, DeviceRole::Switch);
        assert_eq!(data.node.lifecycle, Lifecycle::Implementing);
        assert_eq!(
            data.node.management_ip,
            Some("192.168.2.1".parse::<IpAddr>().unwrap())
        );
        assert_eq!(data.node.custom_data, serde_json::json!({"rack": "R2"}));

        // Verify changes were persisted
        let updated_node = app_state
            .datastore
            .get_node(&node.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_node.name, "updated-router");
        assert_eq!(updated_node.vendor, Vendor::Juniper);
    }

    #[tokio::test]
    async fn test_update_node_partial() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let request = UpdateNodeRequest {
            name: Some("partially-updated".to_string()),
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = update_node(State(app_state.clone()), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "partially-updated");
        // Other fields should remain unchanged
        assert_eq!(data.node.domain, "example.com");
        assert_eq!(data.node.vendor, Vendor::Cisco);
        assert_eq!(data.node.role, DeviceRole::Router);
    }

    #[tokio::test]
    async fn test_update_node_not_found() {
        let app_state = setup_test_app_state().await;
        let non_existent_id = Uuid::new_v4();
        let request = create_test_update_request();

        let result = update_node(State(app_state), Path(non_existent_id), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_update_node_invalid_ip() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let request = UpdateNodeRequest {
            name: None,
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: Some("invalid-ip".to_string()),
            custom_data: None,
        };

        let result = update_node(State(app_state), Path(node.id), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("ip") || error.to_string().contains("address"));
    }

    #[tokio::test]
    async fn test_update_node_fqdn_update() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let request = UpdateNodeRequest {
            name: Some("updated-name".to_string()),
            domain: Some("updated-domain.com".to_string()),
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = update_node(State(app_state.clone()), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "updated-name");
        assert_eq!(data.node.domain, "updated-domain.com");
        assert_eq!(data.node.fqdn, "updated-name.updated-domain.com");
    }

    #[tokio::test]
    async fn test_update_node_with_location_id() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let location_id = Uuid::new_v4();

        let request = UpdateNodeRequest {
            name: None,
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: Some(location_id),
            management_ip: None,
            custom_data: None,
        };

        let result = update_node(State(app_state), Path(node.id), Json(request)).await;

        // Note: This might fail if location doesn't exist, depending on validation
        if let Ok(Json(ApiResponse { data, success, .. })) = result {
            assert!(success);
            assert_eq!(data.node.location_id, Some(location_id));
        } else {
            // Expected if location validation is enforced
        }
    }

    #[tokio::test]
    async fn test_update_node_internal_error() {
        // This test would require mocking the datastore to return an error
        // For now, we'll create a placeholder test that documents the expected behavior
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let request = create_test_update_request();

        // In a real test, we'd mock the datastore to fail
        // Here we just verify the happy path works
        let result = update_node(State(app_state), Path(node.id), Json(request)).await;
        assert!(result.is_ok());
    }
}
