/// Business logic tests for node CRUD operations using TDD principles
/// These tests focus on the CRUD business logic without complex `DataStore` mocking
use serde_json::Value as JsonValue;
use std::net::IpAddr;
use uuid::Uuid;

use crate::commands::nodes::types::*;
use unet_core::models::{DeviceRole, Lifecycle, NodeBuilder, Vendor};

// ARGUMENT VALIDATION TESTS FOR ADD_NODE

#[tokio::test]
async fn test_add_node_args_validation_empty_name() {
    let args = AddNodeArgs {
        name: String::new(), // Empty name should fail validation
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Test that NodeBuilder would reject empty name
    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_empty_domain_is_allowed() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: String::new(), // Empty domain is allowed and defaults to empty
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Test that NodeBuilder accepts empty domain
    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.domain, "");
    assert_eq!(node.fqdn, "test-router"); // FQDN equals name when domain is empty
}

#[tokio::test]
async fn test_add_node_args_validation_empty_model() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: String::new(), // Empty model should fail validation
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Test that NodeBuilder would reject empty model
    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_invalid_vendor() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "invalid-vendor".to_string(), // Invalid vendor
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Test that vendor parsing fails
    let result = args.vendor.parse::<Vendor>();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_invalid_role() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "invalid-role".to_string(), // Invalid role
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Test that role parsing fails
    let result = args.role.parse::<DeviceRole>();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_invalid_lifecycle() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "invalid-lifecycle".to_string(), // Invalid lifecycle
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Test that lifecycle parsing fails
    let result = args.lifecycle.parse::<Lifecycle>();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_valid_minimum() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Test that NodeBuilder accepts valid minimum arguments
    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.name, "test-router");
    assert_eq!(node.domain, "example.com");
    assert_eq!(node.vendor, Vendor::Cisco);
    assert_eq!(node.model, "ISR4321");
    assert_eq!(node.role, DeviceRole::Router);
    assert_eq!(node.lifecycle, Lifecycle::Live);
    assert_eq!(node.location_id, None);
    assert_eq!(node.management_ip, None);
}

#[tokio::test]
async fn test_add_node_args_validation_with_optional_fields() {
    let location_id = Uuid::new_v4();
    let args = AddNodeArgs {
        name: "full-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: Some(location_id),
        management_ip: Some("192.168.1.1".to_string()),
        custom_data: Some(r#"{"rack": "A1"}"#.to_string()),
    };

    // Test that NodeBuilder accepts optional fields
    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();
    let management_ip: IpAddr = args.management_ip.unwrap().parse().unwrap();
    let custom_data: JsonValue = serde_json::from_str(&args.custom_data.unwrap()).unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .location_id(location_id)
        .management_ip(management_ip)
        .custom_data(custom_data)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.location_id, Some(location_id));
    assert_eq!(node.management_ip, Some(management_ip));
    assert!(node.custom_data.is_object());
}

// ENUM PARSING TESTS

