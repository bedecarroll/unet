//! Tests for node update operations

#[cfg(test)]
mod tests {
    use super::super::crud::*;
    use crate::api::ApiResponse;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use axum::{
        extract::{Path, State},
        response::Json,
    };
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

        // Verify the update was persisted
        let updated_node = app_state.datastore.get_node(&node.id).await.unwrap();
        assert!(updated_node.is_some());
        let updated_node = updated_node.unwrap();
        assert_eq!(updated_node.name, "updated-router");
        assert_eq!(updated_node.vendor, Vendor::Juniper);
    }

    #[tokio::test]
    async fn test_update_node_partial() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let mut request = create_test_update_request();
        request.name = Some("partially-updated".to_string());
        request.vendor = None; // Don't update vendor

        let result = update_node(State(app_state.clone()), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "partially-updated");
        assert_eq!(data.node.vendor, Vendor::Cisco); // Should remain unchanged
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
        let mut request = create_test_update_request();
        request.management_ip = Some("invalid-ip".to_string());

        let result = update_node(State(app_state), Path(node.id), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Invalid IP")
                || error.to_string().contains("invalid")
                || error.to_string().contains("IP")
        );
    }

    #[tokio::test]
    async fn test_update_node_fqdn_update() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let mut request = create_test_update_request();
        request.name = Some("updated-hostname".to_string());
        request.domain = Some("new-domain.com".to_string());

        let result = update_node(State(app_state.clone()), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "updated-hostname");
        assert_eq!(data.node.domain, "new-domain.com");
        assert_eq!(data.node.fqdn, "updated-hostname.new-domain.com");
    }

    #[tokio::test]
    async fn test_update_node_with_location_id() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        // Create a valid location first
        let location = create_test_location(&app_state).await;
        let mut request = create_test_update_request();
        request.location_id = Some(location.id);

        let result = update_node(State(app_state), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.location_id, Some(location.id));
    }
}
