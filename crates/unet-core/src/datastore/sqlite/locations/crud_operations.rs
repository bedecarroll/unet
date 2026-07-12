//! Basic CRUD operations for `SQLite` location datastore
//!
//! Contains create, read, update, and delete operations for location entities.

use super::super::super::types::{DataStoreError, DataStoreResult};
use super::super::SqliteStore;
use super::super::conversions::entity_to_location;
use crate::entities::locations;
use crate::models::Location;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
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

    active_location.update(&store.db).await.map_err(|e| {
        let error_msg = e.to_string();
        if error_msg.contains("None of the records are updated") {
            DataStoreError::NotFound {
                entity_type: "Location".to_string(),
                id: location.id.to_string(),
            }
        } else {
            DataStoreError::InternalError {
                message: format!("Failed to update location: {e}"),
            }
        }
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
