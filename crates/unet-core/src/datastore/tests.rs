//! Tests for the datastore module

use crate::datastore::helpers::{
    filter_contains, filter_equals_string, filter_equals_uuid, sort_asc, sort_desc,
};
use crate::datastore::types::{
    BatchResult, DataStoreError, Filter, FilterOperation, FilterValue, PagedResult, Pagination,
    QueryOptions, SortDirection,
};
use uuid::Uuid;

#[test]
fn test_pagination_new() {
    // Valid pagination
    let pagination = Pagination::new(10, 0).unwrap();
    assert_eq!(pagination.limit, 10);
    assert_eq!(pagination.offset, 0);

    // Invalid limit (0)
    assert!(Pagination::new(0, 0).is_err());

    // Invalid limit (too large)
    assert!(Pagination::new(1001, 0).is_err());

    // Valid edge cases
    assert!(Pagination::new(1, 0).is_ok());
    assert!(Pagination::new(1000, 0).is_ok());
}

#[test]
fn test_pagination_page() {
    // Valid page pagination
    let pagination = Pagination::page(1, 10).unwrap();
    assert_eq!(pagination.limit, 10);
    assert_eq!(pagination.offset, 0);

    let pagination = Pagination::page(3, 20).unwrap();
    assert_eq!(pagination.limit, 20);
    assert_eq!(pagination.offset, 40);

    // Invalid page (0)
    assert!(Pagination::page(0, 10).is_err());
}

