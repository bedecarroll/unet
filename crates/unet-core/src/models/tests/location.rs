//! Tests for `Location` model and `LocationBuilder`

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
fn test_location_hierarchy_relationships() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child.id);

    // Test ancestor relationships
    assert!(root.is_ancestor_of(&child));
    assert!(root.is_ancestor_of(&grandchild));
    assert!(child.is_ancestor_of(&grandchild));
    assert!(!child.is_ancestor_of(&root));
    assert!(!grandchild.is_ancestor_of(&root));

    // Test descendant relationships
    assert!(child.is_descendant_of(&root));
    assert!(grandchild.is_descendant_of(&root));
    assert!(grandchild.is_descendant_of(&child));
    assert!(!root.is_descendant_of(&child));

    // Test parent-child relationships
    assert!(child.is_child_of(&root));
    assert!(grandchild.is_child_of(&child));
    assert!(root.is_parent_of(&child));
    assert!(child.is_parent_of(&grandchild));
    assert!(!root.is_child_of(&child));
    assert!(!child.is_parent_of(&root));
}

#[test]
fn test_location_detect_circular_reference() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child.parent_id = Some(root.id);

    let locations = vec![root.clone(), child.clone()];

    // Test self-reference
    assert!(Location::detect_circular_reference(
        &locations, root.id, root.id
    ));

    // Test valid parent-child
    assert!(!Location::detect_circular_reference(
        &locations, root.id, child.id
    ));

    // Test circular reference (child becoming parent of its ancestor)
    assert!(Location::detect_circular_reference(
        &locations, child.id, root.id
    ));
}

#[test]
fn test_location_get_ancestors() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child.id);

    let locations = vec![root.clone(), child.clone(), grandchild.clone()];

    // Root has no ancestors
    let root_ancestors = root.get_ancestors(&locations);
    assert!(root_ancestors.is_empty());

    // Child has root as ancestor
    let child_ancestors = child.get_ancestors(&locations);
    assert_eq!(child_ancestors.len(), 1);
    assert_eq!(child_ancestors[0].id, root.id);

    // Grandchild has child and root as ancestors
    let grandchild_ancestors = grandchild.get_ancestors(&locations);
    assert_eq!(grandchild_ancestors.len(), 2);
    assert_eq!(grandchild_ancestors[0].id, child.id);
    assert_eq!(grandchild_ancestors[1].id, root.id);
}

#[test]
fn test_location_get_descendants() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child1 = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child1.parent_id = Some(root.id);

    let mut child2 = Location::new_child("Texas".to_string(), "state".to_string(), "USA");
    child2.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child1.id);

    let locations = vec![
        root.clone(),
        child1.clone(),
        child2.clone(),
        grandchild.clone(),
    ];

    // Root has all others as descendants
    let root_descendants = root.get_descendants(&locations);
    assert_eq!(root_descendants.len(), 3);

    // Child1 has only grandchild as descendant
    let child1_descendants = child1.get_descendants(&locations);
    assert_eq!(child1_descendants.len(), 1);
    assert_eq!(child1_descendants[0].id, grandchild.id);

    // Child2 has no descendants
    let child2_descendants = child2.get_descendants(&locations);
    assert!(child2_descendants.is_empty());

    // Grandchild has no descendants
    let grandchild_descendants = grandchild.get_descendants(&locations);
    assert!(grandchild_descendants.is_empty());
}

#[test]
fn test_location_get_children() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child1 = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child1.parent_id = Some(root.id);

    let mut child2 = Location::new_child("Texas".to_string(), "state".to_string(), "USA");
    child2.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child1.id);

    let locations = vec![
        root.clone(),
        child1.clone(),
        child2.clone(),
        grandchild.clone(),
    ];

    // Root has two direct children
    let root_children = root.get_children(&locations);
    assert_eq!(root_children.len(), 2);
    let child_ids: Vec<Uuid> = root_children.iter().map(|l| l.id).collect();
    assert!(child_ids.contains(&child1.id));
    assert!(child_ids.contains(&child2.id));

    // Child1 has one direct child
    let child1_children = child1.get_children(&locations);
    assert_eq!(child1_children.len(), 1);
    assert_eq!(child1_children[0].id, grandchild.id);

    // Child2 has no direct children
    let child2_children = child2.get_children(&locations);
    assert!(child2_children.is_empty());
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
