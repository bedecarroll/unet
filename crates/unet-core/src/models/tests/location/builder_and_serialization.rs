//! Builder pattern and serialization tests for `Location` model

use crate::models::*;
use serde_json;
use uuid::Uuid;

#[test]
fn test_location_builder_success() {
    let parent_id = Uuid::new_v4();

    let location = LocationBuilder::new()
        .name("Building A")
        .location_type("building")
        .parent_id(parent_id)
        .parent_path("USA/California/San Francisco")
        .description("Main office building")
        .address("123 Main St, San Francisco, CA")
        .build()
        .unwrap();

    assert_eq!(location.name, "Building A");
    assert_eq!(location.location_type, "building");
    assert_eq!(location.parent_id, Some(parent_id));
    assert_eq!(location.path, "USA/California/San Francisco/Building A");
    assert_eq!(
        location.description,
        Some("Main office building".to_string())
    );
    assert_eq!(
        location.address,
        Some("123 Main St, San Francisco, CA".to_string())
    );
}

#[test]
fn test_location_builder_root_location() {
    let location = LocationBuilder::new()
        .name("USA")
        .location_type("country")
        .description("United States of America")
        .build()
        .unwrap();

    assert_eq!(location.name, "USA");
    assert_eq!(location.location_type, "country");
    assert_eq!(location.parent_id, None);
    assert_eq!(location.path, "USA");
    assert_eq!(
        location.description,
        Some("United States of America".to_string())
    );
}

#[test]
fn test_location_builder_missing_required_fields() {
    let result = LocationBuilder::new()
        .name("Incomplete")
        // Missing location_type
        .build();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Location type is required"));
}

#[test]
fn test_location_builder_validation_failure() {
    let result = LocationBuilder::new()
        .name("") // Invalid empty name
        .location_type("building")
        .build();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name cannot be empty"));
}

#[test]
fn test_location_serde() {
    let location = LocationBuilder::new()
        .name("Test Location")
        .location_type("test")
        .description("Test description")
        .build()
        .unwrap();

    let json = serde_json::to_string(&location).unwrap();
    let deserialized: Location = serde_json::from_str(&json).unwrap();

    assert_eq!(location.name, deserialized.name);
    assert_eq!(location.location_type, deserialized.location_type);
    assert_eq!(location.parent_id, deserialized.parent_id);
    assert_eq!(location.path, deserialized.path);
    assert_eq!(location.description, deserialized.description);
}
