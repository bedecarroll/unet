//! Helper functions for creating common query options

use uuid::Uuid;

use super::types::{Filter, FilterOperation, FilterValue, Sort, SortDirection};

/// Creates a filter for exact string match
#[must_use]
pub fn filter_equals_string(field: &str, value: &str) -> Filter {
    Filter {
        field: field.to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::String(value.to_string()),
    }
}

/// Creates a filter for UUID match
#[must_use]
pub fn filter_equals_uuid(field: &str, value: Uuid) -> Filter {
    Filter {
        field: field.to_string(),
        operation: FilterOperation::Equals,
        value: FilterValue::Uuid(value),
    }
}

/// Creates a filter for string contains (case-insensitive)
#[must_use]
pub fn filter_contains(field: &str, value: &str) -> Filter {
    Filter {
        field: field.to_string(),
        operation: FilterOperation::Contains,
        value: FilterValue::String(value.to_string()),
    }
}

/// Creates ascending sort by field
#[must_use]
pub fn sort_asc(field: &str) -> Sort {
    Sort {
        field: field.to_string(),
        direction: SortDirection::Ascending,
    }
}

/// Creates descending sort by field
#[must_use]
pub fn sort_desc(field: &str) -> Sort {
    Sort {
        field: field.to_string(),
        direction: SortDirection::Descending,
    }
}
