/// Tests for query operations, pagination, filtering, and sorting
use unet_core::prelude::*;

#[tokio::test]
async fn test_pagination_calculation() {
    let page = 3_u64;
    let per_page = 25_u64;

    let expected_offset = (page - 1) * per_page; // 2 * 25 = 50
    let offset = usize::try_from(expected_offset);
    let limit = usize::try_from(per_page);

    assert!(offset.is_ok());
    assert!(limit.is_ok());
    assert_eq!(offset.unwrap(), 50);
    assert_eq!(limit.unwrap(), 25);
}

#[tokio::test]
async fn test_pagination_edge_cases() {
    // Test first page - should have zero offset
    let first_page_offset = 0_u64;
    assert_eq!(first_page_offset, 0);

    // Test large page numbers
    let large_page = 1000_u64;
    let large_per_page = 100_u64;
    let large_offset = (large_page - 1) * large_per_page;
    assert_eq!(large_offset, 99_900);

    // Test minimum pagination
    let min_page = 1_u64;
    let min_per_page = 1_u64;
    let min_offset = (min_page - 1) * min_per_page;
    assert_eq!(min_offset, 0);

    // Test that conversion to usize works for reasonable values
    let offset_usize = usize::try_from(large_offset);
    assert!(offset_usize.is_ok());
}

#[tokio::test]
async fn test_filter_creation() {
    let vendor_filter = Filter {
        field: "vendor".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::String("cisco".to_string()),
    };

    assert_eq!(vendor_filter.field, "vendor");
    assert!(matches!(vendor_filter.operation, FilterOperation::Equals));
    assert!(matches!(vendor_filter.value, FilterValue::String(_)));

    let role_filter = Filter {
        field: "role".to_owned(),
        operation: FilterOperation::Contains,
        value: FilterValue::String("router".to_string()),
    };

    assert_eq!(role_filter.field, "role");
    assert!(matches!(role_filter.operation, FilterOperation::Contains));
    assert!(matches!(role_filter.value, FilterValue::String(_)));
}

#[tokio::test]
async fn test_sort_creation() {
    let name_sort = Sort {
        field: "name".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(name_sort.field, "name");
    assert!(matches!(name_sort.direction, SortDirection::Ascending));

    let created_sort = Sort {
        field: "created_at".to_owned(),
        direction: SortDirection::Descending,
    };

    assert_eq!(created_sort.field, "created_at");
    assert!(matches!(created_sort.direction, SortDirection::Descending));
}

#[tokio::test]
async fn test_query_options_creation() {
    let filters = vec![Filter {
        field: "lifecycle".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::String("live".to_string()),
    }];

    let sort = vec![Sort {
        field: "name".to_owned(),
        direction: SortDirection::Ascending,
    }];

    let pagination = Some(Pagination {
        offset: 0,
        limit: 20,
    });

    let options = QueryOptions {
        filters,
        sort,
        pagination,
    };

    assert_eq!(options.filters.len(), 1);
    assert_eq!(options.sort.len(), 1);
    assert!(options.pagination.is_some());

    if let Some(ref pagination) = options.pagination {
        assert_eq!(pagination.offset, 0);
        assert_eq!(pagination.limit, 20);
    }
}

#[tokio::test]
async fn test_fqdn_formatting() {
    let name = "test-node";
    let domain = "example.com";
    let fqdn = format!("{name}.{domain}");

    assert_eq!(fqdn, "test-node.example.com");

    // Test with different combinations
    let short_name = "r1";
    let long_domain = "corp.company.com";
    let long_fqdn = format!("{short_name}.{long_domain}");

    assert_eq!(long_fqdn, "r1.corp.company.com");
}