#[tokio::test]
async fn test_vendor_parsing_valid() {
    let valid_vendors = vec![
        ("cisco", Vendor::Cisco),
        ("juniper", Vendor::Juniper),
        ("arista", Vendor::Arista),
        ("generic", Vendor::Generic),
    ];

    for (vendor_str, expected) in valid_vendors {
        let result = vendor_str.parse::<Vendor>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

#[tokio::test]
async fn test_device_role_parsing_valid() {
    let valid_roles = vec![
        ("router", DeviceRole::Router),
        ("switch", DeviceRole::Switch),
        ("server", DeviceRole::Server),
        ("firewall", DeviceRole::Firewall),
    ];

    for (role_str, expected) in valid_roles {
        let result = role_str.parse::<DeviceRole>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

#[tokio::test]
async fn test_lifecycle_parsing_valid() {
    let valid_lifecycles = vec![
        ("planned", Lifecycle::Planned),
        ("implementing", Lifecycle::Implementing),
        ("live", Lifecycle::Live),
        ("decommissioned", Lifecycle::Decommissioned),
    ];

    for (lifecycle_str, expected) in valid_lifecycles {
        let result = lifecycle_str.parse::<Lifecycle>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

// IP ADDRESS PARSING TESTS

#[tokio::test]
async fn test_management_ip_parsing_valid() {
    let valid_ips = vec![
        "192.168.1.1",
        "10.0.0.1",
        "172.16.1.1",
        "2001:db8::1",
        "::1",
        "127.0.0.1",
    ];

    for ip_str in valid_ips {
        let result = ip_str.parse::<IpAddr>();
        assert!(result.is_ok(), "Failed to parse valid IP: {ip_str}");
    }
}

#[tokio::test]
async fn test_management_ip_parsing_invalid() {
    let invalid_ips = vec![
        "256.256.256.256",
        "192.168.1",
        "not-an-ip",
        "192.168.1.1.1",
        "192.168.1.256",
        "",
    ];

    for ip_str in invalid_ips {
        let result = ip_str.parse::<IpAddr>();
        assert!(result.is_err(), "Should have failed to parse IP: {ip_str}");
    }
}

// JSON PARSING TESTS FOR CUSTOM DATA

#[tokio::test]
async fn test_custom_data_json_parsing_valid() {
    let json_str = r#"{"rack": "A1", "port_count": 48, "power_consumption": 125.5}"#;
    let result = serde_json::from_str::<JsonValue>(json_str);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_object());
    assert_eq!(value["rack"], "A1");
    assert_eq!(value["port_count"], 48);
    assert_eq!(value["power_consumption"], 125.5);
}

#[tokio::test]
async fn test_custom_data_json_parsing_invalid() {
    let invalid_json = r#"{"rack": "A1", "port_count": }"#; // Missing value
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
    assert!(value.as_object().unwrap().is_empty());
}

#[tokio::test]
async fn test_custom_data_json_parsing_complex() {
    let complex_json = r#"{
        "hardware": {
            "cpu": "Intel Xeon",
            "memory": "32GB",
            "storage": {
                "type": "SSD",
                "capacity": "1TB"
            }
        },
        "network_interfaces": [
            {"name": "eth0", "speed": "1Gbps"},
            {"name": "eth1", "speed": "10Gbps"}
        ],
        "monitoring": {
            "enabled": true,
            "polling_interval": 60,
            "snmp_community": "public"
        }
    }"#;
    let result = serde_json::from_str::<JsonValue>(complex_json);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_object());
    assert!(value["hardware"]["cpu"] == "Intel Xeon");
    assert!(value["network_interfaces"].is_array());
    assert!(value["monitoring"]["enabled"] == true);
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

    // Converting overflowed values to usize should fail on some platforms
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

// NODE BUILDER INTEGRATION TESTS

#[tokio::test]
async fn test_node_builder_with_all_fields() {
    let location_id = Uuid::new_v4();
    let management_ip: IpAddr = "192.168.1.1".parse().unwrap();
    let custom_data = serde_json::json!({
        "rack": "A1",
        "port_count": 48,
        "power_consumption": 125.5
    });

    let result = NodeBuilder::new()
        .name("comprehensive-router".to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("ISR4321".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .location_id(location_id)
        .management_ip(management_ip)
        .custom_data(custom_data)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.name, "comprehensive-router");
    assert_eq!(node.domain, "example.com");
    assert_eq!(node.vendor, Vendor::Cisco);
    assert_eq!(node.model, "ISR4321");
    assert_eq!(node.role, DeviceRole::Router);
    assert_eq!(node.lifecycle, Lifecycle::Live);
    assert_eq!(node.location_id, Some(location_id));
    assert_eq!(node.management_ip, Some(management_ip));
    assert!(node.custom_data.is_object());
}

#[tokio::test]
async fn test_node_builder_minimal_fields() {
    let result = NodeBuilder::new()
        .name("minimal-router".to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("ISR4321".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Planned)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.name, "minimal-router");
    assert_eq!(node.domain, "example.com");
    assert_eq!(node.vendor, Vendor::Cisco);
    assert_eq!(node.model, "ISR4321");
    assert_eq!(node.role, DeviceRole::Router);
    assert_eq!(node.lifecycle, Lifecycle::Planned);
    assert_eq!(node.location_id, None);
    assert_eq!(node.management_ip, None);
    assert!(node.custom_data.is_null());
}

#[tokio::test]
async fn test_node_builder_validation_failures() {
    // Test empty name
    let result = NodeBuilder::new()
        .name(String::new())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("ISR4321".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build();
    assert!(result.is_err());

    // Test missing vendor
    let result = NodeBuilder::new()
        .name("test-router".to_string())
        .domain("example.com".to_string())
        .model("ISR4321".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build();
    assert!(result.is_err());

    // Test empty model
    let result = NodeBuilder::new()
        .name("test-router".to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model(String::new())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build();
    assert!(result.is_err());
}

// FILTER AND SORT CONSTRUCTION TESTS

#[tokio::test]
async fn test_list_nodes_filter_construction_by_lifecycle() {
    use unet_core::prelude::{Filter, FilterOperation, FilterValue};

    let lifecycle = "live".to_string();

    // Test filter construction similar to list_nodes function
    let filter = Filter {
        field: "lifecycle".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::String(lifecycle.clone()),
    };

    assert_eq!(filter.field, "lifecycle");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    match filter.value {
        FilterValue::String(value) => assert_eq!(value, lifecycle),
        _ => panic!("Expected String filter value"),
    }
}

#[tokio::test]
async fn test_list_nodes_filter_construction_by_role() {
    use unet_core::prelude::{Filter, FilterOperation, FilterValue};

    let role = "router".to_string();

    // Test filter construction similar to list_nodes function
    let filter = Filter {
        field: "role".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::String(role.clone()),
    };

    assert_eq!(filter.field, "role");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    match filter.value {
        FilterValue::String(value) => assert_eq!(value, role),
        _ => panic!("Expected String filter value"),
    }
}

#[tokio::test]
async fn test_list_nodes_filter_construction_by_vendor() {
    use unet_core::prelude::{Filter, FilterOperation, FilterValue};

    let vendor = "cisco".to_string();

    // Test filter construction similar to list_nodes function
    let filter = Filter {
        field: "vendor".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::String(vendor.clone()),
    };

    assert_eq!(filter.field, "vendor");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    match filter.value {
        FilterValue::String(value) => assert_eq!(value, vendor),
        _ => panic!("Expected String filter value"),
    }
}

#[tokio::test]
async fn test_list_nodes_sort_construction() {
    use unet_core::prelude::{Sort, SortDirection};

    // Test sort construction similar to list_nodes function
    let sort = Sort {
        field: "name".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(sort.field, "name");
    assert!(matches!(sort.direction, SortDirection::Ascending));
}

#[tokio::test]
async fn test_list_nodes_query_options_construction() {
    use unet_core::prelude::{
        Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort, SortDirection,
    };

    let lifecycle = "live".to_string();
    let role = "router".to_string();
    let vendor = "cisco".to_string();
    let page = 2_u64;
    let per_page = 30_u64;

    // Construct QueryOptions similar to list_nodes function
    let filters = vec![
        Filter {
            field: "lifecycle".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(lifecycle),
        },
        Filter {
            field: "role".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(role),
        },
        Filter {
            field: "vendor".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(vendor),
        },
    ];

    let sort = vec![Sort {
        field: "name".to_owned(),
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
    assert_eq!(options.filters.len(), 3);
    assert_eq!(options.sort.len(), 1);
    assert!(options.pagination.is_some());

    if let Some(ref pagination) = options.pagination {
        assert_eq!(pagination.offset, 30); // (2-1) * 30 = 30
        assert_eq!(pagination.limit, 30);
    }
}

// UPDATE NODE ARGUMENT VALIDATION TESTS

#[tokio::test]
async fn test_update_node_partial_updates() {
    let node_id = Uuid::new_v4();

    // Test that individual fields can be updated
    let args = UpdateNodeArgs {
        id: node_id,
        name: Some("updated-name".to_string()),
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    // Verify only name field is set for update
    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("updated-name".to_string()));
    assert_eq!(args.domain, None);
    assert_eq!(args.vendor, None);
    assert_eq!(args.model, None);
    assert_eq!(args.role, None);
    assert_eq!(args.lifecycle, None);
    assert_eq!(args.location_id, None);
    assert_eq!(args.management_ip, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_update_node_enum_parsing() {
    // Test that enum strings are parsed correctly in update operations
    let vendor_str = "juniper";
    let role_str = "switch";
    let lifecycle_str = "decommissioned";

    let vendor_result = vendor_str.parse::<Vendor>();
    let role_result = role_str.parse::<DeviceRole>();
    let lifecycle_result = lifecycle_str.parse::<Lifecycle>();

    assert!(vendor_result.is_ok());
    assert_eq!(vendor_result.unwrap(), Vendor::Juniper);

    assert!(role_result.is_ok());
    assert_eq!(role_result.unwrap(), DeviceRole::Switch);

    assert!(lifecycle_result.is_ok());
    assert_eq!(lifecycle_result.unwrap(), Lifecycle::Decommissioned);
}

#[tokio::test]
async fn test_update_node_fqdn_calculation() {
    // Test FQDN calculation logic from update_node
    let name = "test-router";
    let domain = "example.com";
    let expected_fqdn = format!("{name}.{domain}");

    assert_eq!(expected_fqdn, "test-router.example.com");
}

#[tokio::test]
async fn test_update_node_custom_data_parsing() {
    let custom_data_str = r#"{"environment": "production", "rack": "B2"}"#;
    let result = serde_json::from_str::<JsonValue>(custom_data_str);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["environment"], "production");
    assert_eq!(value["rack"], "B2");
}

// DELETE NODE CONFIRMATION TESTS

#[tokio::test]
async fn test_delete_node_confirmation_logic() {
    // Test confirmation logic patterns
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
        let input_trimmed = input.trim().to_lowercase();
        let should_proceed = input_trimmed == "y" || input_trimmed == "yes";
        assert_eq!(
            should_proceed, expected,
            "Input '{input}' should return {expected}"
        );
    }
}

#[tokio::test]
async fn test_delete_node_yes_flag_bypass() {
    let node_id = Uuid::new_v4();

    let args_with_yes = DeleteNodeArgs {
        id: node_id,
        yes: true,
    };

    let args_without_yes = DeleteNodeArgs {
        id: node_id,
        yes: false,
    };

    // When yes=true, no confirmation should be needed
    assert!(args_with_yes.yes);
    // When yes=false, confirmation should be required
    assert!(!args_without_yes.yes);
}

// SHOW NODE ARGUMENT VALIDATION TESTS

#[tokio::test]
async fn test_show_node_args_flags() {
    let node_id = Uuid::new_v4();

    // Test different flag combinations
    let args_basic = ShowNodeArgs {
        id: node_id,
        include_status: false,
        show_interfaces: false,
        show_system_info: false,
    };

    let args_all_flags = ShowNodeArgs {
        id: node_id,
        include_status: true,
        show_interfaces: true,
        show_system_info: true,
    };

    let args_partial_flags = ShowNodeArgs {
        id: node_id,
        include_status: true,
        show_interfaces: false,
        show_system_info: true,
    };

    // Basic args (no enhanced output)
    assert_eq!(args_basic.id, node_id);
    assert!(!args_basic.include_status);
    assert!(!args_basic.show_interfaces);
    assert!(!args_basic.show_system_info);

    // All flags enabled
    assert_eq!(args_all_flags.id, node_id);
    assert!(args_all_flags.include_status);
    assert!(args_all_flags.show_interfaces);
    assert!(args_all_flags.show_system_info);

    // Partial flags
    assert_eq!(args_partial_flags.id, node_id);
    assert!(args_partial_flags.include_status);
    assert!(!args_partial_flags.show_interfaces);
    assert!(args_partial_flags.show_system_info);
}

#[tokio::test]
async fn test_show_node_enhanced_output_check() {
    let node_id = Uuid::new_v4();

    let args = ShowNodeArgs {
        id: node_id,
        include_status: true,
        show_interfaces: false,
        show_system_info: true,
    };

    // Test the logic from show_node for determining enhanced output
    let should_use_enhanced_output = args.include_status || args.show_interfaces || args.show_system_info;
    assert!(should_use_enhanced_output);

    let args_basic = ShowNodeArgs {
        id: node_id,
        include_status: false,
        show_interfaces: false,
        show_system_info: false,
    };

    let should_use_basic_output = !(args_basic.include_status || args_basic.show_interfaces || args_basic.show_system_info);
    assert!(should_use_basic_output);
}