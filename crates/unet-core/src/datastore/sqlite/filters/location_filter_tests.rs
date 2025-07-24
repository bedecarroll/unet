//! Tests for location filtering and sorting functionality

#[cfg(test)]
mod tests {
    use crate::datastore::sqlite::filters::{apply_location_filters, apply_location_sorting};
    use crate::datastore::types::{
        DataStoreError, Filter, FilterOperation, FilterValue, Sort, SortDirection,
    };
    use crate::entities::locations;
    use sea_orm::{EntityTrait, Select};

    /// Create a base location query for testing
    fn create_location_query() -> Select<locations::Entity> {
        locations::Entity::find()
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
}
