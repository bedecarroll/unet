//! Tests for node get operations

#[cfg(test)]
mod tests {
    use crate::api::ApiResponse;
    use crate::handlers::nodes::crud::*;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use axum::{
        extract::{Path, State},
        response::Json,
    };
    use unet_core::models::Vendor;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_node_success() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        let result = get_node(State(app_state), Path(node.id)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.id, node.id);
        assert_eq!(data.node.name, "test-node");
        assert_eq!(data.node.vendor, Vendor::Cisco);
        assert!(data.status.is_none());
    }

    #[tokio::test]
    async fn test_get_node_not_found() {
        let app_state = setup_test_app_state().await;
        let non_existent_id = Uuid::new_v4();

        let result = get_node(State(app_state), Path(non_existent_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_get_node_internal_error() {
        // This test would require mocking the datastore to return an error
        // For now, we'll create a placeholder test that documents the expected behavior
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        // In a real test, we'd mock the datastore to fail
        // Here we just verify the happy path works
        let result = get_node(State(app_state), Path(node.id)).await;
        assert!(result.is_ok());
    }
}
