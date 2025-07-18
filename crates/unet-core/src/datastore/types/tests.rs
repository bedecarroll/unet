//! Tests for datastore types

#[cfg(test)]
mod datastore_types_tests {
    use super::super::{
        BatchResult, DataStoreError, FilterValue, PagedResult, Pagination, QueryOptions,
    };

    use uuid::Uuid;

    #[test]
    fn test_datastore_error_display() {
        let err = DataStoreError::NotFound {
            entity_type: "Node".to_string(),
            id: "test-id".to_string(),
        };
        assert!(err.to_string().contains("Node"));
        assert!(err.to_string().contains("test-id"));
    }

    #[test]
    fn test_datastore_error_connection() {
        let err = DataStoreError::ConnectionError {
            message: "Connection failed".to_string(),
        };
        assert!(err.to_string().contains("Connection failed"));
    }

    #[test]
    fn test_datastore_error_validation() {
        let err = DataStoreError::ValidationError {
            message: "Required field".to_string(),
        };
        assert!(err.to_string().contains("Required field"));
    }

    #[test]
    fn test_datastore_error_constraint_violation() {
        let err = DataStoreError::ConstraintViolation {
            message: "Duplicate name".to_string(),
        };
        assert!(err.to_string().contains("Duplicate name"));
    }

    #[test]
    fn test_datastore_error_timeout() {
        let err = DataStoreError::Timeout { seconds: 30 };
        assert!(err.to_string().contains("30"));
    }

    #[test]
    fn test_datastore_error_transaction_failed() {
        let err = DataStoreError::TransactionError {
            message: "Transaction aborted".to_string(),
        };
        assert!(err.to_string().contains("Transaction aborted"));
    }

    #[test]
    fn test_batch_result_new() {
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
    fn test_batch_result_operations() {
        let batch = BatchResult {
            success_count: 1,
            error_count: 1,
            errors: vec![(
                0,
                DataStoreError::ValidationError {
                    message: "Test error".to_string(),
                },
            )],
        };

        assert_eq!(batch.success_count, 1);
        assert_eq!(batch.error_count, 1);
        assert_eq!(batch.errors.len(), 1);
        assert!(batch.errors[0].1.to_string().contains("Test error"));
    }

    #[test]
    fn test_batch_result_complete_success() {
        let batch = BatchResult {
            success_count: 2,
            error_count: 0,
            errors: Vec::new(),
        };

        assert_eq!(batch.success_count, 2);
        assert_eq!(batch.error_count, 0);
        assert!(batch.errors.is_empty());
    }

    #[test]
    fn test_batch_result_complete_failure() {
        let batch = BatchResult {
            success_count: 0,
            error_count: 2,
            errors: vec![
                (
                    0,
                    DataStoreError::ValidationError {
                        message: "Error 1".to_string(),
                    },
                ),
                (
                    1,
                    DataStoreError::ValidationError {
                        message: "Error 2".to_string(),
                    },
                ),
            ],
        };

        assert_eq!(batch.success_count, 0);
        assert_eq!(batch.error_count, 2);
        assert_eq!(batch.errors.len(), 2);
    }

    #[test]
    fn test_paged_result_new() {
        let items = vec![1, 2, 3];
        let pagination = Pagination::new(10, 0).unwrap();
        let paged = PagedResult::new(items.clone(), 100, Some(&pagination));

        assert_eq!(paged.items, items);
        assert_eq!(paged.total_count, 100);
        assert_eq!(paged.page_size, 10);
        assert_eq!(paged.page, 1);
        assert_eq!(paged.total_pages, 10);
        assert!(paged.has_next);
        assert!(!paged.has_previous);
    }

    #[test]
    fn test_paged_result_last_page() {
        let items = vec![1, 2, 3];
        let pagination = Pagination::new(10, 90).unwrap();
        let paged = PagedResult::new(items, 100, Some(&pagination));

        assert_eq!(paged.page, 10);
        assert_eq!(paged.total_pages, 10);
        assert!(!paged.has_next);
        assert!(paged.has_previous);
    }

    #[test]
    fn test_paged_result_single_page() {
        let items = vec![1, 2, 3];
        let paged = PagedResult::new(items, 3, None);

        assert_eq!(paged.total_pages, 1);
        assert_eq!(paged.page, 1);
        assert!(!paged.has_next);
        assert!(!paged.has_previous);
    }

    #[test]
    fn test_query_options_default() {
        let opts = QueryOptions::default();
        assert!(opts.filters.is_empty());
        assert!(opts.sort.is_empty());
        assert!(opts.pagination.is_none());
    }

    #[test]
    fn test_query_options_with_data() {
        let pagination = Pagination::new(20, 10).unwrap();
        let opts = QueryOptions {
            filters: vec![],
            sort: vec![],
            pagination: Some(pagination),
        };

        assert!(opts.pagination.is_some());
        assert_eq!(opts.pagination.as_ref().unwrap().limit, 20);
        assert_eq!(opts.pagination.as_ref().unwrap().offset, 10);
        assert!(opts.filters.is_empty());
        assert!(opts.sort.is_empty());
    }

    #[test]
    fn test_filter_value_string() {
        let value = FilterValue::String("test".to_string());
        match value {
            FilterValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_filter_value_uuid() {
        let uuid = Uuid::new_v4();
        let value = FilterValue::Uuid(uuid);
        match value {
            FilterValue::Uuid(u) => assert_eq!(u, uuid),
            _ => panic!("Expected Uuid variant"),
        }
    }

    #[test]
    fn test_filter_value_integer() {
        let value = FilterValue::Integer(42);
        match value {
            FilterValue::Integer(i) => assert_eq!(i, 42),
            _ => panic!("Expected Integer variant"),
        }
    }

    #[test]
    fn test_filter_value_boolean() {
        let value = FilterValue::Boolean(true);
        match value {
            FilterValue::Boolean(b) => assert!(b),
            _ => panic!("Expected Boolean variant"),
        }
    }
}
