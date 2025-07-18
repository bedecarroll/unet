//! Tests for `locations` entity

#[cfg(test)]
mod tests {
    use super::super::super::locations::*;
    use sea_orm::entity::prelude::*;
    use serde_json;

    #[test]
    fn test_location_model_creation() {
        let location = Model {
            id: "loc-001".to_string(),
            name: "Main Datacenter".to_string(),
            location_type: "datacenter".to_string(),
            path: "/datacenter/main".to_string(),
            parent_id: None,
            description: Some("Primary datacenter facility".to_string()),
            address: Some("123 Main St, City, State 12345".to_string()),
            coordinates: Some("40.7128,-74.0060".to_string()),
            custom_data: Some(r#"{"region": "us-east"}"#.to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(location.id, "loc-001");
        assert_eq!(location.name, "Main Datacenter");
        assert_eq!(location.location_type, "datacenter");
        assert_eq!(location.path, "/datacenter/main");
        assert_eq!(location.parent_id, None);
    }

    #[test]
    fn test_location_model_with_parent() {
        let location = Model {
            id: "loc-002".to_string(),
            name: "Rack 1".to_string(),
            location_type: "rack".to_string(),
            path: "/datacenter/main/floor1/rack1".to_string(),
            parent_id: Some("loc-001".to_string()),
            description: Some("Equipment rack".to_string()),
            address: None,
            coordinates: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(location.parent_id, Some("loc-001".to_string()));
        assert_eq!(location.location_type, "rack");
    }

    #[test]
    fn test_location_model_serialization() {
        let location = Model {
            id: "loc-001".to_string(),
            name: "Main Datacenter".to_string(),
            location_type: "datacenter".to_string(),
            path: "/datacenter/main".to_string(),
            parent_id: None,
            description: None,
            address: None,
            coordinates: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&location).unwrap();
        let deserialized: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(location, deserialized);
    }

    #[test]
    fn test_location_relation_definitions() {
        let nodes_rel = Relation::Nodes.def();
        assert_eq!(nodes_rel.from_tbl, Entity.table_ref());
    }

    #[test]
    fn test_location_relation_enum_iter() {
        use sea_orm::Iterable;
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 1);
        // Test that we can iterate over relations
        for relation in relations {
            let _rel_def = relation.def();
        }
    }
}
