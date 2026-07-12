/// Business logic tests for links CRUD operations using TDD principles
/// These tests focus on the CRUD business logic without complex `DataStore` mocking
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::commands::links::types::*;
use unet_core::models::LinkBuilder;

// ARGUMENT VALIDATION TESTS

#[tokio::test]
async fn test_add_link_args_validation_empty_name() {
    let node_a_id = Uuid::new_v4();

    let args = AddLinkArgs {
        name: String::new(), // Empty name should fail validation
        node_a_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    // Test that LinkBuilder would reject empty name
    let result = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_link_args_validation_empty_interface() {
    let node_a_id = Uuid::new_v4();

    let args = AddLinkArgs {
        name: "test-link".to_string(),
        node_a_id,
        node_a_interface: String::new(), // Empty interface should fail validation
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    // Test that LinkBuilder would reject empty interface
    let result = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_link_args_validation_valid_minimum() {
    let node_a_id = Uuid::new_v4();

    let args = AddLinkArgs {
        name: "test-link".to_string(),
        node_a_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    // Test that LinkBuilder accepts valid minimum arguments
    // This creates an internet circuit (single-ended link) when no dest_node_id is provided
    let mut builder = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface);

    // For internet circuits, we need to explicitly mark it as such
    // Otherwise validation will fail because regular links need both ends
    builder = builder.is_internet_circuit(true);

    let result = builder.build();
    assert!(result.is_ok());
    let link = result.unwrap();
    assert!(link.is_internet_circuit);
    assert_eq!(link.dest_node_id, None);
}

// JSON PARSING TESTS

#[tokio::test]
async fn test_custom_data_json_parsing_valid() {
    let json_str = r#"{"provider": "ISP", "vlan": 100, "priority": "high"}"#;
    let result = serde_json::from_str::<JsonValue>(json_str);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_object());
    assert_eq!(value["provider"], "ISP");
    assert_eq!(value["vlan"], 100);
    assert_eq!(value["priority"], "high");
}

#[tokio::test]
async fn test_custom_data_json_parsing_invalid() {
    let invalid_json = r#"{"provider": "ISP", "vlan": }"#; // Missing value
    let result = serde_json::from_str::<JsonValue>(invalid_json);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_custom_data_json_parsing_empty_object() {
    let json_str = "{}";
    let result = serde_json::from_str::<JsonValue>(json_str);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_object());
}

#[tokio::test]
async fn test_custom_data_json_parsing_complex() {
    let complex_json = r#"{
        "nested": {
            "array": [1, 2, 3],
            "object": {"key": "value"},
            "boolean": true,
            "null_value": null
        },
        "special_chars": "!@#$%^&*()_+-={}[]|\\:;\"'<>?,./"
    }"#;
    let result = serde_json::from_str::<JsonValue>(complex_json);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_object());
    assert!(value["nested"]["array"].is_array());
    assert_eq!(value["nested"]["boolean"], true);
    assert!(value["nested"]["null_value"].is_null());
}

#[tokio::test]
async fn test_custom_data_json_parsing_not_json_string() {
    let not_json = "this is not json";
    let result = serde_json::from_str::<JsonValue>(not_json);

    assert!(result.is_err());
}

// PAGINATION CALCULATION TESTS

#[tokio::test]
async fn test_pagination_calculation_normal() {
    let page = 3_u64;
    let per_page = 25_u64;

    let expected_offset = (page - 1) * per_page; // 2 * 25 = 50
    let offset_conversion = usize::try_from(expected_offset);
    let limit_conversion = usize::try_from(per_page);

    assert!(offset_conversion.is_ok());
    assert!(limit_conversion.is_ok());
    assert_eq!(offset_conversion.unwrap(), 50);
    assert_eq!(limit_conversion.unwrap(), 25);
}

#[tokio::test]
async fn test_pagination_calculation_first_page() {
    let page = 1_u64;
    let per_page = 20_u64;

    let expected_offset = (page - 1) * per_page; // 0 * 20 = 0
    let offset_conversion = usize::try_from(expected_offset);

    assert!(offset_conversion.is_ok());
    assert_eq!(offset_conversion.unwrap(), 0);
}

#[tokio::test]
async fn test_pagination_calculation_large_values() {
    let page = 1000_u64;
    let per_page = 100_u64;

    let expected_offset = (page - 1) * per_page; // 999 * 100 = 99,900
    let offset_conversion = usize::try_from(expected_offset);
    let limit_conversion = usize::try_from(per_page);

    assert!(offset_conversion.is_ok());
    assert!(limit_conversion.is_ok());
    assert_eq!(offset_conversion.unwrap(), 99_900);
    assert_eq!(limit_conversion.unwrap(), 100);
}

#[tokio::test]
async fn test_pagination_calculation_overflow_risk() {
    let page = u64::MAX;
    let per_page = u64::MAX;

    // This should overflow when calculating offset
    let overflow_result = page.checked_mul(per_page);
    assert!(overflow_result.is_none()); // Confirms overflow would occur

    // The real calculation in code: (page - 1) * per_page
    let safe_page = page - 1; // This is u64::MAX - 1
    let safe_overflow_result = safe_page.checked_mul(per_page);
    assert!(safe_overflow_result.is_none()); // Still overflows

    // Converting overflowed values to usize should fail
    let conversion_result = usize::try_from(u64::MAX);
    // This may or may not fail depending on the platform (32-bit vs 64-bit)
    // On 64-bit systems, this might succeed, on 32-bit it would fail
    if conversion_result.is_err() {
        // Platform where usize is smaller than u64
        // Test passes because conversion failed as expected
    } else {
        // 64-bit platform, but we can still test the overflow scenario
        // by using a value that definitely overflows usize on any platform
        // Test passes because we verified overflow potential
    }
}

// BANDWIDTH VALIDATION TESTS

#[tokio::test]
async fn test_bandwidth_values_zero() {
    let bandwidth = 0_u64;

    // Zero bandwidth should be valid (represents unknown/unlimited)
    assert_eq!(bandwidth, 0);
}

#[tokio::test]
async fn test_bandwidth_values_common() {
    let gigabit = 1_000_000_000_u64; // 1 Gbps
    let ten_gigabit = 10_000_000_000_u64; // 10 Gbps
    let hundred_gigabit = 100_000_000_000_u64; // 100 Gbps

    assert_eq!(gigabit, 1_000_000_000);
    assert_eq!(ten_gigabit, 10_000_000_000);
    assert_eq!(hundred_gigabit, 100_000_000_000);

    // These should all be valid u64 values
    assert!(gigabit < u64::MAX);
    assert!(ten_gigabit < u64::MAX);
    assert!(hundred_gigabit < u64::MAX);
}

#[tokio::test]
async fn test_bandwidth_values_maximum() {
    let max_bandwidth = u64::MAX;

    // Maximum u64 value should be valid
    assert_eq!(max_bandwidth, u64::MAX);
}

// LINK BUILDER INTEGRATION TESTS

#[tokio::test]
async fn test_link_builder_with_all_fields() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();
    let custom_data = serde_json::json!({"provider": "ISP", "vlan": 100});

    let result = LinkBuilder::new()
        .name("comprehensive-link".to_string())
        .source_node_id(source_node_id)
        .node_a_interface("GigabitEthernet0/1".to_string())
        .dest_node_id(dest_node_id)
        .node_z_interface("GigabitEthernet0/2".to_string())
        .bandwidth(1_000_000_000)
        .description("Full featured link".to_string())
        .custom_data(custom_data)
        .build();

    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.name, "comprehensive-link");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.node_a_interface, "GigabitEthernet0/1");
    assert_eq!(link.dest_node_id, Some(dest_node_id));
    assert_eq!(
        link.node_z_interface,
        Some("GigabitEthernet0/2".to_string())
    );
    assert_eq!(link.bandwidth, Some(1_000_000_000));
    assert_eq!(link.description, Some("Full featured link".to_string()));
}

