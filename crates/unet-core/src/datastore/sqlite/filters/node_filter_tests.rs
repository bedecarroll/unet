//! Tests for node filtering and sorting functionality

#[cfg(test)]
mod tests {
    use crate::datastore::sqlite::filters::{apply_node_filters, apply_node_sorting};
    use crate::datastore::types::{
        DataStoreError, Filter, FilterOperation, FilterValue, Sort, SortDirection,
    };
    use crate::entities::nodes;
    use sea_orm::{EntityTrait, Select};

    /// Create a base node query for testing
    fn create_node_query() -> Select<nodes::Entity> {
        nodes::Entity::find()
    }

    // Node Filter Tests

    #[test]
    fn test_apply_node_filters_name_string_succeeds() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("router".to_string()),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_filters_name_non_string_fails() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::Boolean(true),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Name filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_node_filters_vendor_string_succeeds() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "vendor".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("cisco".to_string()),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_filters_vendor_non_string_fails() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "vendor".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Integer(123),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Vendor filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_node_filters_role_string_succeeds() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "role".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("router".to_string()),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_filters_role_non_string_fails() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "role".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(false),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Role filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_node_filters_lifecycle_string_succeeds() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "lifecycle".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("live".to_string()),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_filters_lifecycle_non_string_fails() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "lifecycle".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Integer(1),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Lifecycle filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_node_filters_unsupported_field_fails() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "unsupported_field".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("value".to_string()),
        }];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Unsupported filter field: unsupported_field");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_node_filters_multiple_filters_succeeds() {
        let query = create_node_query();
        let filters = vec![
            Filter {
                field: "name".to_string(),
                operation: FilterOperation::Contains,
                value: FilterValue::String("router".to_string()),
            },
            Filter {
                field: "vendor".to_string(),
                operation: FilterOperation::Equals,
                value: FilterValue::String("cisco".to_string()),
            },
        ];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_filters_empty_filters_succeeds() {
        let query = create_node_query();
        let filters = vec![];

        let result = apply_node_filters(query, &filters);
        assert!(result.is_ok());
    }

    // Node Sorting Tests

    #[test]
    fn test_apply_node_sorting_name_ascending_succeeds() {
        let query = create_node_query();
        let sorts = vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_node_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_sorting_name_descending_succeeds() {
        let query = create_node_query();
        let sorts = vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Descending,
        }];

        let result = apply_node_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_sorting_created_at_ascending_succeeds() {
        let query = create_node_query();
        let sorts = vec![Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_node_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_sorting_created_at_descending_succeeds() {
        let query = create_node_query();
        let sorts = vec![Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Descending,
        }];

        let result = apply_node_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_sorting_unsupported_field_fails() {
        let query = create_node_query();
        let sorts = vec![Sort {
            field: "unsupported_field".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_node_sorting(query, &sorts);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Unsupported sort field: unsupported_field");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_node_sorting_multiple_sorts_succeeds() {
        let query = create_node_query();
        let sorts = vec![
            Sort {
                field: "name".to_string(),
                direction: SortDirection::Ascending,
            },
            Sort {
                field: "created_at".to_string(),
                direction: SortDirection::Descending,
            },
        ];

        let result = apply_node_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_node_sorting_empty_sorts_succeeds() {
        let query = create_node_query();
        let sorts = vec![];

        let result = apply_node_sorting(query, &sorts);
        assert!(result.is_ok());
    }
}
