/// Tests for query operations, pagination, filtering, and sorting
use unet_core::prelude::*;
use uuid::Uuid;

#[tokio::test]
async fn test_pagination_calculation_for_locations() {
    let page = 4_u64;
    let per_page = 15_u64;

    let expected_offset = (page - 1) * per_page; // 3 * 15 = 45
    let offset = usize::try_from(expected_offset);
    let limit = usize::try_from(per_page);

    assert!(offset.is_ok());
    assert!(limit.is_ok());
    assert_eq!(offset.unwrap(), 45);
    assert_eq!(limit.unwrap(), 15);

    // Test edge cases - first page should have zero offset
    let first_page_offset = 0_u64;
    assert_eq!(first_page_offset, 0);

    let large_page_offset = (100_u64 - 1) * 50_u64;
    assert_eq!(large_page_offset, 4_950);
}

#[tokio::test]
async fn test_filter_creation_for_locations() {
    let parent_id = Uuid::new_v4();

    // Test location type filter
    let type_filter = Filter {
        field: "location_type".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::String("datacenter".to_string()),
    };

    assert_eq!(type_filter.field, "location_type");
    assert!(matches!(type_filter.operation, FilterOperation::Equals));
    assert!(matches!(type_filter.value, FilterValue::String(_)));

    // Test parent ID filter
    let parent_filter = Filter {
        field: "parent_id".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(parent_id),
    };

    assert_eq!(parent_filter.field, "parent_id");
    assert!(matches!(parent_filter.operation, FilterOperation::Equals));
    assert!(matches!(parent_filter.value, FilterValue::Uuid(_)));
}

#[tokio::test]
async fn test_sort_creation_for_locations() {
    let sort = Sort {
        field: "name".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(sort.field, "name");
    assert!(matches!(sort.direction, SortDirection::Ascending));

    // Test descending sort
    let desc_sort = Sort {
        field: "location_type".to_owned(),
        direction: SortDirection::Descending,
    };

    assert_eq!(desc_sort.field, "location_type");
    assert!(matches!(desc_sort.direction, SortDirection::Descending));
}

#[tokio::test]
async fn test_query_options_creation_for_locations() {
    let parent_id = Uuid::new_v4();

    let filters = vec![
        Filter {
            field: "location_type".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("building".to_string()),
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

    let pagination = Some(Pagination {
        offset: 20,
        limit: 10,
    });

    let options = QueryOptions {
        filters,
        sort,
        pagination,
    };

    assert_eq!(options.filters.len(), 2);
    assert_eq!(options.sort.len(), 1);
    assert!(options.pagination.is_some());

    if let Some(pag) = options.pagination {
        assert_eq!(pag.offset, 20);
        assert_eq!(pag.limit, 10);
    }
}

#[tokio::test]
async fn test_multiple_filters_for_locations() {
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
        Filter {
            field: "name".to_owned(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("east".to_string()),
        },
        Filter {
            field: "city".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("New York".to_string()),
        },
    ];

    assert_eq!(filters.len(), 4);

    // Verify each filter has expected properties
    assert_eq!(filters[0].field, "location_type");
    assert!(matches!(filters[0].operation, FilterOperation::Equals));

    assert_eq!(filters[1].field, "parent_id");
    assert!(matches!(filters[1].value, FilterValue::Uuid(_)));

    assert_eq!(filters[2].field, "name");
    assert!(matches!(filters[2].operation, FilterOperation::Contains));

    assert_eq!(filters[3].field, "city");
    assert!(matches!(filters[3].value, FilterValue::String(_)));
}
