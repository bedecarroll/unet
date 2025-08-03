//! Tests for node deletion operations

#[cfg(test)]
mod tests {
    use super::super::crud::*;
    use crate::api::ApiResponse;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use axum::{
        extract::{Path, State},
        response::Json,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_delete_node_success() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        let result = delete_node(State(app_state.clone()), Path(node.id)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { success, .. }) = result.unwrap();
        assert!(success);

        // Verify the node was actually deleted
        let deleted_node = app_state.datastore.get_node(&node.id).await.unwrap();
        assert!(deleted_node.is_none());
    }

    #[tokio::test]
    async fn test_delete_node_not_found() {
        let app_state = setup_test_app_state().await;
        let non_existent_id = Uuid::new_v4();

        let result = delete_node(State(app_state), Path(non_existent_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }
}
