//! Basic operations tests for `Location` model

use crate::models::*;
use serde_json;
use uuid::Uuid;

#[test]
fn test_location_new_root() {
    let location = Location::new_root("USA".to_string(), "country".to_string());

    assert_eq!(location.name, "USA");
    assert_eq!(location.location_type, "country");
    assert_eq!(location.parent_id, None);
    assert_eq!(location.path, "USA");
    assert!(location.custom_data.is_null());
}

#[test]
fn test_location_new_child() {
    let location = Location::new_child("California".to_string(), "state".to_string(), "USA");

    assert_eq!(location.name, "California");
    assert_eq!(location.location_type, "state");
    assert_eq!(location.parent_id, None); // Will be set by caller
    assert_eq!(location.path, "USA/California");
}

#[test]
fn test_location_new_child_empty_parent() {
    let location = Location::new_child("RootLocation".to_string(), "building".to_string(), "");

    assert_eq!(location.path, "RootLocation");
}

#[test]
fn test_location_validation_success() {
    let location = Location::new_root("USA".to_string(), "country".to_string());
    assert!(location.validate().is_ok());

    let mut child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child.parent_id = Some(Uuid::new_v4());
    assert!(child.validate().is_ok());
}

#[test]
fn test_location_validation_empty_name() {
    let location = Location::new_root(String::new(), "country".to_string());
    assert!(location.validate().is_err());
    assert!(
        location
            .validate()
            .unwrap_err()
            .contains("name cannot be empty")
    );
}

#[test]
fn test_location_validation_empty_type() {
    let location = Location::new_root("USA".to_string(), String::new());
    assert!(location.validate().is_err());
    assert!(
        location
            .validate()
            .unwrap_err()
            .contains("type cannot be empty")
    );
}

#[test]
fn test_location_validation_root_path_mismatch() {
    let mut location = Location::new_root("USA".to_string(), "country".to_string());
    location.path = "Wrong".to_string();

    assert!(location.validate().is_err());
    assert!(
        location
            .validate()
            .unwrap_err()
            .contains("Root location path must equal name")
    );
}

#[test]
fn test_location_validation_child_path_mismatch() {
    let mut location = Location::new_child("California".to_string(), "state".to_string(), "USA");
    location.parent_id = Some(Uuid::new_v4());
    location.path = "USA/WrongState".to_string();

    assert!(location.validate().is_err());
    assert!(
        location
            .validate()
            .unwrap_err()
            .contains("path must end with location name")
    );
}

#[test]
fn test_location_update_path() {
    let mut location = Location::new_root("California".to_string(), "state".to_string());

    // Update with parent path
    location.update_path(Some("USA"));
    assert_eq!(location.path, "USA/California");

    // Update without parent path
    location.update_path(None);
    assert_eq!(location.path, "California");

    // Update with empty parent path
    location.update_path(Some(""));
    assert_eq!(location.path, "California");
}

#[test]
fn test_location_get_depth() {
    let root = Location::new_root("USA".to_string(), "country".to_string());
    assert_eq!(root.get_depth(), 0);

    let child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    assert_eq!(child.get_depth(), 1);

    let grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    assert_eq!(grandchild.get_depth(), 2);
}

#[test]
fn test_location_get_path_components() {
    let root = Location::new_root("USA".to_string(), "country".to_string());
    assert_eq!(root.get_path_components(), vec!["USA"]);

    let child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    assert_eq!(child.get_path_components(), vec!["USA", "California"]);

    let grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    assert_eq!(
        grandchild.get_path_components(),
        vec!["USA", "California", "San Francisco"]
    );
}

#[test]
fn test_location_custom_data() {
    let mut location = Location::new_root("USA".to_string(), "country".to_string());

    // Set custom data
    let value = serde_json::json!("UTC-8");
    assert!(location.set_custom_data("timezone", value.clone()).is_ok());

    // Get custom data
    let retrieved = location.get_custom_data("timezone");
    assert_eq!(retrieved, Some(&value));

    // Test nested path
    let coords = serde_json::json!({"lat": 37.7749, "lng": -122.4194});
    assert!(
        location
            .set_custom_data("coordinates.center", coords.clone())
            .is_ok()
    );

    let retrieved_coords = location.get_custom_data("coordinates.center");
    assert_eq!(retrieved_coords, Some(&coords));
}
