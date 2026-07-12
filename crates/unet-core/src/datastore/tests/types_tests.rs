//! Tests for datastore types - Pagination, `PagedResult`, `QueryOptions`, and helpers

use crate::datastore::helpers::{
    filter_contains, filter_equals_string, filter_equals_uuid, sort_asc, sort_desc,
};
use crate::datastore::types::{
    FilterOperation, FilterValue, PagedResult, Pagination, QueryOptions, SortDirection,
};
use uuid::Uuid;

#[test]
fn test_pagination_new() {
    let pagination = Pagination::new(10, 0).unwrap();
    assert_eq!(pagination.limit, 10);
    assert_eq!(pagination.offset, 0);
    assert!(Pagination::new(0, 0).is_err());
    assert!(Pagination::new(1_001, 0).is_err());
    assert!(Pagination::new(1, 0).is_ok());
    assert!(Pagination::new(1_000, 0).is_ok());
}

#[test]
fn test_pagination_page() {
    let pagination = Pagination::page(1, 10).unwrap();
    assert_eq!(pagination.limit, 10);
    assert_eq!(pagination.offset, 0);

    let pagination = Pagination::page(3, 20).unwrap();
    assert_eq!(pagination.limit, 20);
    assert_eq!(pagination.offset, 40);

    assert!(Pagination::page(0, 10).is_err());
}

#[test]
fn test_paged_result_new() {
    let items = vec![1, 2, 3];

    let result = PagedResult::new(items.clone(), 3, None);
    assert_eq!(result.items, items);
    assert_eq!(result.total_count, 3);
    assert_eq!(result.page_size, 3);
    assert_eq!(result.page, 1);
    assert_eq!(result.total_pages, 1);
    assert!(!result.has_next);
    assert!(!result.has_previous);

    let pagination = Pagination::new(2, 0).unwrap();
    let result = PagedResult::new(vec![1, 2], 5, Some(&pagination));
    assert_eq!(result.items, vec![1, 2]);
    assert_eq!(result.total_count, 5);
    assert_eq!(result.page_size, 2);
    assert_eq!(result.page, 1);
    assert_eq!(result.total_pages, 3);
    assert!(result.has_next);
    assert!(!result.has_previous);

    let pagination = Pagination::new(2, 2).unwrap();
    let result = PagedResult::new(vec![3, 4], 5, Some(&pagination));
    assert_eq!(result.page, 2);
    assert!(result.has_next);
    assert!(result.has_previous);

    let pagination = Pagination::new(2, 4).unwrap();
    let result = PagedResult::new(vec![5], 5, Some(&pagination));
    assert_eq!(result.page, 3);
    assert!(!result.has_next);
    assert!(result.has_previous);
}

#[test]
fn test_filter_helpers() {
    let filter = filter_equals_string("name", "test");
    assert_eq!(filter.field, "name");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    assert!(matches!(filter.value, FilterValue::String(ref s) if s == "test"));

    let uuid = Uuid::new_v4();
    let filter = filter_equals_uuid("id", uuid);
    assert_eq!(filter.field, "id");
    assert!(matches!(filter.operation, FilterOperation::Equals));
    assert!(matches!(filter.value, FilterValue::Uuid(u) if u == uuid));

    let filter = filter_contains("description", "partial");
    assert_eq!(filter.field, "description");
    assert!(matches!(filter.operation, FilterOperation::Contains));
    assert!(matches!(filter.value, FilterValue::String(ref s) if s == "partial"));
}

#[test]
fn test_sort_helpers() {
    let sort = sort_asc("name");
    assert_eq!(sort.field, "name");
    assert!(matches!(sort.direction, SortDirection::Ascending));

    let sort = sort_desc("created_at");
    assert_eq!(sort.field, "created_at");
    assert!(matches!(sort.direction, SortDirection::Descending));
}

#[test]
fn test_query_options_default() {
    let options = QueryOptions::default();
    assert!(options.filters.is_empty());
    assert!(options.sort.is_empty());
    assert!(options.pagination.is_none());
}
