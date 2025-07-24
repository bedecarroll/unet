//! Tests for link filtering and sorting functionality

#[cfg(test)]
mod tests {
    use crate::datastore::sqlite::filters::{apply_link_filters, apply_link_sorting};
    use crate::datastore::types::{
        DataStoreError, Filter, FilterOperation, FilterValue, Sort, SortDirection,
    };
    use crate::entities::links;
    use sea_orm::{EntityTrait, Select};

    /// Create a base link query for testing
    fn create_link_query() -> Select<links::Entity> {
        links::Entity::find()
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
}
