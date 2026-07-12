/// Tests for location command arguments
use crate::commands::locations::types::*;
use uuid::Uuid;

#[tokio::test]
async fn test_add_location_args_creation() {
    let parent_id = Uuid::new_v4();

    let args = AddLocationArgs {
        name: "datacenter-east".to_string(),
        location_type: "datacenter".to_string(),
        parent_id: Some(parent_id),
        address: Some("123 Server St".to_string()),
        city: Some("New York".to_string()),
        country: Some("USA".to_string()),
        custom_data: Some(r#"{"zone": "production"}"#.to_string()),
    };

    assert_eq!(args.name, "datacenter-east");
    assert_eq!(args.location_type, "datacenter");
    assert_eq!(args.parent_id, Some(parent_id));
    assert_eq!(args.address, Some("123 Server St".to_string()));
    assert_eq!(args.city, Some("New York".to_string()));
    assert_eq!(args.country, Some("USA".to_string()));
    assert!(args.custom_data.is_some());
}

#[tokio::test]
async fn test_add_location_args_minimal() {
    let args = AddLocationArgs {
        name: "rack-a1".to_string(),
        location_type: "rack".to_string(),
        parent_id: None,
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    assert_eq!(args.name, "rack-a1");
    assert_eq!(args.location_type, "rack");
    assert_eq!(args.parent_id, None);
    assert_eq!(args.address, None);
    assert_eq!(args.city, None);
    assert_eq!(args.country, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_list_location_args_creation() {
    let parent_id = Uuid::new_v4();

    let args = ListLocationArgs {
        parent_id: Some(parent_id),
        location_type: Some("datacenter".to_string()),
        page: 2,
        per_page: 50,
    };

    assert_eq!(args.parent_id, Some(parent_id));
    assert_eq!(args.location_type, Some("datacenter".to_string()));
    assert_eq!(args.page, 2);
    assert_eq!(args.per_page, 50);
}

#[tokio::test]
async fn test_list_location_args_defaults() {
    let args = ListLocationArgs {
        parent_id: None,
        location_type: None,
        page: 1,
        per_page: 20,
    };

    assert_eq!(args.parent_id, None);
    assert_eq!(args.location_type, None);
    assert_eq!(args.page, 1);
    assert_eq!(args.per_page, 20);
}

#[tokio::test]
async fn test_show_location_args_creation() {
    let location_id = Uuid::new_v4();
    let args = ShowLocationArgs { id: location_id };

    assert_eq!(args.id, location_id);
}

#[tokio::test]
async fn test_update_location_args_creation() {
    let location_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();

    let args = UpdateLocationArgs {
        id: location_id,
        name: Some("updated-datacenter".to_string()),
        location_type: Some("building".to_string()),
        parent_id: Some(parent_id),
        address: Some("456 New Ave".to_string()),
        city: Some("Boston".to_string()),
        country: Some("USA".to_string()),
        custom_data: Some(r#"{"environment": "staging"}"#.to_string()),
    };

    assert_eq!(args.id, location_id);
    assert_eq!(args.name, Some("updated-datacenter".to_string()));
    assert_eq!(args.location_type, Some("building".to_string()));
    assert_eq!(args.parent_id, Some(parent_id));
    assert_eq!(args.address, Some("456 New Ave".to_string()));
    assert_eq!(args.city, Some("Boston".to_string()));
    assert_eq!(args.country, Some("USA".to_string()));
    assert!(args.custom_data.is_some());
}

#[tokio::test]
async fn test_update_location_args_partial() {
    let location_id = Uuid::new_v4();

    let args = UpdateLocationArgs {
        id: location_id,
        name: Some("updated-name".to_string()),
        location_type: None,
        parent_id: None,
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    assert_eq!(args.id, location_id);
    assert_eq!(args.name, Some("updated-name".to_string()));
    assert_eq!(args.location_type, None);
    assert_eq!(args.parent_id, None);
    assert_eq!(args.address, None);
    assert_eq!(args.city, None);
    assert_eq!(args.country, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_delete_location_args_creation() {
    let location_id = Uuid::new_v4();

    let args = DeleteLocationArgs {
        id: location_id,
        yes: false,
    };

    assert_eq!(args.id, location_id);
    assert!(!args.yes);
}

#[tokio::test]
async fn test_delete_location_args_interactive() {
    let location_id = Uuid::new_v4();

    let args = DeleteLocationArgs {
        id: location_id,
        yes: true,
    };

    assert_eq!(args.id, location_id);
    assert!(args.yes);
}