#[test]
fn test_paged_result_new() {
    let items = vec![1, 2, 3];

    // Without pagination
    let result = PagedResult::new(items.clone(), 3, None);
    assert_eq!(result.items, items);
    assert_eq!(result.total_count, 3);
    assert_eq!(result.page_size, 3);
    assert_eq!(result.page, 1);
    assert_eq!(result.total_pages, 1);
    assert!(!result.has_next);
    assert!(!result.has_previous);

    // With pagination - first page
    let pagination = Pagination::new(2, 0).unwrap();
    let result = PagedResult::new(vec![1, 2], 5, Some(&pagination));
    assert_eq!(result.items, vec![1, 2]);
    assert_eq!(result.total_count, 5);
    assert_eq!(result.page_size, 2);
    assert_eq!(result.page, 1);
    assert_eq!(result.total_pages, 3);
    assert!(result.has_next);
    assert!(!result.has_previous);

    // With pagination - middle page
    let pagination = Pagination::new(2, 2).unwrap();
    let result = PagedResult::new(vec![3, 4], 5, Some(&pagination));
    assert_eq!(result.page, 2);
    assert!(result.has_next);
    assert!(result.has_previous);

    // With pagination - last page
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

#[test]
fn test_batch_result() {
    let batch = BatchResult {
        success_count: 0,
        error_count: 0,
        errors: Vec::new(),
    };
    assert_eq!(batch.success_count, 0);
    assert_eq!(batch.error_count, 0);
    assert!(batch.errors.is_empty());
}

#[test]
fn test_datastore_error_display() {
    let error = DataStoreError::NotFound {
        entity_type: "Node".to_string(),
        id: "test-id".to_string(),
    };
    assert!(error.to_string().contains("Node"));
    assert!(error.to_string().contains("test-id"));

    let error = DataStoreError::ValidationError {
        message: "Invalid data".to_string(),
    };
    assert!(error.to_string().contains("Invalid data"));

    let error = DataStoreError::Timeout { seconds: 30 };
    assert!(error.to_string().contains("30"));
}

#[test]
fn test_datastore_error_all_variants() {
    // Test all DataStoreError variants for completeness
    let not_found = DataStoreError::NotFound {
        entity_type: "Link".to_string(),
        id: "link-123".to_string(),
    };
    assert!(not_found.to_string().contains("Link"));
    assert!(not_found.to_string().contains("link-123"));

    let validation = DataStoreError::ValidationError {
        message: "Name too long".to_string(),
    };
    assert!(validation.to_string().contains("Name too long"));

    let timeout = DataStoreError::Timeout { seconds: 45 };
    assert!(timeout.to_string().contains("45"));

    let connection = DataStoreError::ConnectionError {
        message: "DB offline".to_string(),
    };
    assert!(connection.to_string().contains("DB offline"));

    let internal = DataStoreError::InternalError {
        message: "Unexpected error".to_string(),
    };
    assert!(internal.to_string().contains("Unexpected error"));

    let constraint = DataStoreError::ConstraintViolation {
        message: "Name already exists for node-456".to_string(),
    };
    assert!(constraint.to_string().contains("Name already exists"));
    assert!(constraint.to_string().contains("node-456"));

    let transaction = DataStoreError::TransactionError {
        message: "Transaction rolled back".to_string(),
    };
    assert!(transaction.to_string().contains("Transaction rolled back"));
}

#[test]
fn test_filter_operation_variants() {
    // Test all FilterOperation variants
    let ops = [
        FilterOperation::Equals,
        FilterOperation::Contains,
        FilterOperation::StartsWith,
        FilterOperation::EndsWith,
        FilterOperation::GreaterThan,
        FilterOperation::LessThan,
        FilterOperation::GreaterThanOrEqual,
        FilterOperation::LessThanOrEqual,
    ];

    // Just ensure they can be created and are different
    for (i, op1) in ops.iter().enumerate() {
        for (j, op2) in ops.iter().enumerate() {
            if i == j {
                assert_eq!(op1, op2);
            } else {
                assert_ne!(op1, op2);
            }
        }
    }
}

#[test]
fn test_filter_value_variants() {
    let string_val = FilterValue::String("test".to_string());
    let uuid_val = FilterValue::Uuid(uuid::Uuid::new_v4());
    let int_val = FilterValue::Integer(42);
    let bool_val = FilterValue::Boolean(true);

    // Test that different types are not equal
    assert_ne!(string_val, int_val);
    assert_ne!(uuid_val, bool_val);
    assert_ne!(int_val, bool_val);

    // Test same values are equal
    assert_eq!(
        FilterValue::String("test".to_string()),
        FilterValue::String("test".to_string())
    );
    assert_eq!(FilterValue::Integer(42), FilterValue::Integer(42));
    assert_eq!(FilterValue::Boolean(true), FilterValue::Boolean(true));
    assert_eq!(FilterValue::Boolean(false), FilterValue::Boolean(false));

    // Test different values of same type are not equal
    assert_ne!(
        FilterValue::String("test".to_string()),
        FilterValue::String("other".to_string())
    );
    assert_ne!(FilterValue::Integer(42), FilterValue::Integer(43));
    assert_ne!(FilterValue::Boolean(true), FilterValue::Boolean(false));
}

#[test]
fn test_pagination_edge_cases() {
    // Test maximum valid limit
    let pagination = Pagination::new(1000, 0).unwrap();
    assert_eq!(pagination.limit, 1000);
    assert_eq!(pagination.offset, 0);

    // Test minimum valid limit
    let pagination = Pagination::new(1, 0).unwrap();
    assert_eq!(pagination.limit, 1);
    assert_eq!(pagination.offset, 0);

    // Test large offset
    let pagination = Pagination::new(10, 1_000_000).unwrap();
    assert_eq!(pagination.limit, 10);
    assert_eq!(pagination.offset, 1_000_000);

    // Test page calculation with large numbers
    let pagination = Pagination::page(100, 50).unwrap();
    assert_eq!(pagination.limit, 50);
    assert_eq!(pagination.offset, 4950); // (100-1) * 50
}

#[test]
fn test_paged_result_edge_cases() {
    // Test empty result
    let result: PagedResult<String> = PagedResult::new(vec![], 0, None);
    assert_eq!(result.items.len(), 0);
    assert_eq!(result.total_count, 0);
    assert_eq!(result.page_size, 0);
    assert_eq!(result.page, 1);
    assert_eq!(result.total_pages, 1); // Empty result still has 1 page
    assert!(!result.has_next);
    assert!(!result.has_previous);

    // Test single item
    let result = PagedResult::new(vec!["item1"], 1, None);
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.total_count, 1);
    assert_eq!(result.page_size, 1);
    assert_eq!(result.page, 1);
    assert_eq!(result.total_pages, 1);
    assert!(!result.has_next);
    assert!(!result.has_previous);

    // Test large dataset with pagination
    let pagination = Pagination::new(10, 90).unwrap(); // Page 10 of 10-item pages
    let result = PagedResult::new(vec![1, 2, 3, 4, 5], 95, Some(&pagination));
    assert_eq!(result.page, 10); // (90/10) + 1
    assert_eq!(result.total_pages, 10); // ceil(95/10)
    assert!(!result.has_next);
    assert!(result.has_previous);
}

#[test]
fn test_query_options_with_multiple_filters_and_sorts() {
    let mut options = QueryOptions::default();

    // Add multiple filters
    options.filters.push(filter_equals_string("name", "test"));
    options
        .filters
        .push(filter_contains("description", "network"));

    // Add multiple sorts
    options.sort.push(sort_asc("name"));
    options.sort.push(sort_desc("created_at"));

    // Add pagination
    options.pagination = Some(Pagination::new(25, 50).unwrap());

    assert_eq!(options.filters.len(), 2);
    assert_eq!(options.sort.len(), 2);
    assert!(options.pagination.is_some());

    // Test first filter
    assert_eq!(options.filters[0].field, "name");
    assert!(matches!(
        options.filters[0].operation,
        FilterOperation::Equals
    ));

    // Test second filter
    assert_eq!(options.filters[1].field, "description");
    assert!(matches!(
        options.filters[1].operation,
        FilterOperation::Contains
    ));

    // Test sorts
    assert_eq!(options.sort[0].field, "name");
    assert!(matches!(
        options.sort[0].direction,
        SortDirection::Ascending
    ));
    assert_eq!(options.sort[1].field, "created_at");
    assert!(matches!(
        options.sort[1].direction,
        SortDirection::Descending
    ));
}

