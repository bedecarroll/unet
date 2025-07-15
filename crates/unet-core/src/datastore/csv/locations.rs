//! Location operations for CSV datastore

use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, FilterValue, PagedResult,
    QueryOptions, SortDirection,
};
use super::CsvStore;
use super::utils::apply_filters;
use crate::models::Location;
use uuid::Uuid;

/// Creates a new location
pub async fn create_location(store: &CsvStore, location: &Location) -> DataStoreResult<Location> {
    location
        .validate()
        .map_err(|e| DataStoreError::ValidationError { message: e })?;

    {
        let mut data = store.data.lock().await;
        if data.locations.contains_key(&location.id) {
            return Err(DataStoreError::ConstraintViolation {
                message: format!("Location with ID {} already exists", location.id),
            });
        }
        data.locations.insert(location.id, location.clone());
    }

    store.save_data().await?;
    Ok(location.clone())
}

/// Gets a location by ID
pub async fn get_location(store: &CsvStore, id: &Uuid) -> DataStoreResult<Option<Location>> {
    let data = store.data.lock().await;
    Ok(data.locations.get(id).cloned())
}

/// Lists locations with filtering, sorting, and pagination
pub async fn list_locations(
    store: &CsvStore,
    options: &QueryOptions,
) -> DataStoreResult<PagedResult<Location>> {
    let mut items: Vec<Location> = {
        let data = store.data.lock().await;
        data.locations.values().cloned().collect()
    };

    // Apply filters
    items = apply_filters(items, &options.filters, |location, field| match field {
        "name" => Some(FilterValue::String(location.name.clone())),
        "location_type" => Some(FilterValue::String(location.location_type.clone())),
        "path" => Some(FilterValue::String(location.path.clone())),
        "parent_id" => location.parent_id.map(FilterValue::Uuid),
        _ => None,
    });

    // Apply sorting
    for sort in &options.sort {
        items.sort_by(|a, b| {
            let result = match sort.field.as_str() {
                "name" => a.name.cmp(&b.name),
                "location_type" => a.location_type.cmp(&b.location_type),
                "path" => a.path.cmp(&b.path),
                _ => std::cmp::Ordering::Equal,
            };
            if matches!(sort.direction, SortDirection::Descending) {
                result.reverse()
            } else {
                result
            }
        });
    }

    // Apply pagination
    let total = items.len();
    let (start, page_limit) = options
        .pagination
        .as_ref()
        .map_or((0, total), |pagination| {
            (pagination.offset, pagination.limit)
        });
    let paginated_items = items.into_iter().skip(start).take(page_limit).collect();

    Ok(PagedResult::new(
        paginated_items,
        total,
        options.pagination.as_ref(),
    ))
}

/// Updates an existing location
pub async fn update_location(store: &CsvStore, location: &Location) -> DataStoreResult<Location> {
    location
        .validate()
        .map_err(|e| DataStoreError::ValidationError { message: e })?;

    {
        let mut data = store.data.lock().await;
        if !data.locations.contains_key(&location.id) {
            return Err(DataStoreError::NotFound {
                entity_type: "location".to_string(),
                id: location.id.to_string(),
            });
        }
        data.locations.insert(location.id, location.clone());
    }

    store.save_data().await?;
    Ok(location.clone())
}

/// Deletes a location by ID
pub async fn delete_location(store: &CsvStore, id: &Uuid) -> DataStoreResult<()> {
    {
        let mut data = store.data.lock().await;
        if data.locations.remove(id).is_none() {
            return Err(DataStoreError::NotFound {
                entity_type: "location".to_string(),
                id: id.to_string(),
            });
        }
    }

    store.save_data().await?;
    Ok(())
}

/// Performs batch operations on locations
pub async fn batch_locations(
    store: &CsvStore,
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
