//! `SeaORM` Entity for Locations table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Physical or logical location entity for organizing network devices
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "location")]
pub struct Model {
    /// Unique identifier for the location
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Human-readable location name
    pub name: String,
    /// Type of location (datacenter, building, floor, rack, etc.)
    pub location_type: String,
    /// Hierarchical path for location tree navigation
    pub path: String,
    /// Foreign key to parent location for hierarchy
    pub parent_id: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Physical address or location details
    pub address: Option<String>,
    /// GPS coordinates or position data
    pub coordinates: Option<String>,
    /// JSON string for custom attributes
    pub custom_data: Option<String>,
    /// Timestamp when record was created
    pub created_at: String,
    /// Timestamp when record was last updated
    pub updated_at: String,
}

/// Database relations for location entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relation to nodes located at this location
    #[sea_orm(has_many = "super::nodes::Entity")]
    Nodes,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Iterable;

    #[test]
    fn test_location_model_creation() {
        let location = Model {
            id: "loc-123".to_string(),
            name: "Test Location".to_string(),
            location_type: "datacenter".to_string(),
            path: "/datacenter/test".to_string(),
            parent_id: Some("loc-parent".to_string()),
            description: Some("Test description".to_string()),
            address: Some("123 Test St".to_string()),
            coordinates: Some("40.7128,-74.0060".to_string()),
            custom_data: Some(r#"{"zone": "A"}"#.to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(location.id, "loc-123");
        assert_eq!(location.name, "Test Location");
        assert_eq!(location.location_type, "datacenter");
        assert_eq!(location.path, "/datacenter/test");
        assert_eq!(location.parent_id, Some("loc-parent".to_string()));
        assert_eq!(location.description, Some("Test description".to_string()));
        assert_eq!(location.address, Some("123 Test St".to_string()));
        assert_eq!(location.coordinates, Some("40.7128,-74.0060".to_string()));
        assert_eq!(location.custom_data, Some(r#"{"zone": "A"}"#.to_string()));
    }

    #[test]
    fn test_location_model_minimal() {
        let location = Model {
            id: "loc-minimal".to_string(),
            name: "Minimal Location".to_string(),
            location_type: "room".to_string(),
            path: "/room/minimal".to_string(),
            parent_id: None,
            description: None,
            address: None,
            coordinates: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(location.id, "loc-minimal");
        assert_eq!(location.name, "Minimal Location");
        assert_eq!(location.location_type, "room");
        assert_eq!(location.path, "/room/minimal");
        assert_eq!(location.parent_id, None);
        assert_eq!(location.description, None);
        assert_eq!(location.address, None);
        assert_eq!(location.coordinates, None);
        assert_eq!(location.custom_data, None);
    }

    #[test]
    fn test_location_model_debug() {
        let location = Model {
            id: "loc-debug".to_string(),
            name: "Debug Test".to_string(),
            location_type: "rack".to_string(),
            path: "/rack/debug".to_string(),
            parent_id: None,
            description: None,
            address: None,
            coordinates: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let debug_str = format!("{location:?}");
        assert!(debug_str.contains("loc-debug"));
        assert!(debug_str.contains("Debug Test"));
        assert!(debug_str.contains("rack"));
    }

    #[test]
    fn test_location_model_equality() {
        let location1 = Model {
            id: "loc-eq".to_string(),
            name: "Equal Test".to_string(),
            location_type: "floor".to_string(),
            path: "/floor/equal".to_string(),
            parent_id: None,
            description: None,
            address: None,
            coordinates: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        // Test equality with itself
        assert_eq!(location1, location1);

        let location3 = Model {
            id: "loc-eq-diff".to_string(),
            name: "Different Name".to_string(),
            location_type: "floor".to_string(),
            path: "/floor/equal".to_string(),
            parent_id: None,
            description: None,
            address: None,
            coordinates: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };
        assert_ne!(location1, location3);
    }

    #[test]
    fn test_location_model_serialization() {
        let location = Model {
            id: "loc-ser".to_string(),
            name: "Serialize Test".to_string(),
            location_type: "zone".to_string(),
            path: "/zone/serialize".to_string(),
            parent_id: Some("parent-zone".to_string()),
            description: Some("Serialization test".to_string()),
            address: None,
            coordinates: None,
            custom_data: Some(r#"{"test": true}"#.to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let serialized = serde_json::to_string(&location).expect("Failed to serialize");
        let deserialized: Model = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(location.id, deserialized.id);
        assert_eq!(location.name, deserialized.name);
        assert_eq!(location.location_type, deserialized.location_type);
        assert_eq!(location.path, deserialized.path);
        assert_eq!(location.parent_id, deserialized.parent_id);
        assert_eq!(location.custom_data, deserialized.custom_data);
    }

    #[test]
    fn test_relation_enum() {
        // Test that Relation enum can be iterated
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 1);
        // Verify the relation can be formatted
        let relation_name = format!("{:?}", relations[0]);
        assert!(relation_name.contains("Nodes"));
    }

    #[test]
    fn test_relation_enum_debug() {
        let relation = Relation::Nodes;
        let debug_str = format!("{relation:?}");
        assert!(debug_str.contains("Nodes"));
    }

    #[test]
    fn test_relation_enum_clone_copy() {
        let relation1 = Relation::Nodes;
        let relation2 = relation1;
        assert!(matches!(relation1, Relation::Nodes));
        assert!(matches!(relation2, Relation::Nodes));
    }

    #[test]
    fn test_nodes_relation() {
        use crate::entities::nodes;
        use sea_orm::Related;

        // Test that the relation to nodes entity is properly defined
        let _relation_def = <Entity as Related<nodes::Entity>>::to();

        // The fact that this compiles and runs means the relation is properly implemented
    }
}
