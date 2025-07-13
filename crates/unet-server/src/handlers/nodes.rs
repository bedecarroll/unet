//! Node API handlers

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::{
    ApiResponse, CreateNodeRequest, NodeResponse, PaginatedResponse, UpdateNodeRequest,
};
use crate::handlers::{ServerError, ServerResult};
use crate::server::AppState;
use unet_core::models::derived::{InterfaceStatus, NodeStatus, PerformanceMetrics};
use unet_core::prelude::*;

/// Query parameters for listing nodes
#[derive(Debug, Deserialize)]
pub struct ListNodesQuery {
    /// Page number (1-based)
    pub page: Option<u64>,
    /// Items per page
    pub per_page: Option<u64>,
    /// Filter by lifecycle
    pub lifecycle: Option<String>,
    /// Filter by role
    pub role: Option<String>,
    /// Filter by vendor
    pub vendor: Option<String>,
    /// Include derived state in response
    pub include_status: Option<bool>,
}

/// List all nodes with optional filtering and pagination
pub async fn list_nodes(
    State(app_state): State<AppState>,
    Query(query): Query<ListNodesQuery>,
) -> ServerResult<Json<ApiResponse<PaginatedResponse<NodeResponse>>>> {
    let mut filters = Vec::new();

    if let Some(lifecycle) = query.lifecycle {
        filters.push(Filter {
            field: "lifecycle".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(lifecycle),
        });
    }

    if let Some(role) = query.role {
        filters.push(Filter {
            field: "role".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(role),
        });
    }

    if let Some(vendor) = query.vendor {
        filters.push(Filter {
            field: "vendor".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(vendor),
        });
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let options = QueryOptions {
        filters,
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }],
        pagination: Some(Pagination {
            offset: usize::try_from((page - 1) * per_page).unwrap_or(0),
            limit: usize::try_from(per_page).unwrap_or(20),
        }),
    };

    let result = app_state
        .datastore
        .list_nodes(&options)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // Convert to NodeResponse with optional status
    let include_status = query.include_status.unwrap_or(false);
    let node_responses: Vec<NodeResponse> = result
        .items
        .into_iter()
        .map(|node| {
            if include_status {
                // TODO: Fetch actual status from datastore - for now just use None
            }
            NodeResponse::from_node(node)
        })
        .collect();

    let paginated = PaginatedResponse {
        data: node_responses,
        total: result.total_count as u64,
        page,
        per_page,
        total_pages: result.total_pages as u64,
        has_next: result.has_next,
        has_prev: result.has_previous,
    };

    Ok(Json(ApiResponse::success(paginated)))
}

/// Get a specific node by ID
pub async fn get_node(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<NodeResponse>>> {
    let node = app_state
        .datastore
        .get_node_required(&id)
        .await
        .map_err(|e| match e {
            unet_core::datastore::DataStoreError::NotFound { .. } => {
                ServerError::NotFound(format!("Node with ID {id} not found"))
            }
            _ => ServerError::Internal(e.to_string()),
        })?;

    let response = NodeResponse::from_node(node);
    Ok(Json(ApiResponse::success(response)))
}

/// Create a new node
pub async fn create_node(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNodeRequest>,
) -> ServerResult<Json<ApiResponse<NodeResponse>>> {
    // Use the existing into_node method
    let node = payload
        .into_node()
        .map_err(|e| ServerError::BadRequest(format!("Node validation failed: {e}")))?;

    let created_node = app_state
        .datastore
        .create_node(&node)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    let response = NodeResponse::from_node(created_node);
    Ok(Json(ApiResponse::success(response)))
}

/// Update an existing node
pub async fn update_node(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNodeRequest>,
) -> ServerResult<Json<ApiResponse<NodeResponse>>> {
    let mut node = app_state
        .datastore
        .get_node_required(&id)
        .await
        .map_err(|e| match e {
            unet_core::datastore::DataStoreError::NotFound { .. } => {
                ServerError::NotFound(format!("Node with ID {id} not found"))
            }
            _ => ServerError::Internal(e.to_string()),
        })?;

    // Update fields that were provided
    if let Some(name) = payload.name {
        node.name = name;
    }

    if let Some(domain) = payload.domain {
        node.domain = domain;
    }

    if let Some(vendor) = payload.vendor {
        node.vendor = vendor;
    }

    if let Some(model) = payload.model {
        node.model = model;
    }

    if let Some(role) = payload.role {
        node.role = role;
    }

    if let Some(lifecycle) = payload.lifecycle {
        node.lifecycle = lifecycle;
    }

    if let Some(location_id) = payload.location_id {
        node.location_id = Some(location_id);
    }

    if let Some(management_ip_str) = payload.management_ip {
        let management_ip = management_ip_str
            .parse()
            .map_err(|e| ServerError::BadRequest(format!("Invalid management IP: {e}")))?;
        node.management_ip = Some(management_ip);
    }

    if let Some(custom_data) = payload.custom_data {
        node.custom_data = custom_data;
    }

    let updated_node = app_state
        .datastore
        .update_node(&node)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    let response = NodeResponse::from_node(updated_node);
    Ok(Json(ApiResponse::success(response)))
}

/// Delete a node
pub async fn delete_node(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<()>>> {
    app_state
        .datastore
        .delete_node(&id)
        .await
        .map_err(|e| match e {
            unet_core::datastore::DataStoreError::NotFound { .. } => {
                ServerError::NotFound(format!("Node with ID {id} not found"))
            }
            _ => ServerError::Internal(e.to_string()),
        })?;

    Ok(Json(ApiResponse::success(())))
}

/// Get node status (derived state)
pub async fn get_node_status(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<NodeStatus>>> {
    // First verify the node exists
    let _node = app_state
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
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .unwrap_or_else(|| NodeStatus::new(id));

    Ok(Json(ApiResponse::success(status)))
}

/// Get node interfaces (derived state)
pub async fn get_node_interfaces(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<Vec<InterfaceStatus>>>> {
    // First verify the node exists
    let _node = app_state
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
    let interfaces = app_state
        .datastore
        .get_node_interfaces(&id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(interfaces)))
}

/// Get node performance metrics (derived state)
pub async fn get_node_metrics(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<ApiResponse<PerformanceMetrics>>> {
    // First verify the node exists
    let _node = app_state
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
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .ok_or_else(|| ServerError::NotFound(format!("No metrics available for node {id}")))?;

    Ok(Json(ApiResponse::success(metrics)))
}
