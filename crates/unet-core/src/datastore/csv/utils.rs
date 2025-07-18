//! Utility functions for CSV datastore

use super::super::types::{Filter, FilterOperation, FilterValue};

/// Applies filters to a collection
pub fn apply_filters<T, F>(items: Vec<T>, filters: &[Filter], field_getter: F) -> Vec<T>
where
    F: Fn(&T, &str) -> Option<FilterValue>,
{
    if filters.is_empty() {
        return items;
    }

    items
        .into_iter()
        .filter(|item| {
            filters.iter().all(|filter| {
                let field_value = field_getter(item, &filter.field);
                field_value.map_or(
                    matches!(filter.operation, FilterOperation::IsNull),
                    |value| matches_filter(&value, filter),
                )
            })
        })
        .collect()
}

/// Checks if a value matches a filter
pub fn matches_filter(value: &FilterValue, filter: &Filter) -> bool {
    use FilterOperation::{Contains, EndsWith, Equals, IsNotNull, NotEquals, StartsWith};
    use FilterValue as FV;

    match (&filter.operation, value, &filter.value) {
        (Equals, FV::String(a), FV::String(b)) => a == b,
        (Equals, FV::Uuid(a), FV::Uuid(b)) => a == b,
        (NotEquals, a, b) => !matches_filter(
            a,
            &Filter {
                field: filter.field.clone(),
                operation: Equals,
                value: b.clone(),
            },
        ),
        (Contains, FV::String(a), FV::String(b)) => a.to_lowercase().contains(&b.to_lowercase()),
        (StartsWith, FV::String(a), FV::String(b)) => {
            a.to_lowercase().starts_with(&b.to_lowercase())
        }
        (EndsWith, FV::String(a), FV::String(b)) => a.to_lowercase().ends_with(&b.to_lowercase()),
        (IsNotNull, _, _) => true,
        _ => false, // IsNull is handled in apply_filters, everything else is unsupported
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    struct TestItem {
        name: String,
        id: Uuid,
        description: Option<String>,
    }

    fn test_field_getter(item: &TestItem, field: &str) -> Option<FilterValue> {
        match field {
            "name" => Some(FilterValue::String(item.name.clone())),
            "id" => Some(FilterValue::Uuid(item.id)),
            "description" => item
                .description
                .as_ref()
                .map(|d| FilterValue::String(d.clone())),
            _ => None,
        }
    }

    fn create_test_items() -> Vec<TestItem> {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        vec![
            TestItem {
                name: "Alice".to_string(),
                id: id1,
                description: Some("Developer".to_string()),
            },
            TestItem {
                name: "Bob".to_string(),
                id: id2,
                description: None,
            },
            TestItem {
                name: "Charlie".to_string(),
                id: id3,
                description: Some("Manager".to_string()),
            },
        ]
    }

    #[test]
    fn test_apply_filters_empty_filters() {
        let items = create_test_items();
        let filters = vec![];
        let result = apply_filters(items.clone(), &filters, test_field_getter);
        assert_eq!(result.len(), items.len());
    }

    #[test]
    fn test_apply_filters_equals_string() {
        let items = create_test_items();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("Alice".to_string()),
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Alice");
    }

    #[test]
    fn test_apply_filters_equals_uuid() {
        let items = create_test_items();
        let target_id = items[0].id;
        let filters = vec![Filter {
            field: "id".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Uuid(target_id),
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, target_id);
    }

    #[test]
    fn test_apply_filters_not_equals() {
        let items = create_test_items();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::NotEquals,
            value: FilterValue::String("Alice".to_string()),
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|item| item.name != "Alice"));
    }

    #[test]
    fn test_apply_filters_contains() {
        let items = create_test_items();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("ha".to_string()),
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Charlie");
    }

    #[test]
    fn test_apply_filters_starts_with() {
        let items = create_test_items();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::StartsWith,
            value: FilterValue::String("Al".to_string()),
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Alice");
    }

    #[test]
    fn test_apply_filters_ends_with() {
        let items = create_test_items();
        let filters = vec![Filter {
            field: "name".to_string(),
            operation: FilterOperation::EndsWith,
            value: FilterValue::String("ie".to_string()),
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Charlie");
    }

    #[test]
    fn test_apply_filters_is_null() {
        let items = create_test_items();
        let filters = vec![Filter {
            field: "description".to_string(),
            operation: FilterOperation::IsNull,
            value: FilterValue::String(String::new()), // Value is ignored for IsNull
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Bob");
    }

    #[test]
    fn test_apply_filters_is_not_null() {
        let items = create_test_items();
        let filters = vec![Filter {
            field: "description".to_string(),
            operation: FilterOperation::IsNotNull,
            value: FilterValue::String(String::new()), // Value is ignored for IsNotNull
        }];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|item| item.description.is_some()));
    }

    #[test]
    fn test_apply_filters_multiple() {
        let items = create_test_items();
        let filters = vec![
            Filter {
                field: "description".to_string(),
                operation: FilterOperation::IsNotNull,
                value: FilterValue::String(String::new()),
            },
            Filter {
                field: "name".to_string(),
                operation: FilterOperation::StartsWith,
                value: FilterValue::String("A".to_string()),
            },
        ];
        let result = apply_filters(items, &filters, test_field_getter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Alice");
    }

    #[test]
    fn test_matches_filter_string_equals() {
        let filter = Filter {
            field: "test".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String("hello".to_string()),
        };
        let value = FilterValue::String("hello".to_string());
        assert!(matches_filter(&value, &filter));

        let value2 = FilterValue::String("world".to_string());
        assert!(!matches_filter(&value2, &filter));
    }

    #[test]
    fn test_matches_filter_uuid_equals() {
        let uuid = Uuid::new_v4();
        let filter = Filter {
            field: "test".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::Uuid(uuid),
        };
        let value = FilterValue::Uuid(uuid);
        assert!(matches_filter(&value, &filter));

        let value2 = FilterValue::Uuid(Uuid::new_v4());
        assert!(!matches_filter(&value2, &filter));
    }

    #[test]
    fn test_matches_filter_case_insensitive() {
        let filter = Filter {
            field: "test".to_string(),
            operation: FilterOperation::Contains,
            value: FilterValue::String("HELLO".to_string()),
        };
        let value = FilterValue::String("hello world".to_string());
        assert!(matches_filter(&value, &filter));
    }

    #[test]
    fn test_matches_filter_unsupported_operation() {
        let filter = Filter {
            field: "test".to_string(),
            operation: FilterOperation::IsNull, // This returns false in matches_filter
            value: FilterValue::String("test".to_string()),
        };
        let value = FilterValue::String("test".to_string());
        assert!(!matches_filter(&value, &filter));
    }
}
