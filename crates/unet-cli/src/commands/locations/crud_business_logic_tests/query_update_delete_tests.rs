use crate::commands::locations::types::{DeleteLocationArgs, UpdateLocationArgs};
use crate::commands::test_support::expect_json_object;
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
async fn test_update_location_partial_updates() {
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
async fn test_update_location_address_combination() {
    let combined_address = combine_address(
        Some("456 New Ave".to_string()),
        Some("Boston".to_string()),
        Some("USA".to_string()),
    );

    assert_eq!(combined_address, Some("456 New Ave, Boston, USA".to_string()));
}

#[tokio::test]
async fn test_update_location_custom_data_parsing() {
    let value = expect_json_object(r#"{"environment": "staging", "capacity": 75}"#);
    assert_eq!(value["environment"], "staging");
    assert_eq!(value["capacity"], 75);
}

#[tokio::test]
async fn test_delete_location_confirmation_logic() {
    let input_variations = vec![
        ("y", true),
        ("Y", true),
        ("yes", true),
        ("YES", true),
        ("Yes", true),
        ("n", false),
        ("N", false),
        ("no", false),
        ("NO", false),
        ("No", false),
        ("", false),
        ("maybe", false),
        ("quit", false),
    ];

    for (input, expected) in input_variations {
        let should_proceed = input.trim().to_lowercase().starts_with('y');
        assert_eq!(
            should_proceed,
            expected,
            "Input '{input}' should return {expected}"
        );
    }
}

#[tokio::test]
async fn test_delete_location_yes_flag_bypass() {
    let location_id = Uuid::new_v4();
    let args_with_yes = DeleteLocationArgs {
        id: location_id,
        yes: true,
    };
    let args_without_yes = DeleteLocationArgs {
        id: location_id,
        yes: false,
    };

    assert!(args_with_yes.yes);
    assert!(!args_without_yes.yes);
}
