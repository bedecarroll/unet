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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datastore::sqlite::tests::setup::setup_test_db;
    use crate::datastore::types::{
        Filter, FilterOperation, FilterValue, Pagination, Sort, SortDirection,
    };
    use crate::models::location::LocationBuilder;
    use serde_json::json;

    fn create_test_location() -> Location {
        LocationBuilder::new()
            .name("Test Location".to_string())
            .location_type("datacenter".to_string())
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_location_success() {
        let test_db = setup_test_db().await;
        let location = create_test_location();

        let result = create_location(&test_db.store, &location).await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.name, location.name);
        assert_eq!(created.location_type, location.location_type);
    }

    #[tokio::test]
    async fn test_get_location_existing() {
        let test_db = setup_test_db().await;
        let location = create_test_location();
        let _created = create_location(&test_db.store, &location).await.unwrap();

        let result = get_location(&test_db.store, &location.id).await;
        assert!(result.is_ok());

        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, location.id);
    }

    #[tokio::test]
    async fn test_get_location_not_found() {
        let test_db = setup_test_db().await;
        let non_existent_id = Uuid::new_v4();

        let result = get_location(&test_db.store, &non_existent_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_locations_empty() {
        let test_db = setup_test_db().await;
        let options = QueryOptions::default();

        let result = list_locations(&test_db.store, &options).await;
        assert!(result.is_ok());

        let paged_result = result.unwrap();
        assert_eq!(paged_result.items.len(), 0);
        assert_eq!(paged_result.total_count, 0);
    }

    #[tokio::test]
    async fn test_list_locations_with_data() {
        let test_db = setup_test_db().await;
        let location1 = create_test_location();
        let location2 = {
            let mut loc = create_test_location();
            loc.name = "Second Location".to_string();
            loc
        };

        let _created1 = create_location(&test_db.store, &location1).await.unwrap();
        let _created2 = create_location(&test_db.store, &location2).await.unwrap();

        let options = QueryOptions::default();
        let result = list_locations(&test_db.store, &options).await;
        assert!(result.is_ok());

        let paged_result = result.unwrap();
        assert_eq!(paged_result.items.len(), 2);
        assert_eq!(paged_result.total_count, 2);
    }

    #[tokio::test]
    async fn test_list_locations_with_pagination() {
        let test_db = setup_test_db().await;

        // Create multiple locations
        for i in 0..5 {
            let mut location = create_test_location();
            location.name = format!("Location {i}");
            let _created = create_location(&test_db.store, &location).await.unwrap();
        }

        let options = QueryOptions {
            pagination: Some(Pagination {
                offset: 1,
                limit: 2,
            }),
            ..Default::default()
        };

        let result = list_locations(&test_db.store, &options).await;
        assert!(result.is_ok());

        let paged_result = result.unwrap();
        assert_eq!(paged_result.items.len(), 2);
        assert_eq!(paged_result.total_count, 5);
    }

    #[tokio::test]
    async fn test_list_locations_with_filters() {
        let test_db = setup_test_db().await;
        let mut location1 = create_test_location();
        location1.name = "Datacenter Alpha".to_string();
        let mut location2 = create_test_location();
        location2.name = "Office Beta".to_string();

        let _created1 = create_location(&test_db.store, &location1).await.unwrap();
        let _created2 = create_location(&test_db.store, &location2).await.unwrap();

        let options = QueryOptions {
            filters: vec![Filter {
                field: "name".to_string(),
                operation: FilterOperation::Contains,
                value: FilterValue::String("Datacenter".to_string()),
            }],
            ..Default::default()
        };

        let result = list_locations(&test_db.store, &options).await;
        assert!(result.is_ok());

        let paged_result = result.unwrap();
        assert_eq!(paged_result.items.len(), 1);
        assert_eq!(paged_result.items[0].name, "Datacenter Alpha");
    }

    #[tokio::test]
    async fn test_list_locations_with_sorting() {
        let test_db = setup_test_db().await;
        let mut location1 = create_test_location();
        location1.name = "Z Location".to_string();
        let mut location2 = create_test_location();
        location2.name = "A Location".to_string();

        let _created1 = create_location(&test_db.store, &location1).await.unwrap();
        let _created2 = create_location(&test_db.store, &location2).await.unwrap();

        let options = QueryOptions {
            sort: vec![Sort {
                field: "name".to_string(),
                direction: SortDirection::Ascending,
            }],
            ..Default::default()
        };

        let result = list_locations(&test_db.store, &options).await;
        assert!(result.is_ok());

        let paged_result = result.unwrap();
        assert_eq!(paged_result.items.len(), 2);
        assert_eq!(paged_result.items[0].name, "A Location");
        assert_eq!(paged_result.items[1].name, "Z Location");
    }

    #[tokio::test]
    async fn test_update_location_success() {
        let test_db = setup_test_db().await;
        let mut location = create_test_location();
        let _created = create_location(&test_db.store, &location).await.unwrap();

        // Update the location
        location.name = "Updated Location".to_string();
        location.description = Some("Updated description".to_string());

        let result = update_location(&test_db.store, &location).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "Updated Location");
        assert_eq!(updated.description, Some("Updated description".to_string()));
    }

    #[tokio::test]
    async fn test_update_location_not_found() {
        let test_db = setup_test_db().await;
        let location = create_test_location(); // Not created in store

        let result = update_location(&test_db.store, &location).await;
        assert!(result.is_err());

        if let Err(DataStoreError::NotFound { entity_type, .. }) = result {
            assert_eq!(entity_type, "Location");
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_delete_location_success() {
        let test_db = setup_test_db().await;
        let location = create_test_location();
        let _created = create_location(&test_db.store, &location).await.unwrap();

        let result = delete_location(&test_db.store, &location.id).await;
        assert!(result.is_ok());

        // Verify deletion
        let found = get_location(&test_db.store, &location.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_location_not_found() {
        let test_db = setup_test_db().await;
        let non_existent_id = Uuid::new_v4();

        let result = delete_location(&test_db.store, &non_existent_id).await;
        assert!(result.is_err());

        if let Err(DataStoreError::NotFound { entity_type, .. }) = result {
            assert_eq!(entity_type, "Location");
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_batch_locations_all_success() {
        let test_db = setup_test_db().await;

        let location1 = create_test_location();
        let location2 = {
            let mut loc = create_test_location();
            loc.name = "Second Location".to_string();
            loc
        };

        let operations = vec![
            BatchOperation::Insert(location1.clone()),
            BatchOperation::Insert(location2.clone()),
        ];

        let result = batch_locations(&test_db.store, &operations).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.success_count, 2);
        assert_eq!(batch_result.error_count, 0);
        assert!(batch_result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_batch_locations_mixed_results() {
        let test_db = setup_test_db().await;

        let location1 = create_test_location();
        let mut location2 = create_test_location();
        location2.id = location1.id; // Same ID as location1 - will cause conflict

        let operations = vec![
            BatchOperation::Insert(location1.clone()),
            BatchOperation::Insert(location2.clone()), // Should fail due to duplicate ID
        ];

        let result = batch_locations(&test_db.store, &operations).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.success_count, 1);
        assert_eq!(batch_result.error_count, 1);
        assert_eq!(batch_result.errors.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_locations_update_operations() {
        let test_db = setup_test_db().await;

        let mut location = create_test_location();
        let _created = create_location(&test_db.store, &location).await.unwrap();

        // Update via batch operation
        location.name = "Batch Updated".to_string();

        let operations = vec![BatchOperation::Update(location.clone())];

        let result = batch_locations(&test_db.store, &operations).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.success_count, 1);
        assert_eq!(batch_result.error_count, 0);

        // Verify update
        let updated = get_location(&test_db.store, &location.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "Batch Updated");
    }

    #[tokio::test]
    async fn test_batch_locations_delete_operations() {
        let test_db = setup_test_db().await;

        let location = create_test_location();
        let _created = create_location(&test_db.store, &location).await.unwrap();

        let operations = vec![BatchOperation::Delete(location.id)];

        let result = batch_locations(&test_db.store, &operations).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.success_count, 1);
        assert_eq!(batch_result.error_count, 0);

        // Verify deletion
        let found = get_location(&test_db.store, &location.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_location_with_custom_data() {
        let test_db = setup_test_db().await;
        let mut location = create_test_location();
        location.custom_data = json!({
            "capacity": 100,
            "cooling": "air",
            "security_level": "high"
        });

        let created = create_location(&test_db.store, &location).await.unwrap();
        assert_eq!(created.custom_data, location.custom_data);

        // Verify through get_location as well
        let retrieved = get_location(&test_db.store, &location.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.custom_data, location.custom_data);
    }

    #[tokio::test]
    async fn test_location_with_parent_relationship() {
        let test_db = setup_test_db().await;

        // Create parent location
        let parent = create_test_location();
        let _parent_created = create_location(&test_db.store, &parent).await.unwrap();

        // Create child location
        let mut child = create_test_location();
        child.name = "Child Location".to_string();
        child.parent_id = Some(parent.id);
        child.path = format!("{}/{}", parent.path, child.name);

        let child_created = create_location(&test_db.store, &child).await.unwrap();
        assert_eq!(child_created.parent_id, Some(parent.id));
        assert!(child_created.path.contains(&parent.name));
    }
}
