//! Tests for CSV locations operations

use super::super::locations::*;
use super::super::store::CsvStore;
use crate::datastore::types::{
    BatchOperation, DataStoreError, Filter, FilterOperation, FilterValue, Pagination, QueryOptions,
    Sort, SortDirection,
};
use crate::models::Location;
use tempfile::TempDir;
use uuid::Uuid;

async fn setup_test_store() -> (CsvStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let store = CsvStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    (store, temp_dir)
}

fn create_test_location_with_name(name: &str) -> Location {
    Location::new_root(name.to_string(), "Building".to_string())
}

fn create_test_location_with_parent(name: &str, parent_id: Option<Uuid>) -> Location {
    parent_id.map_or_else(
        || Location::new_root(name.to_string(), "Building".to_string()),
        |pid| {
            let mut location =
                Location::new_child(name.to_string(), "Building".to_string(), "parent/path");
            location.parent_id = Some(pid);
            location
        },
    )
}

#[tokio::test]
async fn test_create_location_success() {
    let (store, _temp_dir) = setup_test_store().await;
    let location = create_test_location_with_name("Test Location");

    let result = create_location(&store, &location).await;
    assert!(result.is_ok());

    let created_location = result.unwrap();
    assert_eq!(created_location.name, "Test Location");
    assert_eq!(created_location.id, location.id);
}

#[tokio::test]
async fn test_create_location_duplicate_id() {
    let (store, _temp_dir) = setup_test_store().await;
    let location = create_test_location_with_name("Test Location");

    // Create first location
    let result1 = create_location(&store, &location).await;
    assert!(result1.is_ok());

    // Try to create location with same ID
    let result2 = create_location(&store, &location).await;
    assert!(result2.is_err());

    match result2.unwrap_err() {
        DataStoreError::ConstraintViolation { message } => {
            assert!(message.contains("already exists"));
        }
        _ => panic!("Expected ConstraintViolation"),
    }
}

#[tokio::test]
async fn test_get_location_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let location = create_test_location_with_name("Test Location");

    // Create location first
    create_location(&store, &location).await.unwrap();

    // Get location
    let result = get_location(&store, &location.id).await;
    assert!(result.is_ok());

    let found_location = result.unwrap();
    assert!(found_location.is_some());
    assert_eq!(found_location.unwrap().name, "Test Location");
}

#[tokio::test]
async fn test_get_location_not_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let non_existent_id = Uuid::new_v4();

    let result = get_location(&store, &non_existent_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_list_locations_empty() {
    let (store, _temp_dir) = setup_test_store().await;
    let options = QueryOptions::default();

    let result = list_locations(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 0);
    assert_eq!(paged_result.total_count, 0);
}

#[tokio::test]
async fn test_list_locations_multiple() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create multiple locations
    let location1 = create_test_location_with_name("Location 1");
    let location2 = create_test_location_with_name("Location 2");
    let location3 = create_test_location_with_name("Location 3");

    create_location(&store, &location1).await.unwrap();
    create_location(&store, &location2).await.unwrap();
    create_location(&store, &location3).await.unwrap();

    let options = QueryOptions::default();
    let result = list_locations(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.total_count, 3);
}

#[tokio::test]
async fn test_list_locations_with_pagination() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create multiple locations
    for i in 1..=5 {
        let location = create_test_location_with_name(&format!("Location {i}"));
        create_location(&store, &location).await.unwrap();
    }

    let options = QueryOptions {
        pagination: Some(Pagination {
            offset: 1,
            limit: 2,
        }),
        ..Default::default()
    };

    let result = list_locations(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2);
    assert_eq!(paged_result.total_count, 5);
}

#[tokio::test]
async fn test_list_locations_with_name_filter() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create locations with different names
    let location1 = create_test_location_with_name("Building A");
    let location2 = create_test_location_with_name("Room 101");
    let location3 = create_test_location_with_name("Building B");

    create_location(&store, &location1).await.unwrap();
    create_location(&store, &location2).await.unwrap();
    create_location(&store, &location3).await.unwrap();

    let options = QueryOptions {
        filters: vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("Building".to_string()),
        }],
        ..Default::default()
    };

    let result = list_locations(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2);
    assert!(
        paged_result
            .items
            .iter()
            .all(|l| l.name.contains("Building"))
    );
}

#[tokio::test]
async fn test_list_locations_with_sorting() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create locations with names in non-alphabetical order
    let location1 = create_test_location_with_name("Charlie");
    let location2 = create_test_location_with_name("Alpha");
    let location3 = create_test_location_with_name("Bravo");

    create_location(&store, &location1).await.unwrap();
    create_location(&store, &location2).await.unwrap();
    create_location(&store, &location3).await.unwrap();

    let options = QueryOptions {
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }],
        ..Default::default()
    };

    let result = list_locations(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.items[0].name, "Alpha");
    assert_eq!(paged_result.items[1].name, "Bravo");
    assert_eq!(paged_result.items[2].name, "Charlie");
}

