//! Tests for node delete operations

#[cfg(test)]
mod tests {
    use crate::api::ApiResponse;
    use crate::handlers::nodes::crud::*;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use crate::server::AppState;
    use axum::{
        extract::{Path, State},
        response::Json,
    };
    use std::sync::Arc;
    use unet_core::{
        config::Config,
        datastore::{DataStoreError, MockDataStore},
        policy_integration::PolicyService,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_delete_node_success() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        let result = delete_node(State(app_state.clone()), Path(node.id)).await;

        assert!(result.is_ok());
        let Json(ApiResponse {
            data: (), success, ..
        }) = result.unwrap();
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

    #[tokio::test]
    async fn test_delete_node_internal_error() {
        let mut mock_datastore = MockDataStore::new();
        mock_datastore.expect_delete_node().returning(|_| {
            Box::pin(async move {
                Err(DataStoreError::TransactionError {
                    message: "Transaction failed".to_string(),
                })
            })
        });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::new(Config::default().git),
        };

        let node_id = Uuid::new_v4();
        let result = delete_node(State(app_state), Path(node_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Transaction failed"));
    }
}
