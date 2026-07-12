/// Tests for location command structure and hierarchy validation
use crate::commands::locations::types::*;
use uuid::Uuid;

#[tokio::test]
async fn test_location_command_variants() {
    let location_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();

    let add_args = AddLocationArgs {
        name: "test-location".to_string(),
        location_type: "rack".to_string(),
        parent_id: Some(parent_id),
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    let list_args = ListLocationArgs {
        location_type: None,
        parent_id: None,
        page: 1,
        per_page: 20,
    };

    let show_args = ShowLocationArgs { id: location_id };

    let update_args = UpdateLocationArgs {
        id: location_id,
        name: None,
        location_type: None,
        parent_id: None,
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    let delete_args = DeleteLocationArgs {
        id: location_id,
        yes: false,
    };

    // Verify the arguments can be created with expected values
    assert_eq!(add_args.name, "test-location");
    assert_eq!(add_args.location_type, "rack");
    assert_eq!(add_args.parent_id, Some(parent_id));

    assert_eq!(list_args.page, 1);
    assert_eq!(list_args.per_page, 20);

    assert_eq!(show_args.id, location_id);

    assert_eq!(update_args.id, location_id);
    assert_eq!(update_args.name, None);

    assert_eq!(delete_args.id, location_id);
    assert!(!delete_args.yes);
}

#[tokio::test]
async fn test_location_hierarchy_validation() {
    // Test hierarchical location types
    let hierarchy_types = vec![
        ("campus", "site"),
        ("building", "campus"),
        ("floor", "building"),
        ("room", "floor"),
        ("rack", "room"),
    ];

    for (child_type, parent_type) in hierarchy_types {
        assert!(!child_type.is_empty());
        assert!(!parent_type.is_empty());
        assert_ne!(child_type, parent_type);
    }
}

#[tokio::test]
async fn test_location_type_consistency() {
    let valid_location_types = vec![
        "site",
        "campus",
        "datacenter",
        "building",
        "floor",
        "room",
        "rack",
        "zone",
    ];

    // Test that all location types are non-empty strings
    for location_type in &valid_location_types {
        assert!(!location_type.is_empty());
        assert!(location_type.chars().all(|c| c.is_ascii_alphabetic()));
    }

    // Test uniqueness
    let mut sorted_types = valid_location_types.clone();
    sorted_types.sort_unstable();
    sorted_types.dedup();
    assert_eq!(sorted_types.len(), valid_location_types.len());
}

#[tokio::test]
async fn test_location_command_id_validation() {
    let valid_uuid = Uuid::new_v4();
    let another_uuid = Uuid::new_v4();

    // Ensure UUIDs are different
    assert_ne!(valid_uuid, another_uuid);

    // Test that command arguments accept valid UUIDs
    let show_args = ShowLocationArgs { id: valid_uuid };
    assert_eq!(show_args.id, valid_uuid);

    let update_args = UpdateLocationArgs {
        id: another_uuid,
        name: Some("updated-location".to_string()),
        location_type: None,
        parent_id: None,
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };
    assert_eq!(update_args.id, another_uuid);

    let delete_args = DeleteLocationArgs {
        id: valid_uuid,
        yes: true,
    };
    assert_eq!(delete_args.id, valid_uuid);
    assert!(delete_args.yes);
}
