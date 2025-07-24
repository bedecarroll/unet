//! Tests for datastore filter functions
//!
//! Contains comprehensive tests for node, location, and link filtering and sorting
//! functionality using the `SQLite` query builders.

#[cfg(test)]
mod filter_tests {
    use crate::datastore::sqlite::filters::{
        apply_link_filters, apply_link_sorting, apply_location_filters, apply_location_sorting,
        apply_node_filters, apply_node_sorting,
    };
    use crate::datastore::types::{
        DataStoreError, Filter, FilterOperation, FilterValue, Sort, SortDirection,
    };
    use crate::entities::{links, locations, nodes};
    use sea_orm::{EntityTrait, Select};

    /// Create a base node query for testing
    fn create_node_query() -> Select<nodes::Entity> {
        nodes::Entity::find()
    }

    /// Create a base location query for testing
    fn create_location_query() -> Select<locations::Entity> {
        locations::Entity::find()
    }

    /// Create a base link query for testing
    fn create_link_query() -> Select<links::Entity> {
        links::Entity::find()
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

    // Location Filter Tests

    #[test]
    fn test_apply_location_filters_name_string_succeeds() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("datacenter".to_string()),
        }];

        let result = apply_location_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_location_filters_name_non_string_fails() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::Integer(123),
        }];

        let result = apply_location_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Name filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_location_filters_location_type_string_succeeds() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "location_type".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("datacenter".to_string()),
        }];

        let result = apply_location_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_location_filters_location_type_non_string_fails() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "location_type".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(true),
        }];

        let result = apply_location_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Location type filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_location_filters_parent_id_string_succeeds() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "parent_id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("parent-id".to_string()),
        }];

        let result = apply_location_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_location_filters_parent_id_non_string_fails() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "parent_id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Integer(456),
        }];

        let result = apply_location_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Parent ID filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_location_filters_unsupported_field_fails() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "invalid_field".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("value".to_string()),
        }];

        let result = apply_location_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Unsupported filter field: invalid_field");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    // Location Sorting Tests

    #[test]
    fn test_apply_location_sorting_name_ascending_succeeds() {
        let query = create_location_query();
        let sorts = vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_location_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_location_sorting_path_descending_succeeds() {
        let query = create_location_query();
        let sorts = vec![Sort {
            field: "path".to_string(),
            direction: SortDirection::Descending,
        }];

        let result = apply_location_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_location_sorting_created_at_ascending_succeeds() {
        let query = create_location_query();
        let sorts = vec![Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_location_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_location_sorting_unsupported_field_fails() {
        let query = create_location_query();
        let sorts = vec![Sort {
            field: "invalid_sort_field".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_location_sorting(query, &sorts);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Unsupported sort field: invalid_sort_field");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    // Link Filter Tests

    #[test]
    fn test_apply_link_filters_name_string_succeeds() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("connection".to_string()),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_filters_name_non_string_fails() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::Boolean(false),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Name filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_link_filters_node_a_id_string_succeeds() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "node_a_id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("node-a-id".to_string()),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_filters_node_a_id_non_string_fails() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "node_a_id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Integer(789),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Node A ID filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_link_filters_node_b_id_string_succeeds() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "node_b_id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("node-b-id".to_string()),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_filters_node_b_id_non_string_fails() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "node_b_id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(true),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Node B ID filter must be a string");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_link_filters_is_internet_circuit_boolean_succeeds() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "is_internet_circuit".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(true),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_filters_is_internet_circuit_non_boolean_fails() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "is_internet_circuit".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("true".to_string()),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Is internet circuit filter must be a boolean");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_apply_link_filters_unsupported_field_fails() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "unknown_field".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("value".to_string()),
        }];

        let result = apply_link_filters(query, &filters);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Unsupported filter field: unknown_field");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    // Link Sorting Tests

    #[test]
    fn test_apply_link_sorting_name_ascending_succeeds() {
        let query = create_link_query();
        let sorts = vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_link_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_sorting_name_descending_succeeds() {
        let query = create_link_query();
        let sorts = vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Descending,
        }];

        let result = apply_link_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_sorting_created_at_ascending_succeeds() {
        let query = create_link_query();
        let sorts = vec![Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_link_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_sorting_created_at_descending_succeeds() {
        let query = create_link_query();
        let sorts = vec![Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Descending,
        }];

        let result = apply_link_sorting(query, &sorts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_link_sorting_unsupported_field_fails() {
        let query = create_link_query();
        let sorts = vec![Sort {
            field: "bad_field".to_string(),
            direction: SortDirection::Ascending,
        }];

        let result = apply_link_sorting(query, &sorts);
        assert!(result.is_err());
        match result.unwrap_err() {
            DataStoreError::ValidationError { message } => {
                assert_eq!(message, "Unsupported sort field: bad_field");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    // Integration Tests

    #[test]
    fn test_combined_filtering_and_sorting_nodes() {
        let query = create_node_query();
        let filters = vec![Filter {
            field: "vendor".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("cisco".to_string()),
        }];
        let sorts = vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }];

        let filtered_query = apply_node_filters(query, &filters);
        assert!(filtered_query.is_ok());

        let sorted_query = apply_node_sorting(filtered_query.unwrap(), &sorts);
        assert!(sorted_query.is_ok());
    }

    #[test]
    fn test_combined_filtering_and_sorting_locations() {
        let query = create_location_query();
        let filters = vec![Filter {
            field: "location_type".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("datacenter".to_string()),
        }];
        let sorts = vec![Sort {
            field: "path".to_string(),
            direction: SortDirection::Descending,
        }];

        let filtered_query = apply_location_filters(query, &filters);
        assert!(filtered_query.is_ok());

        let sorted_query = apply_location_sorting(filtered_query.unwrap(), &sorts);
        assert!(sorted_query.is_ok());
    }

    #[test]
    fn test_combined_filtering_and_sorting_links() {
        let query = create_link_query();
        let filters = vec![Filter {
            field: "is_internet_circuit".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Boolean(false),
        }];
        let sorts = vec![Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Descending,
        }];

        let filtered_query = apply_link_filters(query, &filters);
        assert!(filtered_query.is_ok());

        let sorted_query = apply_link_sorting(filtered_query.unwrap(), &sorts);
        assert!(sorted_query.is_ok());
    }
}
