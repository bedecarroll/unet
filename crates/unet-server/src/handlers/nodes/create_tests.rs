//! Tests for node creation operations

#[cfg(test)]
mod tests {
    use super::super::crud::*;
    use crate::api::ApiResponse;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use axum::{extract::State, response::Json};
    use std::net::IpAddr;
    use unet_core::models::{DeviceRole, Lifecycle, Vendor};

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
        assert_eq!(saved_node.unwrap().name, "test-router");
    }

    #[tokio::test]
    async fn test_create_node_validation_error() {
        let app_state = setup_test_app_state().await;
        let mut request = create_test_create_request();
        request.name = String::new(); // Invalid empty name

        let result = create_node(State(app_state), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("validation failed"));
    }

    #[tokio::test]
    async fn test_create_node_invalid_ip() {
        let app_state = setup_test_app_state().await;
        let mut request = create_test_create_request();
        request.management_ip = Some("invalid-ip".to_string());

        let result = create_node(State(app_state), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Invalid IP")
                || error.to_string().contains("invalid")
                || error.to_string().contains("IP")
        );
    }

    #[tokio::test]
    async fn test_create_node_with_minimal_data() {
        let app_state = setup_test_app_state().await;
        let mut request = create_test_create_request();
        request.domain = None;
        request.management_ip = None;
        request.custom_data = None;

        let result = create_node(State(app_state.clone()), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "test-router");
        assert_eq!(data.node.domain, "");
        assert!(data.node.management_ip.is_none());
        // custom_data may be null when not provided, which is valid
        assert!(
            data.node.custom_data == serde_json::json!({})
                || data.node.custom_data == serde_json::Value::Null
        );
    }
}
