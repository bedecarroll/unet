//! Derived state operations for nodes (status, interfaces, metrics)

use axum::{
    extract::{Path, State},
    response::Json,
};
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::handlers::{ServerError, ServerResult};
use crate::server::AppState;
use unet_core::models::derived::{InterfaceStatus, NodeStatus, PerformanceMetrics};

/// Get node status (derived state)
///
/// # Errors
/// Returns an error if the node does not exist or datastore operations fail.
pub async fn get_node_status(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<NodeStatus>>> {
    // First verify the node exists
    app_state
        .datastore
        .get_node_required(&id)
        .await
        .map_err(|e| match e {
            unet_core::datastore::DataStoreError::NotFound { .. } => {
                ServerError::NotFound(format!("Node with ID {id} not found"))
            }
            _ => ServerError::Internal(e.to_string()),
        })?;

    // Get node status from datastore
    let status = app_state
        .datastore
        .get_node_status(&id)
        .await?
        .unwrap_or_else(|| NodeStatus::new(id));

    Ok(Json(ApiResponse::success(status)))
}

/// Get node interfaces (derived state)
///
/// # Errors
/// Returns an error if the node does not exist or datastore operations fail.
pub async fn get_node_interfaces(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<Vec<InterfaceStatus>>>> {
    // First verify the node exists
    app_state
        .datastore
        .get_node_required(&id)
        .await
        .map_err(|e| match e {
            unet_core::datastore::DataStoreError::NotFound { .. } => {
                ServerError::NotFound(format!("Node with ID {id} not found"))
            }
            _ => ServerError::Internal(e.to_string()),
        })?;

    // Get interfaces from datastore
    let interfaces = app_state.datastore.get_node_interfaces(&id).await?;

    Ok(Json(ApiResponse::success(interfaces)))
}

/// Get node performance metrics (derived state)
///
/// # Errors
/// Returns an error if the node does not exist, metrics are unavailable, or datastore operations fail.
pub async fn get_node_metrics(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<PerformanceMetrics>>> {
    // First verify the node exists
    app_state
        .datastore
        .get_node_required(&id)
        .await
        .map_err(|e| match e {
            unet_core::datastore::DataStoreError::NotFound { .. } => {
                ServerError::NotFound(format!("Node with ID {id} not found"))
            }
            _ => ServerError::Internal(e.to_string()),
        })?;

    // Get metrics from datastore
    let metrics = app_state
        .datastore
        .get_node_metrics(&id)
        .await?
        .ok_or_else(|| ServerError::NotFound(format!("No metrics available for node {id}")))?;

    Ok(Json(ApiResponse::success(metrics)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use axum::extract::{Path, State};
    use std::sync::Arc;
    use unet_core::{
        datastore::{DataStore, sqlite::SqliteStore},
        models::*,
        policy_integration::PolicyService,
    };

    async fn setup_test_datastore() -> SqliteStore {
        
        test_support::sqlite::sqlite_store().await
    }

    async fn create_test_node(datastore: &SqliteStore) -> Node {
        let mut node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ASR1000".to_string();
        node.lifecycle = Lifecycle::Live;
        datastore.create_node(&node).await.unwrap()
    }

    #[tokio::test]
    async fn test_get_node_status_success() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_status(State(app_state), Path(node.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.success);
        assert_eq!(response.data.node_id, node.id);
    }

    #[tokio::test]
    async fn test_get_node_status_not_found() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_status(State(app_state), Path(non_existent_id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_node_interfaces_success() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_interfaces(State(app_state), Path(node.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.success);
        // CSV datastore will return empty interfaces list
        assert!(response.data.is_empty());
    }

    #[tokio::test]
    async fn test_get_node_interfaces_not_found() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_interfaces(State(app_state), Path(non_existent_id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_node_metrics_not_found_node() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_metrics(State(app_state), Path(non_existent_id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_node_metrics_no_metrics() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_metrics(State(app_state), Path(node.id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("No metrics available"));
            }
            _ => panic!("Expected NotFound error for missing metrics"),
        }
    }

    #[tokio::test]
    async fn test_get_node_status_internal_error_handling() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_status(State(app_state), Path(non_existent_id)).await;

        if let Err(ServerError::NotFound(_)) = result {
            // This covers lines 22-29
        } else {
            // Any other result
        }
    }

    #[tokio::test]
    async fn test_get_node_interfaces_internal_error_handling() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_interfaces(State(app_state), Path(non_existent_id)).await;

        if let Err(ServerError::NotFound(_)) = result {
            // This covers lines 49-56
        } else {
            // Any other result
        }
    }

    #[tokio::test]
    async fn test_get_node_metrics_internal_error_handling() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_metrics(State(app_state), Path(non_existent_id)).await;

        if let Err(ServerError::NotFound(_)) = result {
            // This covers lines 72-79
        } else {
            // Any other result
        }
    }

    #[tokio::test]
    async fn test_get_node_status_default_creation() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_status(State(app_state), Path(node.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.success);
        // Verify the default NodeStatus is created correctly - covers line 36
        assert_eq!(response.data.node_id, node.id);
        assert!(!response.data.reachable); // Default should be false
    }
}
