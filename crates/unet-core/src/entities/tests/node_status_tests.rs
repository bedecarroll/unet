//! Tests for `node_status` entity

#[cfg(test)]
mod tests {
    use super::super::super::node_status::*;
    use sea_orm::entity::prelude::*;
    use serde_json;

    #[test]
    fn test_node_status_model_creation() {
        let status = Model {
            id: "status-001".to_string(),
            node_id: "node-001".to_string(),
            last_updated: "2023-01-01T12:00:00Z".to_string(),
            reachable: true,
            system_info: Some(r#"{"uptime": 86400, "name": "router1"}"#.to_string()),
            performance: Some(r#"{"cpu_usage": 45.2, "memory_usage": 78.5}"#.to_string()),
            environmental: Some(r#"{"temperature": 42.0, "fans": ["ok", "ok"]}"#.to_string()),
            vendor_metrics: Some(r#"{"bgp_peers": 8}"#.to_string()),
            raw_snmp_data: Some(r#"{"1.3.6.1.2.1.1.3.0": "86400"}"#.to_string()),
            last_snmp_success: Some("2023-01-01T12:00:00Z".to_string()),
            last_error: None,
            consecutive_failures: 0,
        };

        assert_eq!(status.id, "status-001");
        assert_eq!(status.node_id, "node-001");
        assert!(status.reachable);
        assert_eq!(status.consecutive_failures, 0);
    }

    #[test]
    fn test_node_status_model_with_error() {
        let status = Model {
            id: "status-002".to_string(),
            node_id: "node-002".to_string(),
            last_updated: "2023-01-01T12:00:00Z".to_string(),
            reachable: false,
            system_info: None,
            performance: None,
            environmental: None,
            vendor_metrics: None,
            raw_snmp_data: None,
            last_snmp_success: None,
            last_error: Some("Timeout".to_string()),
            consecutive_failures: 3,
        };

        assert!(!status.reachable);
        assert_eq!(status.consecutive_failures, 3);
        assert_eq!(status.last_error, Some("Timeout".to_string()));
    }

    #[test]
    fn test_node_status_model_serialization() {
        let status = Model {
            id: "status-001".to_string(),
            node_id: "node-001".to_string(),
            last_updated: "2023-01-01T12:00:00Z".to_string(),
            reachable: true,
            system_info: None,
            performance: None,
            environmental: None,
            vendor_metrics: None,
            raw_snmp_data: None,
            last_snmp_success: None,
            last_error: None,
            consecutive_failures: 0,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_node_status_relation_definitions() {
        let node_rel = Relation::Node.def();
        assert_eq!(node_rel.from_tbl, Entity.table_ref());

        let interface_status_rel = Relation::InterfaceStatus.def();
        assert_eq!(interface_status_rel.from_tbl, Entity.table_ref());
    }

    #[test]
    fn test_node_status_relation_enum_iter() {
        use sea_orm::Iterable;
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 2);
        // Test that we can iterate over relations
        for relation in relations {
            let _rel_def = relation.def();
        }
    }
}
