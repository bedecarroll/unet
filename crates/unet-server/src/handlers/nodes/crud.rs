//! CRUD operations for node management

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use uuid::Uuid;

use crate::api::{
    ApiResponse, CreateNodeRequest, NodeResponse, PaginatedResponse, UpdateNodeRequest,
};
use crate::handlers::{ServerError, ServerResult};
use crate::server::AppState;
use unet_core::prelude::*;

use super::types::ListNodesQuery;

/// List all nodes with optional filtering and pagination
///
/// # Errors
/// Returns an error if datastore operations fail.
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

    let result = app_state.datastore.list_nodes(&options).await?;

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
///
/// # Errors
/// Returns an error if the node does not exist or datastore operations fail.
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
///
/// # Errors
/// Returns an error if validation fails or datastore operations fail.
pub async fn create_node(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNodeRequest>,
) -> ServerResult<Json<ApiResponse<NodeResponse>>> {
    // Use the existing into_node method
    let node = payload
        .into_node()
        .map_err(|e| ServerError::BadRequest(format!("Node validation failed: {e}")))?;

    let created_node = app_state.datastore.create_node(&node).await?;

    let response = NodeResponse::from_node(created_node);
    Ok(Json(ApiResponse::success(response)))
}

/// Update an existing node
///
/// # Errors
/// Returns an error if the node is not found, input is invalid, or datastore operations fail.
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
    let mut fqdn_needs_update = false;

    if let Some(name) = payload.name {
        node.name = name;
        fqdn_needs_update = true;
    }

    if let Some(domain) = payload.domain {
        node.domain = domain;
        fqdn_needs_update = true;
    }

    if fqdn_needs_update {
        node.update_fqdn();
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

    let updated_node = app_state.datastore.update_node(&node).await?;

    let response = NodeResponse::from_node(updated_node);
    Ok(Json(ApiResponse::success(response)))
}

/// Delete a node
///
/// # Errors
/// Returns an error if the node is not found or datastore operations fail.
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
