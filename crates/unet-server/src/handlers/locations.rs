//! Location API handlers

use axum::{
    extract::{Path, Query},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::{ApiResponse, CreateLocationRequest, PaginatedResponse, UpdateLocationRequest};
use crate::handlers::{ServerError, ServerResult};
use unet_core::prelude::*;

/// Query parameters for listing locations
#[derive(Debug, Deserialize)]
pub struct ListLocationsQuery {
    /// Page number (1-based)
    pub page: Option<u64>,
    /// Items per page
    pub per_page: Option<u64>,
}

/// List all locations with optional filtering and pagination
pub async fn list_locations(
    Query(query): Query<ListLocationsQuery>,
) -> ServerResult<Json<ApiResponse<PaginatedResponse<Location>>>> {
    // TODO: Implement actual datastore integration
    // For now, return mock data

    let mock_locations = vec![
        Location::new_root("headquarters".to_string(), "building".to_string()),
        Location::new_root("branch-office".to_string(), "building".to_string()),
    ];

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let total = mock_locations.len() as u64;

    let paginated = PaginatedResponse {
        data: mock_locations,
        total,
        page,
        per_page,
        total_pages: total.div_ceil(per_page),
        has_next: page * per_page < total,
        has_prev: page > 1,
    };

    Ok(Json(ApiResponse::success(paginated)))
}

/// Get a specific location by ID
pub async fn get_location(Path(_id): Path<Uuid>) -> ServerResult<Json<ApiResponse<Location>>> {
    // TODO: Implement actual datastore lookup

    let location = Location::new_root("headquarters".to_string(), "building".to_string());
    Ok(Json(ApiResponse::success(location)))
}

/// Create a new location
pub async fn create_location(
    Json(request): Json<CreateLocationRequest>,
) -> ServerResult<Json<ApiResponse<Location>>> {
    let location = request
        .into_location()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    // TODO: Save to datastore

    Ok(Json(ApiResponse::success_with_message(
        location,
        "Location created successfully".to_string(),
    )))
}

/// Update an existing location
pub async fn update_location(
    Path(_id): Path<Uuid>,
    Json(request): Json<UpdateLocationRequest>,
) -> ServerResult<Json<ApiResponse<Location>>> {
    // TODO: Fetch existing location from datastore and apply updates

    let location = Location::new_root(
        request
            .name
            .unwrap_or_else(|| "updated-location".to_string()),
        request
            .location_type
            .unwrap_or_else(|| "building".to_string()),
    );

    Ok(Json(ApiResponse::success_with_message(
        location,
        "Location updated successfully".to_string(),
    )))
}

/// Delete a location
pub async fn delete_location(Path(id): Path<Uuid>) -> ServerResult<Json<ApiResponse<()>>> {
    // TODO: Delete from datastore

    Ok(Json(ApiResponse::success_with_message(
        (),
        format!("Location {id} deleted successfully"),
    )))
}
