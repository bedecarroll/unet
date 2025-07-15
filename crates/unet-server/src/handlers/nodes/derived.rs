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
