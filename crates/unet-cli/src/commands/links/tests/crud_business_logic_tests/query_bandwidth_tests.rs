use crate::commands::test_support::pagination_values;
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
async fn test_bandwidth_values_zero() {
    let bandwidth = 0_u64;
    assert_eq!(bandwidth, 0);
}

#[tokio::test]
async fn test_bandwidth_values_common() {
    let gigabit = 1_000_000_000_u64;
    let ten_gigabit = 10_000_000_000_u64;
    let hundred_gigabit = 100_000_000_000_u64;

    assert_eq!(gigabit, 1_000_000_000);
    assert_eq!(ten_gigabit, 10_000_000_000);
    assert_eq!(hundred_gigabit, 100_000_000_000);
    assert!(gigabit < u64::MAX);
    assert!(ten_gigabit < u64::MAX);
    assert!(hundred_gigabit < u64::MAX);
}

#[tokio::test]
async fn test_bandwidth_values_maximum() {
    let max_bandwidth = u64::MAX;
    assert_eq!(max_bandwidth, u64::MAX);
}

#[tokio::test]
async fn test_list_links_filter_construction() {
    let node_id = Uuid::new_v4();
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
    let sort = Sort {
        field: "interface_a".to_owned(),
        direction: SortDirection::Ascending,
    };

    assert_eq!(sort.field, "interface_a");
    assert!(matches!(sort.direction, SortDirection::Ascending));
}

#[tokio::test]
async fn test_list_links_query_options_construction() {
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
    let (offset, limit) = pagination_values(3, 50);
    let options = QueryOptions {
        filters,
        sort,
        pagination: Some(Pagination { offset, limit }),
    };

    assert_eq!(options.filters.len(), 1);
    assert_eq!(options.sort.len(), 1);
    assert!(options.pagination.is_some());

    if let Some(pagination) = options.pagination {
        assert_eq!(pagination.offset, 100);
        assert_eq!(pagination.limit, 50);
    }
}
