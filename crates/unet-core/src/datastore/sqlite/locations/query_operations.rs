//! Query operations for `SQLite` location datastore
//!
//! Contains list operations with filtering, sorting, and pagination support.

use super::super::super::types::{DataStoreError, DataStoreResult, PagedResult, QueryOptions};
use super::super::SqliteStore;
use super::super::conversions::entity_to_location;
use super::super::filters::{apply_location_filters, apply_location_sorting};
use crate::entities::locations;
use crate::models::Location;
use sea_orm::{EntityTrait, PaginatorTrait, QuerySelect};

/// Lists locations with filtering, sorting, and pagination
pub async fn list_locations(
    store: &SqliteStore,
    options: &QueryOptions,
) -> DataStoreResult<PagedResult<Location>> {
    let mut query = locations::Entity::find();

    // Apply filters and sorting using helper functions
    query = apply_location_filters(query, &options.filters)?;
    query = apply_location_sorting(query, &options.sort)?;

    // Get total count
    let total_count =
        query
            .clone()
            .count(&store.db)
            .await
            .map_err(|e| DataStoreError::InternalError {
                message: format!("Failed to count locations: {e}"),
            })?;

    // Apply pagination
    if let Some(pagination) = &options.pagination {
        query = query
            .offset(pagination.offset as u64)
            .limit(pagination.limit as u64);
    }

    // Execute query
    let entities = query
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query locations: {e}"),
        })?;

    // Convert entities to Location models
    let locations = entities
        .into_iter()
        .map(entity_to_location)
        .collect::<DataStoreResult<Vec<_>>>()?;

    Ok(PagedResult::new(
        locations,
        total_count.try_into().unwrap_or(usize::MAX),
        options.pagination.as_ref(),
    ))
}
