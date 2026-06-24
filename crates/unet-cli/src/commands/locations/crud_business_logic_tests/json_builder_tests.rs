use crate::commands::test_support::{expect_json_object, expect_json_parse_error, pagination_values};
use unet_core::models::LocationBuilder;
use unet_core::prelude::{
    Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort, SortDirection,
};
use uuid::Uuid;

#[tokio::test]
async fn test_custom_data_json_parsing_valid() {
    let value = expect_json_object(
        r#"{"zone": "production", "capacity": 42, "contact": "admin@example.com"}"#,
    );

    assert_eq!(value["zone"], "production");
    assert_eq!(value["capacity"], 42);
    assert_eq!(value["contact"], "admin@example.com");
}

#[tokio::test]
async fn test_custom_data_json_parsing_invalid() {
    expect_json_parse_error(r#"{"zone": "production", "capacity": }"#);
}

#[tokio::test]
async fn test_custom_data_json_parsing_empty_object() {
    let value = expect_json_object("{}");
    assert!(value.as_object().unwrap().is_empty());
}

#[tokio::test]
async fn test_custom_data_json_parsing_complex() {
    let value = expect_json_object(
        r#"{
            "infrastructure": {
                "racks": 20,
                "power_circuits": 4,
                "cooling": {
                    "type": "water",
                    "redundancy": "N+1"
                }
            },
            "contacts": [
                {"name": "John Doe", "role": "admin", "phone": "555-1234"},
                {"name": "Jane Smith", "role": "operations", "email": "jane@example.com"}
            ],
            "maintenance_windows": ["02:00-04:00", "14:00-16:00"]
        }"#,
    );

    assert_eq!(value["infrastructure"]["cooling"]["type"], "water");
    assert!(value["contacts"].is_array());
    assert!(value["maintenance_windows"].is_array());
}

#[tokio::test]
async fn test_location_builder_with_all_fields() {
    let parent_id = Uuid::new_v4();
    let custom_data = serde_json::json!({
        "zone": "production",
        "capacity": 100,
        "contact": "admin@example.com"
    });

    let result = LocationBuilder::new()
        .name("comprehensive-datacenter".to_string())
        .location_type("datacenter".to_string())
        .parent_id(parent_id)
        .address("123 Server St, New York, USA".to_string())
        .custom_data(custom_data)
        .build();

    assert!(result.is_ok());
    let location = result.unwrap();
    assert_eq!(location.parent_id, Some(parent_id));
    assert_eq!(location.address, Some("123 Server St, New York, USA".to_string()));
    assert!(location.custom_data.is_object());
}

#[tokio::test]
async fn test_location_builder_minimal_fields() {
    let result = LocationBuilder::new()
        .name("minimal-datacenter".to_string())
        .location_type("datacenter".to_string())
        .build();

    assert!(result.is_ok());
    let location = result.unwrap();
    assert_eq!(location.parent_id, None);
    assert_eq!(location.address, None);
    assert!(location.custom_data.is_null());
}

#[tokio::test]
async fn test_location_builder_validation_failures() {
    let result = LocationBuilder::new()
        .name(String::new())
        .location_type("datacenter".to_string())
        .build();
    assert!(result.is_err());

    let result = LocationBuilder::new()
        .name("test-location".to_string())
        .location_type(String::new())
        .build();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_pagination_calculation_normal() {
    let (offset, limit) = pagination_values(3, 25);
    assert_eq!(offset, 50);
    assert_eq!(limit, 25);
}

#[tokio::test]
async fn test_pagination_calculation_first_page() {
    let (offset, limit) = pagination_values(1, 20);
    assert_eq!(offset, 0);
    assert_eq!(limit, 20);
}

#[tokio::test]
async fn test_pagination_calculation_large_values() {
    let (offset, limit) = pagination_values(1_000, 100);
    assert_eq!(offset, 99_900);
    assert_eq!(limit, 100);
}

#[tokio::test]
async fn test_pagination_calculation_overflow_risk() {
    let page = u64::MAX;
    let per_page = u64::MAX;

    let overflow_result = page.checked_mul(per_page);
    assert!(overflow_result.is_none());

    let safe_page = page - 1;
    let safe_overflow_result = safe_page.checked_mul(per_page);
    assert!(safe_overflow_result.is_none());

    let conversion_result = usize::try_from(u64::MAX);
    if conversion_result.is_err() {
        assert!(conversion_result.is_err());
    } else {
        assert!(conversion_result.is_ok());
    }
}

#[tokio::test]
async fn test_list_locations_filter_construction_by_type() {
    let location_type = "datacenter".to_string();
    let filter = Filter {
        field: "location_type".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::String(location_type.clone()),
    };

    assert_eq!(filter.field, "location_type");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    match filter.value {
        FilterValue::String(value) => assert_eq!(value, location_type),
        _ => panic!("Expected String filter value"),
    }
}

#[tokio::test]
async fn test_list_locations_filter_construction_by_parent() {
    let parent_id = Uuid::new_v4();
    let filter = Filter {
        field: "parent_id".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(parent_id),
    };

    assert_eq!(filter.field, "parent_id");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    match filter.value {
        FilterValue::Uuid(id) => assert_eq!(id, parent_id),
        _ => panic!("Expected UUID filter value"),
    }
}

#[tokio::test]
async fn test_list_locations_sort_construction() {
    let sort = Sort {
        field: "name".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(sort.field, "name");
    assert!(matches!(sort.direction, SortDirection::Ascending));
}

#[tokio::test]
async fn test_list_locations_query_options_construction() {
    let parent_id = Uuid::new_v4();
    let filters = vec![
        Filter {
            field: "location_type".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("datacenter".to_string()),
        },
        Filter {
            field: "parent_id".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::Uuid(parent_id),
        },
    ];
    let sort = vec![Sort {
        field: "name".to_owned(),
        direction: SortDirection::Ascending,
    }];
    let (offset, limit) = pagination_values(2, 30);
    let options = QueryOptions {
        filters,
        sort,
        pagination: Some(Pagination { offset, limit }),
    };

    assert_eq!(options.filters.len(), 2);
    assert_eq!(options.sort.len(), 1);
    assert!(options.pagination.is_some());

    if let Some(pagination) = options.pagination {
        assert_eq!(pagination.offset, 30);
        assert_eq!(pagination.limit, 30);
    }
}
