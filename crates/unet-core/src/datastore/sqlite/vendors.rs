//! Vendor operations for `SQLite` datastore

use super::super::types::{DataStoreError, DataStoreResult};
use super::SqliteStore;
use crate::entities::vendors;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

/// Creates a vendor record
pub async fn create_vendor(store: &SqliteStore, name: &str) -> DataStoreResult<()> {
    let active = vendors::ActiveModel {
        name: Set(name.to_owned()),
    };
    active
        .insert(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to insert vendor: {e}"),
        })?;
    Ok(())
}

/// Lists all vendor names
pub async fn list_vendors(store: &SqliteStore) -> DataStoreResult<Vec<String>> {
    let items = vendors::Entity::find().all(&store.db).await.map_err(|e| {
        DataStoreError::InternalError {
            message: format!("Failed to query vendors: {e}"),
        }
    })?;
    Ok(items.into_iter().map(|v| v.name).collect())
}

/// Deletes a vendor by name
pub async fn delete_vendor(store: &SqliteStore, name: &str) -> DataStoreResult<()> {
    let result = vendors::Entity::delete_by_id(name)
        .exec(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to delete vendor: {e}"),
        })?;
    if result.rows_affected == 0 {
        return Err(DataStoreError::NotFound {
            entity_type: "Vendor".to_owned(),
            id: name.to_string(),
        });
    }
    Ok(())
}
