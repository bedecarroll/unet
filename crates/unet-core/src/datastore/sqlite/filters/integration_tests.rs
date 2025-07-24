//! Integration tests for combined filtering and sorting operations

#[cfg(test)]
mod tests {
    use crate::datastore::sqlite::filters::{
        apply_link_filters, apply_link_sorting, apply_location_filters, apply_location_sorting,
        apply_node_filters, apply_node_sorting,
    };
    use crate::datastore::types::{Filter, FilterOperation, FilterValue, Sort, SortDirection};
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
