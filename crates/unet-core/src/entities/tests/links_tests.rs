//! Tests for `links` entity

#[cfg(test)]
mod tests {
    use super::super::super::links::*;
    use sea_orm::entity::prelude::*;
    use serde_json;

    #[test]
    fn test_link_model_creation() {
        let link = Model {
            id: "link-001".to_string(),
            name: "Router A to Router B".to_string(),
            node_a_id: "node-a".to_string(),
            interface_a: "GigabitEthernet0/0/0".to_string(),
            node_b_id: Some("node-b".to_string()),
            interface_b: Some("GigabitEthernet0/0/1".to_string()),
            capacity: Some(1_000_000_000),
            utilization: Some(0.75),
            is_internet_circuit: 0,
            circuit_id: None,
            provider: None,
            description: Some("Internal link".to_string()),
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(link.id, "link-001");
        assert_eq!(link.name, "Router A to Router B");
        assert_eq!(link.node_a_id, "node-a");
        assert_eq!(link.interface_a, "GigabitEthernet0/0/0");
        assert_eq!(link.node_b_id, Some("node-b".to_string()));
        assert_eq!(link.capacity, Some(1_000_000_000));
        assert!((link.utilization.unwrap() - 0.75).abs() < f64::EPSILON);
        assert_eq!(link.is_internet_circuit, 0);
    }

    #[test]
    fn test_link_model_internet_circuit() {
        let link = Model {
            id: "link-002".to_string(),
            name: "Internet Circuit".to_string(),
            node_a_id: "router-001".to_string(),
            interface_a: "GigabitEthernet0/0/2".to_string(),
            node_b_id: None,
            interface_b: None,
            capacity: Some(100_000_000),
            utilization: Some(0.25),
            is_internet_circuit: 1,
            circuit_id: Some("CIRC-12345".to_string()),
            provider: Some("ISP Corp".to_string()),
            description: Some("Primary internet connection".to_string()),
            custom_data: Some(r#"{"sla": "99.9%"}"#.to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(link.is_internet_circuit, 1);
        assert_eq!(link.node_b_id, None);
        assert_eq!(link.circuit_id, Some("CIRC-12345".to_string()));
        assert_eq!(link.provider, Some("ISP Corp".to_string()));
    }

    #[test]
    fn test_link_model_serialization() {
        let link = Model {
            id: "link-001".to_string(),
            name: "Router A to Router B".to_string(),
            node_a_id: "node-a".to_string(),
            interface_a: "GigabitEthernet0/0/0".to_string(),
            node_b_id: Some("node-b".to_string()),
            interface_b: Some("GigabitEthernet0/0/1".to_string()),
            capacity: None,
            utilization: None,
            is_internet_circuit: 0,
            circuit_id: None,
            provider: None,
            description: None,
            custom_data: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&link).unwrap();
        let deserialized: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(link, deserialized);
    }

    #[test]
    fn test_link_relation_definitions() {
        let node_a_rel = Relation::NodeA.def();
        assert_eq!(node_a_rel.from_tbl, Entity.table_ref());

        let node_b_relation = Relation::NodeB.def();
        assert_eq!(node_b_relation.from_tbl, Entity.table_ref());
    }

    #[test]
    fn test_link_relation_enum_iter() {
        use sea_orm::Iterable;
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 2);
        // Test that we can iterate over relations
        for relation in relations {
            let _rel_def = relation.def();
        }
    }
}
