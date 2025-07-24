/// Filter and sorting helper functions for `SQLite` queries
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod link_filter_tests;
#[cfg(test)]
mod location_filter_tests;
#[cfg(test)]
mod node_filter_tests;
#[cfg(test)]
mod tests;

use super::super::types::{
    DataStoreError, DataStoreResult, Filter, FilterValue, Sort, SortDirection,
};
use crate::entities::{links, locations, nodes};
use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};

/// Apply filters to a node query
pub fn apply_node_filters(
    mut query: sea_orm::Select<nodes::Entity>,
    filters: &[Filter],
) -> DataStoreResult<sea_orm::Select<nodes::Entity>> {
    for filter in filters {
        match filter.field.as_str() {
            "name" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(nodes::Column::Name.contains(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Name filter must be a string".to_string(),
                    });
                }
            },
            "vendor" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(nodes::Column::Vendor.eq(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Vendor filter must be a string".to_string(),
                    });
                }
            },
            "role" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(nodes::Column::Role.eq(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Role filter must be a string".to_string(),
                    });
                }
            },
            "lifecycle" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(nodes::Column::Lifecycle.eq(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Lifecycle filter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(DataStoreError::ValidationError {
                    message: format!("Unsupported filter field: {}", filter.field),
                });
            }
        }
    }
    Ok(query)
}

/// Apply sorting to a node query
pub fn apply_node_sorting(
    mut query: sea_orm::Select<nodes::Entity>,
    sorts: &[Sort],
) -> DataStoreResult<sea_orm::Select<nodes::Entity>> {
    for sort in sorts {
        match sort.field.as_str() {
            "name" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(nodes::Column::Name),
                    SortDirection::Descending => query.order_by_desc(nodes::Column::Name),
                };
            }
            "created_at" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(nodes::Column::CreatedAt),
                    SortDirection::Descending => query.order_by_desc(nodes::Column::CreatedAt),
                };
            }
            _ => {
                return Err(DataStoreError::ValidationError {
                    message: format!("Unsupported sort field: {}", sort.field),
                });
            }
        }
    }
    Ok(query)
}

/// Apply filters to a location query
pub fn apply_location_filters(
    mut query: sea_orm::Select<locations::Entity>,
    filters: &[Filter],
) -> DataStoreResult<sea_orm::Select<locations::Entity>> {
    for filter in filters {
        match filter.field.as_str() {
            "name" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(locations::Column::Name.contains(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Name filter must be a string".to_string(),
                    });
                }
            },
            "location_type" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(locations::Column::LocationType.eq(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Location type filter must be a string".to_string(),
                    });
                }
            },
            "parent_id" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(locations::Column::ParentId.eq(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Parent ID filter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(DataStoreError::ValidationError {
                    message: format!("Unsupported filter field: {}", filter.field),
                });
            }
        }
    }
    Ok(query)
}

/// Apply sorting to a location query
pub fn apply_location_sorting(
    mut query: sea_orm::Select<locations::Entity>,
    sorts: &[Sort],
) -> DataStoreResult<sea_orm::Select<locations::Entity>> {
    for sort in sorts {
        match sort.field.as_str() {
            "name" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(locations::Column::Name),
                    SortDirection::Descending => query.order_by_desc(locations::Column::Name),
                };
            }
            "path" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(locations::Column::Path),
                    SortDirection::Descending => query.order_by_desc(locations::Column::Path),
                };
            }
            "created_at" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(locations::Column::CreatedAt),
                    SortDirection::Descending => query.order_by_desc(locations::Column::CreatedAt),
                };
            }
            _ => {
                return Err(DataStoreError::ValidationError {
                    message: format!("Unsupported sort field: {}", sort.field),
                });
            }
        }
    }
    Ok(query)
}

/// Apply filters to a link query  
pub fn apply_link_filters(
    mut query: sea_orm::Select<links::Entity>,
    filters: &[Filter],
) -> DataStoreResult<sea_orm::Select<links::Entity>> {
    for filter in filters {
        match filter.field.as_str() {
            "name" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(links::Column::Name.contains(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Name filter must be a string".to_string(),
                    });
                }
            },
            "node_a_id" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(links::Column::NodeAId.eq(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Node A ID filter must be a string".to_string(),
                    });
                }
            },
            "node_b_id" => match &filter.value {
                FilterValue::String(s) => {
                    query = query.filter(links::Column::NodeBId.eq(s));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Node B ID filter must be a string".to_string(),
                    });
                }
            },
            "is_internet_circuit" => match &filter.value {
                FilterValue::Boolean(b) => {
                    let value = i32::from(*b);
                    query = query.filter(links::Column::IsInternetCircuit.eq(value));
                }
                _ => {
                    return Err(DataStoreError::ValidationError {
                        message: "Is internet circuit filter must be a boolean".to_string(),
                    });
                }
            },
            _ => {
                return Err(DataStoreError::ValidationError {
                    message: format!("Unsupported filter field: {}", filter.field),
                });
            }
        }
    }
    Ok(query)
}

/// Apply sorting to a link query
pub fn apply_link_sorting(
    mut query: sea_orm::Select<links::Entity>,
    sorts: &[Sort],
) -> DataStoreResult<sea_orm::Select<links::Entity>> {
    for sort in sorts {
        match sort.field.as_str() {
            "name" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(links::Column::Name),
                    SortDirection::Descending => query.order_by_desc(links::Column::Name),
                };
            }
            "interface_a" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(links::Column::InterfaceA),
                    SortDirection::Descending => query.order_by_desc(links::Column::InterfaceA),
                };
            }
            "created_at" => {
                query = match sort.direction {
                    SortDirection::Ascending => query.order_by_asc(links::Column::CreatedAt),
                    SortDirection::Descending => query.order_by_desc(links::Column::CreatedAt),
                };
            }
            _ => {
                return Err(DataStoreError::ValidationError {
                    message: format!("Unsupported sort field: {}", sort.field),
                });
            }
        }
    }
    Ok(query)
}