#[test]
fn test_batch_result_with_errors() {
    let batch = BatchResult {
        success_count: 2,
        error_count: 1,
        errors: vec![(
            0,
            DataStoreError::ValidationError {
                message: "Validation failed".to_string(),
            },
        )],
    };
    assert_eq!(batch.success_count, 2);
    assert_eq!(batch.error_count, 1);
    assert_eq!(batch.errors.len(), 1);
    assert_eq!(batch.errors[0].0, 0);
    match &batch.errors[0].1 {
        DataStoreError::ValidationError { message } => {
            assert_eq!(message, "Validation failed");
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_batch_result_all_success() {
    let batch = BatchResult {
        success_count: 5,
        error_count: 0,
        errors: Vec::new(),
    };
    assert_eq!(batch.success_count, 5);
    assert_eq!(batch.error_count, 0);
    assert!(batch.errors.is_empty());
}

#[test]
fn test_sort_direction_variants() {
    let asc = SortDirection::Ascending;
    let desc = SortDirection::Descending;

    assert_eq!(asc, SortDirection::Ascending);
    assert_eq!(desc, SortDirection::Descending);
    assert_ne!(asc, desc);
}

#[test]
fn test_filter_with_boolean_values() {
    let true_filter = Filter {
        field: "enabled".to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Boolean(true),
    };

    let false_filter = Filter {
        field: "enabled".to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Boolean(false),
    };

    assert_ne!(true_filter.value, false_filter.value);
    assert_eq!(true_filter.field, false_filter.field);
    assert_eq!(true_filter.operation, false_filter.operation);
}

#[test]
fn test_filter_with_uuid_values() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();

    let filter1 = Filter {
        field: "id".to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(uuid1),
    };

    let filter2 = Filter {
        field: "id".to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(uuid2),
    };

    assert_ne!(filter1.value, filter2.value);

    // Test same UUID values are equal
    let filter3 = Filter {
        field: "id".to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(uuid1),
    };

    assert_eq!(filter1.value, filter3.value);
}

#[test]
fn test_paged_result_calculation_edge_cases() {
    // Test when total_count is exactly divisible by page_size
    let pagination = Pagination::new(10, 0).unwrap();
    let result = PagedResult::new(vec![1, 2, 3, 4, 5], 100, Some(&pagination));
    assert_eq!(result.total_pages, 10);
    assert_eq!(result.page, 1);
    assert!(result.has_next);
    assert!(!result.has_previous);

    // Test when total_count is not divisible by page_size
    let pagination = Pagination::new(10, 0).unwrap();
    let result = PagedResult::new(vec![1, 2, 3], 103, Some(&pagination));
    assert_eq!(result.total_pages, 11);

    // Test when items returned is less than page_size (partial page)
    let pagination = Pagination::new(10, 100).unwrap();
    let result = PagedResult::new(vec![1, 2, 3], 103, Some(&pagination));
    assert_eq!(result.page, 11);
    assert_eq!(result.items.len(), 3);
    assert!(!result.has_next);
    assert!(result.has_previous);
}

#[test]
fn test_complex_filter_combinations() {
    let mut options = QueryOptions::default();

    // Add filters with different operations
    options.filters.push(Filter {
        field: "name".to_string(),
        operation: FilterOperation::StartsWith,
        value: FilterValue::String("test".to_string()),
    });

    options.filters.push(Filter {
        field: "description".to_string(),
        operation: FilterOperation::EndsWith,
        value: FilterValue::String("suffix".to_string()),
    });

    options.filters.push(Filter {
        field: "count".to_string(),
        operation: FilterOperation::GreaterThan,
        value: FilterValue::Integer(10),
    });

    options.filters.push(Filter {
        field: "limit".to_string(),
        operation: FilterOperation::LessThanOrEqual,
        value: FilterValue::Integer(100),
    });

    assert_eq!(options.filters.len(), 4);

    // Verify each filter operation
    assert!(matches!(
        options.filters[0].operation,
        FilterOperation::StartsWith
    ));
    assert!(matches!(
        options.filters[1].operation,
        FilterOperation::EndsWith
    ));
    assert!(matches!(
        options.filters[2].operation,
        FilterOperation::GreaterThan
    ));
    assert!(matches!(
        options.filters[3].operation,
        FilterOperation::LessThanOrEqual
    ));
}

#[test]
fn test_pagination_with_zero_total_count() {
    let pagination = Pagination::new(10, 0).unwrap();
    let result: PagedResult<String> = PagedResult::new(vec![], 0, Some(&pagination));

    assert_eq!(result.total_count, 0);
    assert_eq!(result.total_pages, 0);
    assert_eq!(result.page, 1);
    assert!(!result.has_next);
    assert!(!result.has_previous);
    assert!(result.items.is_empty());
}
