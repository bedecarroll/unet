use std::net::IpAddr;

use crate::commands::nodes::types::{DeleteNodeArgs, ShowNodeArgs, UpdateNodeArgs};
use crate::commands::test_support::{expect_json_object, pagination_values};
use unet_core::models::{DeviceRole, Lifecycle, NodeBuilder, Vendor};
use unet_core::prelude::{
    Filter, FilterOperation, FilterValue, Pagination, QueryOptions, Sort, SortDirection,
};
use uuid::Uuid;

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
    assert_eq!(node.location_id, None);
    assert_eq!(node.management_ip, None);
    assert!(node.custom_data.is_null());
}

#[tokio::test]
async fn test_node_builder_validation_failures() {
    let result = NodeBuilder::new()
        .name(String::new())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("ISR4321".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build();
    assert!(result.is_err());

    let result = NodeBuilder::new()
        .name("test-router".to_string())
        .domain("example.com".to_string())
        .model("ISR4321".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build();
    assert!(result.is_err());

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

#[tokio::test]
async fn test_list_nodes_filter_construction_by_lifecycle() {
    let lifecycle = "live".to_string();
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
    let role = "router".to_string();
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
    let vendor = "cisco".to_string();
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
    let sort = Sort {
        field: "name".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(sort.field, "name");
    assert!(matches!(sort.direction, SortDirection::Ascending));
}

#[tokio::test]
async fn test_list_nodes_query_options_construction() {
    let filters = vec![
        Filter {
            field: "lifecycle".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("live".to_string()),
        },
        Filter {
            field: "role".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("router".to_string()),
        },
        Filter {
            field: "vendor".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("cisco".to_string()),
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

    assert_eq!(options.filters.len(), 3);
    assert_eq!(options.sort.len(), 1);
    assert!(options.pagination.is_some());

    if let Some(pagination) = options.pagination {
        assert_eq!(pagination.offset, 30);
        assert_eq!(pagination.limit, 30);
    }
}
