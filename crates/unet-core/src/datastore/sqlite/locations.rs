//! Location operations for `SQLite` datastore

use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, PagedResult, QueryOptions,
};
use super::SqliteStore;
use super::conversions::entity_to_location;
use super::filters::{apply_location_filters, apply_location_sorting};
use crate::entities::locations;
use crate::models::Location;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QuerySelect, Set};
use uuid::Uuid;

/// Creates a new location
pub async fn create_location(
    store: &SqliteStore,
    location: &Location,
) -> DataStoreResult<Location> {
    let active_location = locations::ActiveModel {
        id: Set(location.id.to_string()),
        name: Set(location.name.clone()),
        location_type: Set(location.location_type.clone()),
        path: Set(location.path.clone()),
        parent_id: Set(location.parent_id.map(|id| id.to_string())),
        description: Set(location.description.clone()),
        address: Set(location.address.clone()),
        coordinates: Set(None), // Not in Location model yet
        custom_data: Set(Some(
            serde_json::to_string(&location.custom_data).unwrap_or_default(),
        )),
        created_at: Set(Utc::now().to_rfc3339()),
        updated_at: Set(Utc::now().to_rfc3339()),
    };

    active_location
        .insert(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to create location: {e}"),
        })?;

    // Convert back to Location model
    get_location(store, &location.id)
        .await?
        .ok_or_else(|| DataStoreError::NotFound {
            entity_type: "Location".to_string(),
            id: location.id.to_string(),
        })
}

/// Gets a location by ID
pub async fn get_location(store: &SqliteStore, id: &Uuid) -> DataStoreResult<Option<Location>> {
    let entity = locations::Entity::find_by_id(id.to_string())
        .one(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query location: {e}"),
        })?;

    match entity {
        Some(e) => Ok(Some(entity_to_location(e)?)),
        None => Ok(None),
    }
}

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

/// Updates an existing location
pub async fn update_location(
    store: &SqliteStore,
    location: &Location,
) -> DataStoreResult<Location> {
    let active_location = locations::ActiveModel {
        id: Set(location.id.to_string()),
        name: Set(location.name.clone()),
        location_type: Set(location.location_type.clone()),
        path: Set(location.path.clone()),
        parent_id: Set(location.parent_id.map(|id| id.to_string())),
        description: Set(location.description.clone()),
        address: Set(location.address.clone()),
        coordinates: Set(None), // Not in Location model yet
        custom_data: Set(Some(
            serde_json::to_string(&location.custom_data).unwrap_or_default(),
        )),
        created_at: Set(Utc::now().to_rfc3339()),
        updated_at: Set(Utc::now().to_rfc3339()),
    };

    active_location
        .update(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to update location: {e}"),
        })?;

    // Convert back to Location model
    get_location(store, &location.id)
        .await?
        .ok_or_else(|| DataStoreError::NotFound {
            entity_type: "Location".to_string(),
            id: location.id.to_string(),
        })
}

/// Deletes a location by ID
pub async fn delete_location(store: &SqliteStore, id: &Uuid) -> DataStoreResult<()> {
    let result = locations::Entity::delete_by_id(id.to_string())
        .exec(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to delete location: {e}"),
        })?;

    if result.rows_affected == 0 {
        return Err(DataStoreError::NotFound {
            entity_type: "Location".to_string(),
            id: id.to_string(),
        });
    }

    Ok(())
}

/// Performs batch operations on locations
pub async fn batch_locations(
    store: &SqliteStore,
    operations: &[BatchOperation<Location>],
) -> DataStoreResult<BatchResult> {
    let mut success_count = 0;
    let mut errors = Vec::new();

    for (index, operation) in operations.iter().enumerate() {
        let result = match operation {
            BatchOperation::Insert(location) => create_location(store, location).await.map(|_| ()),
            BatchOperation::Update(location) => update_location(store, location).await.map(|_| ()),
            BatchOperation::Delete(id) => delete_location(store, id).await,
        };

        match result {
            Ok(()) => success_count += 1,
            Err(e) => errors.push((index, e)),
        }
    }

    Ok(BatchResult {
        success_count,
        error_count: errors.len(),
        errors,
    })
}
