//! Tests for `nodes` entity

#[cfg(test)]
mod tests {
    use super::super::super::nodes::*;
    use sea_orm::entity::prelude::*;
    use serde_json;

    #[test]
    fn test_node_model_creation() {
        let node = Model {
            id: "test-node-id".to_string(),
            name: "test-node".to_string(),
            fqdn: Some("test-node.example.com".to_string()),
            domain: Some("example.com".to_string()),
            vendor: "Cisco".to_string(),
            model: "ASR1000".to_string(),
            role: "router".to_string(),
            lifecycle: "live".to_string(),
            serial_number: Some("FXS12345".to_string()),
            asset_tag: Some("ASSET-001".to_string()),
            location_id: Some("loc-001".to_string()),
            management_ip: Some("192.168.1.1".to_string()),
            description: Some("Test router".to_string()),
            custom_data: Some(r#"{"key": "value"}"#.to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(node.id, "test-node-id");
        assert_eq!(node.name, "test-node");
        assert_eq!(node.fqdn, Some("test-node.example.com".to_string()));
        assert_eq!(node.vendor, "Cisco");
        assert_eq!(node.model, "ASR1000");
        assert_eq!(node.role, "router");
        assert_eq!(node.lifecycle, "live");
    }

    #[test]
    fn test_node_model_serialization() {
        let node = Model {
            id: "test-node-id".to_string(),
            name: "test-node".to_string(),
            fqdn: None,
            domain: None,
            vendor: "Cisco".to_string(),
            model: "ASR1000".to_string(),
            role: "router".to_string(),
            lifecycle: "live".to_string(),
            serial_number: None,
            asset_tag: None,
            location_id: None,
            management_ip: None,
            description: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(node, deserialized);
    }

    #[test]
    fn test_node_relation_definitions() {
        let location_rel = Relation::Location.def();
        assert_eq!(location_rel.from_tbl, Entity.table_ref());

        let node_status_rel = Relation::NodeStatus.def();
        assert_eq!(node_status_rel.from_tbl, Entity.table_ref());

        let polling_tasks_rel = Relation::PollingTasks.def();
        assert_eq!(polling_tasks_rel.from_tbl, Entity.table_ref());
    }

    #[test]
    fn test_node_relation_enum_iter() {
        use sea_orm::Iterable;
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 5);
        // Test that we can iterate over relations
        for relation in relations {
            let _rel_def = relation.def();
        }
    }

    #[test]
    fn test_node_related_implementations() {
        use sea_orm::entity::Related;

        // Test Related<locations::Entity> implementation
        let location_relation = <Entity as Related<super::super::super::locations::Entity>>::to();
        assert_eq!(location_relation.from_tbl, Entity.table_ref());

        // Test Related<node_status::Entity> implementation
        let node_status_relation =
            <Entity as Related<super::super::super::node_status::Entity>>::to();
        assert_eq!(node_status_relation.from_tbl, Entity.table_ref());

        // Test Related<polling_tasks::Entity> implementation
        let polling_tasks_relation =
            <Entity as Related<super::super::super::polling_tasks::Entity>>::to();
        assert_eq!(polling_tasks_relation.from_tbl, Entity.table_ref());
    }

    #[test]
    fn test_active_model_behavior() {
        // Test that ActiveModelBehavior is implemented
        let _active_model = ActiveModel::new();
        // ActiveModelBehavior provides default implementations, so we just test it compiles
    }
}