#[tokio::test]
async fn test_list_locations_with_descending_sort() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create locations with names in non-alphabetical order
    let location1 = create_test_location_with_name("Alpha");
    let location2 = create_test_location_with_name("Bravo");
    let location3 = create_test_location_with_name("Charlie");

    create_location(&store, &location1).await.unwrap();
    create_location(&store, &location2).await.unwrap();
    create_location(&store, &location3).await.unwrap();

    let options = QueryOptions {
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Descending,
        }],
        ..Default::default()
    };

    let result = list_locations(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 3);
    assert_eq!(paged_result.items[0].name, "Charlie");
    assert_eq!(paged_result.items[1].name, "Bravo");
    assert_eq!(paged_result.items[2].name, "Alpha");
}

#[tokio::test]
async fn test_update_location_success() {
    let (store, _temp_dir) = setup_test_store().await;
    let mut location = create_test_location_with_name("Original Name");

    // Create location first
    create_location(&store, &location).await.unwrap();

    // Update location
    location.name = "Updated Name".to_string();
    location.update_path(None); // Update path to match name for root location
    let result = update_location(&store, &location).await;
    assert!(result.is_ok());

    let updated_location = result.unwrap();
    assert_eq!(updated_location.name, "Updated Name");

    // Verify in store
    let stored_location = get_location(&store, &location.id).await.unwrap().unwrap();
    assert_eq!(stored_location.name, "Updated Name");
}

#[tokio::test]
async fn test_update_location_not_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let location = create_test_location_with_name("Test Location");

    // Try to update non-existent location
    let result = update_location(&store, &location).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "location");
            assert_eq!(id, location.id.to_string());
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_delete_location_success() {
    let (store, _temp_dir) = setup_test_store().await;
    let location = create_test_location_with_name("Test Location");

    // Create location first
    create_location(&store, &location).await.unwrap();

    // Delete location
    let result = delete_location(&store, &location.id).await;
    assert!(result.is_ok());

    // Verify location is gone
    let stored_location = get_location(&store, &location.id).await.unwrap();
    assert!(stored_location.is_none());
}

#[tokio::test]
async fn test_delete_location_not_exists() {
    let (store, _temp_dir) = setup_test_store().await;
    let non_existent_id = Uuid::new_v4();

    let result = delete_location(&store, &non_existent_id).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "location");
            assert_eq!(id, non_existent_id.to_string());
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_batch_locations_mixed_operations() {
    let (store, _temp_dir) = setup_test_store().await;

    // Create initial location
    let existing_location = create_test_location_with_name("Existing Location");
    create_location(&store, &existing_location).await.unwrap();

    // Prepare batch operations
    let new_location = create_test_location_with_name("New Location");
    let mut updated_location = existing_location.clone();
    updated_location.name = "Updated Existing Location".to_string();
    updated_location.update_path(None); // Update path to match name for root location

    let operations = vec![
        BatchOperation::Insert(new_location.clone()),
        BatchOperation::Update(updated_location.clone()),
        BatchOperation::Delete(Uuid::new_v4()), // This will fail
    ];

    let result = batch_locations(&store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 2);
    assert_eq!(batch_result.error_count, 1);
    assert_eq!(batch_result.errors.len(), 1);

    // Verify successful operations
    assert!(
        get_location(&store, &new_location.id)
            .await
            .unwrap()
            .is_some()
    );
    let stored_updated = get_location(&store, &existing_location.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_updated.name, "Updated Existing Location");
}

#[tokio::test]
async fn test_batch_locations_all_success() {
    let (store, _temp_dir) = setup_test_store().await;

    let location1 = create_test_location_with_name("Location 1");
    let location2 = create_test_location_with_name("Location 2");

    let operations = vec![
        BatchOperation::Insert(location1.clone()),
        BatchOperation::Insert(location2.clone()),
    ];

    let result = batch_locations(&store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 2);
    assert_eq!(batch_result.error_count, 0);
    assert!(batch_result.errors.is_empty());
}

#[tokio::test]
async fn test_batch_locations_empty_operations() {
    let (store, _temp_dir) = setup_test_store().await;

    let operations = vec![];
    let result = batch_locations(&store, &operations).await;
    assert!(result.is_ok());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.success_count, 0);
    assert_eq!(batch_result.error_count, 0);
}

#[tokio::test]
async fn test_locations_with_parent_filter() {
    let (store, _temp_dir) = setup_test_store().await;
    let parent_id = Uuid::new_v4();

    // Create locations with and without parent
    let location1 = create_test_location_with_parent("Child 1", Some(parent_id));
    let location2 = create_test_location_with_parent("Root", None);
    let location3 = create_test_location_with_parent("Child 2", Some(parent_id));

    create_location(&store, &location1).await.unwrap();
    create_location(&store, &location2).await.unwrap();
    create_location(&store, &location3).await.unwrap();

    let options = QueryOptions {
        filters: vec![Filter {
            field: "parent_id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Uuid(parent_id),
        }],
        ..Default::default()
    };

    let result = list_locations(&store, &options).await;
    assert!(result.is_ok());

    let paged_result = result.unwrap();
    assert_eq!(paged_result.items.len(), 2);
    assert!(
        paged_result
            .items
            .iter()
            .all(|l| l.parent_id == Some(parent_id))
    );
}
