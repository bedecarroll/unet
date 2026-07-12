/// Tests for query and filtering operations in link commands
use unet_core::prelude::*;
use uuid::Uuid;

#[tokio::test]
async fn test_pagination_calculation_for_links() {
    // Test pagination offset calculation
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
async fn test_filter_creation_for_links() {
    let node_id = Uuid::new_v4();

    let filter = Filter {
        field: "node_a_id".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(node_id),
    };

    assert_eq!(filter.field, "node_a_id");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    assert!(matches!(filter.value, FilterValue::Uuid(_)));
}

#[tokio::test]
async fn test_sort_creation_for_links() {
    let sort = Sort {
        field: "interface_a".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(sort.field, "interface_a");
    assert!(matches!(sort.direction, SortDirection::Ascending));
}

#[tokio::test]
async fn test_query_options_creation_for_links() {
    let node_id = Uuid::new_v4();

    let filters = vec![Filter {
        field: "node_a_id".to_owned(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(node_id),
    }];

    let sort = vec![Sort {
        field: "interface_a".to_owned(),
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