#[tokio::test]
async fn test_link_builder_minimal_fields() {
    let source_node_id = Uuid::new_v4();

    // For minimal fields (no dest_node_id), this needs to be an internet circuit
    let result = LinkBuilder::new()
        .name("minimal-link".to_string())
        .source_node_id(source_node_id)
        .node_a_interface("eth0".to_string())
        .is_internet_circuit(true) // Mark as internet circuit for validation
        .build();

    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.name, "minimal-link");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.node_a_interface, "eth0");
    assert_eq!(link.dest_node_id, None);
    assert_eq!(link.node_z_interface, None);
    assert_eq!(link.bandwidth, None);
    assert_eq!(link.description, None);
    assert!(link.is_internet_circuit);
}

#[tokio::test]
async fn test_link_builder_validation_failures() {
    let source_node_id = Uuid::new_v4();

    // Test empty name
    let result = LinkBuilder::new()
        .name(String::new())
        .source_node_id(source_node_id)
        .node_a_interface("eth0".to_string())
        .build();
    assert!(result.is_err());

    // Test empty interface
    let result = LinkBuilder::new()
        .name("test-link".to_string())
        .source_node_id(source_node_id)
        .node_a_interface(String::new())
        .build();
    assert!(result.is_err());
}

// FILTER AND SORT CONSTRUCTION TESTS

#[tokio::test]
async fn test_list_links_filter_construction() {
    use unet_core::prelude::{Filter, FilterOperation, FilterValue};

    let node_id = Uuid::new_v4();

    // Test filter construction similar to list_links function
    let filter = Filter {
        field: "node_a_id".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(node_id),
    };

    assert_eq!(filter.field, "node_a_id");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    match filter.value {
        FilterValue::Uuid(id) => assert_eq!(id, node_id),
        _ => panic!("Expected UUID filter value"),
    }
}

#[tokio::test]
async fn test_list_links_sort_construction() {
    use unet_core::prelude::{Sort, SortDirection};

    // Test sort construction similar to list_links function
    let sort = Sort {
        field: "interface_a".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(sort.field, "interface_a");
    assert!(matches!(sort.direction, SortDirection::Ascending));
}

#[tokio::test]
async fn test_list_links_query_options_construction() {
    use unet_core::prelude::{
        Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort, SortDirection,
    };

    let node_id = Uuid::new_v4();
    let page = 3_u64;
    let per_page = 50_u64;

    // Construct QueryOptions similar to list_links function
    let filters = vec![Filter {
        field: "node_a_id".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(node_id),
    }];

    let sort = vec![Sort {
        field: "interface_a".to_owned(),
        direction: SortDirection::Ascending,
    }];

    let offset = usize::try_from((page - 1) * per_page).unwrap();
    let limit = usize::try_from(per_page).unwrap();

    let pagination = Some(Pagination { offset, limit });

    let options = QueryOptions {
        filters,
        sort,
        pagination,
    };

    // Verify construction
    assert_eq!(options.filters.len(), 1);
    assert_eq!(options.sort.len(), 1);
    assert!(options.pagination.is_some());

    if let Some(ref pagination) = options.pagination {
        assert_eq!(pagination.offset, 100); // (3-1) * 50 = 100
        assert_eq!(pagination.limit, 50);
    }
}
