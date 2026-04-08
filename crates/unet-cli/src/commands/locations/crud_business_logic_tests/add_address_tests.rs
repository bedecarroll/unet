use crate::commands::locations::types::AddLocationArgs;
use unet_core::models::LocationBuilder;
use uuid::Uuid;

fn combine_address(
    address: Option<String>,
    city: Option<String>,
    country: Option<String>,
) -> Option<String> {
    let mut address_parts = Vec::new();
    if let Some(value) = address {
        address_parts.push(value);
    }
    if let Some(value) = city {
        address_parts.push(value);
    }
    if let Some(value) = country {
        address_parts.push(value);
    }

    if address_parts.is_empty() {
        None
    } else {
        Some(address_parts.join(", "))
    }
}

#[tokio::test]
async fn test_add_location_args_validation_empty_name() {
    let args = AddLocationArgs {
        name: String::new(),
        location_type: "datacenter".to_string(),
        parent_id: None,
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    let result = LocationBuilder::new()
        .name(args.name)
        .location_type(args.location_type)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_location_args_validation_empty_location_type() {
    let args = AddLocationArgs {
        name: "test-datacenter".to_string(),
        location_type: String::new(),
        parent_id: None,
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    let result = LocationBuilder::new()
        .name(args.name)
        .location_type(args.location_type)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_location_args_validation_valid_minimum() {
    let args = AddLocationArgs {
        name: "test-datacenter".to_string(),
        location_type: "datacenter".to_string(),
        parent_id: None,
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    let result = LocationBuilder::new()
        .name(args.name)
        .location_type(args.location_type)
        .build();

    assert!(result.is_ok());
    let location = result.unwrap();
    assert_eq!(location.name, "test-datacenter");
    assert_eq!(location.location_type, "datacenter");
    assert_eq!(location.parent_id, None);
    assert_eq!(location.address, None);
}

#[tokio::test]
async fn test_add_location_args_validation_with_parent() {
    let parent_id = Uuid::new_v4();
    let args = AddLocationArgs {
        name: "test-rack".to_string(),
        location_type: "rack".to_string(),
        parent_id: Some(parent_id),
        address: None,
        city: None,
        country: None,
        custom_data: None,
    };

    let result = LocationBuilder::new()
        .name(args.name)
        .location_type(args.location_type)
        .parent_id(parent_id)
        .build();

    assert!(result.is_ok());
    let location = result.unwrap();
    assert_eq!(location.parent_id, Some(parent_id));
}

#[tokio::test]
async fn test_address_combination_single_address() {
    let combined_address = combine_address(Some("123 Server St".to_string()), None, None);
    assert_eq!(combined_address, Some("123 Server St".to_string()));
}

#[tokio::test]
async fn test_address_combination_address_and_city() {
    let combined_address = combine_address(
        Some("123 Server St".to_string()),
        Some("New York".to_string()),
        None,
    );
    assert_eq!(combined_address, Some("123 Server St, New York".to_string()));
}

#[tokio::test]
async fn test_address_combination_all_fields() {
    let combined_address = combine_address(
        Some("123 Server St".to_string()),
        Some("New York".to_string()),
        Some("USA".to_string()),
    );
    assert_eq!(combined_address, Some("123 Server St, New York, USA".to_string()));
}

#[tokio::test]
async fn test_address_combination_city_and_country_only() {
    let combined_address = combine_address(
        None,
        Some("New York".to_string()),
        Some("USA".to_string()),
    );
    assert_eq!(combined_address, Some("New York, USA".to_string()));
}

#[tokio::test]
async fn test_address_combination_empty() {
    let combined_address = combine_address(None, None, None);
    assert_eq!(combined_address, None);
}
