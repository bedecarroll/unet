//! Link API handlers

use axum::{
    extract::{Path, Query},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::{ApiResponse, CreateLinkRequest, PaginatedResponse, UpdateLinkRequest};
use crate::handlers::{ServerError, ServerResult};
use unet_core::prelude::*;

/// Query parameters for listing links
#[derive(Debug, Deserialize)]
pub struct ListLinksQuery {
    /// Page number (1-based)
    pub page: Option<u64>,
    /// Items per page
    pub per_page: Option<u64>,
    /// Filter by node ID (shows links involving this node)
    pub node_id: Option<Uuid>,
    /// Filter by bandwidth (minimum)
    pub min_bandwidth: Option<u64>,
}

/// List all links with optional filtering and pagination
pub async fn list_links(
    Query(query): Query<ListLinksQuery>,
) -> ServerResult<Json<ApiResponse<PaginatedResponse<Link>>>> {
    // TODO: Implement actual datastore integration
    // For now, return mock data

    let node_a = Uuid::new_v4();
    let node_z = Uuid::new_v4();

    let mock_links = vec![
        Link::new(
            "link-01".to_string(),
            node_a,
            "GigabitEthernet0/0/0".to_string(),
            node_z,
            "GigabitEthernet0/0/1".to_string(),
        ),
        Link::new_internet_circuit(
            "internet-01".to_string(),
            node_a,
            "GigabitEthernet0/1/0".to_string(),
        ),
    ];

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let total = mock_links.len() as u64;

    let paginated = PaginatedResponse {
        data: mock_links,
        total,
        page,
        per_page,
        total_pages: (total + per_page - 1) / per_page,
        has_next: page * per_page < total,
        has_prev: page > 1,
    };

    Ok(Json(ApiResponse::success(paginated)))
}

/// Get a specific link by ID
pub async fn get_link(Path(id): Path<Uuid>) -> ServerResult<Json<ApiResponse<Link>>> {
    // TODO: Implement actual datastore lookup

    let node_a = Uuid::new_v4();
    let node_z = Uuid::new_v4();

    let link = Link::new(
        "link-01".to_string(),
        node_a,
        "GigabitEthernet0/0/0".to_string(),
        node_z,
        "GigabitEthernet0/0/1".to_string(),
    );

    Ok(Json(ApiResponse::success(link)))
}

/// Create a new link
pub async fn create_link(
    Json(request): Json<CreateLinkRequest>,
) -> ServerResult<Json<ApiResponse<Link>>> {
    let link = request
        .to_link()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    // TODO: Save to datastore

    Ok(Json(ApiResponse::success_with_message(
        link,
        "Link created successfully".to_string(),
    )))
}

/// Update an existing link
pub async fn update_link(
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateLinkRequest>,
) -> ServerResult<Json<ApiResponse<Link>>> {
    // TODO: Fetch existing link from datastore and apply updates

    // For demo purposes, create a simple link with required fields
    // In real implementation, we'd fetch the existing link and update it
    let node_a = request.node_a_id.unwrap_or_else(Uuid::new_v4);
    let interface_a = request
        .interface_a
        .unwrap_or_else(|| "updated-interface".to_string());

    let link =
        if let (Some(node_z_id), Some(interface_z)) = (request.node_z_id, request.interface_z) {
            // Regular link between two nodes
            Link::new(
                request.name.unwrap_or_else(|| "updated-link".to_string()),
                node_a,
                interface_a,
                node_z_id,
                interface_z,
            )
        } else {
            // Internet circuit
            Link::new_internet_circuit(
                request
                    .name
                    .unwrap_or_else(|| "updated-internet-circuit".to_string()),
                node_a,
                interface_a,
            )
        };

    Ok(Json(ApiResponse::success_with_message(
        link,
        "Link updated successfully".to_string(),
    )))
}

/// Delete a link
pub async fn delete_link(Path(id): Path<Uuid>) -> ServerResult<Json<ApiResponse<()>>> {
    // TODO: Delete from datastore

    Ok(Json(ApiResponse::success_with_message(
        (),
        format!("Link {} deleted successfully", id),
    )))
}
