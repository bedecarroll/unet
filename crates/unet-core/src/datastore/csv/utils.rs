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
