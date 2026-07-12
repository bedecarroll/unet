//! Tests for `interface_status` entity

#[cfg(test)]
mod tests {
    use super::super::super::interface_status::*;
    use sea_orm::entity::prelude::*;
    use serde_json;

    #[test]
    fn test_interface_status_model_creation() {
        let status = Model {
            id: "int-status-001".to_string(),
            node_status_id: "node-status-001".to_string(),
            index: 1,
            name: "GigabitEthernet0/0/0".to_string(),
            interface_type: 6,
            mtu: Some(1500),
            speed: Some(1_000_000_000),
            physical_address: Some("00:11:22:33:44:55".to_string()),
            admin_status: "up".to_string(),
            oper_status: "up".to_string(),
            last_change: Some(12345),
            input_stats: r#"{"packets": 1000, "bytes": 64000, "errors": 0}"#.to_string(),
            output_stats: r#"{"packets": 800, "bytes": 51200, "errors": 0}"#.to_string(),
        };

        assert_eq!(status.id, "int-status-001");
        assert_eq!(status.node_status_id, "node-status-001");
        assert_eq!(status.index, 1);
        assert_eq!(status.name, "GigabitEthernet0/0/0");
        assert_eq!(status.interface_type, 6);
        assert_eq!(status.mtu, Some(1500));
        assert_eq!(status.speed, Some(1_000_000_000));
        assert_eq!(status.admin_status, "up");
        assert_eq!(status.oper_status, "up");
    }

    #[test]
    fn test_interface_status_model_down() {
        let status = Model {
            id: "int-status-002".to_string(),
            node_status_id: "node-status-001".to_string(),
            index: 2,
            name: "GigabitEthernet0/0/1".to_string(),
            interface_type: 6,
            mtu: Some(1500),
            speed: Some(1_000_000_000),
            physical_address: Some("00:11:22:33:44:56".to_string()),
            admin_status: "up".to_string(),
            oper_status: "down".to_string(),
            last_change: Some(54321),
            input_stats: r#"{"packets": 0, "bytes": 0, "errors": 1}"#.to_string(),
            output_stats: r#"{"packets": 0, "bytes": 0, "errors": 0}"#.to_string(),
        };

        assert_eq!(status.oper_status, "down");
        assert_eq!(status.admin_status, "up");
    }

    #[test]
    fn test_interface_status_model_serialization() {
        let status = Model {
            id: "int-status-001".to_string(),
            node_status_id: "node-status-001".to_string(),
            index: 1,
            name: "GigabitEthernet0/0/0".to_string(),
            interface_type: 6,
            mtu: None,
            speed: None,
            physical_address: None,
            admin_status: "up".to_string(),
            oper_status: "up".to_string(),
            last_change: None,
            input_stats: r#"{"packets": 0, "bytes": 0, "errors": 0}"#.to_string(),
            output_stats: r#"{"packets": 0, "bytes": 0, "errors": 0}"#.to_string(),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_interface_status_relation_definitions() {
        let node_status_rel = Relation::NodeStatus.def();
        assert_eq!(node_status_rel.from_tbl, Entity.table_ref());
    }

    #[test]
    fn test_interface_status_relation_enum_iter() {
        use sea_orm::Iterable;
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 1);
        // Test that we can iterate over relations
        for relation in relations {
            let _rel_def = relation.def();
        }
    }
}
