//! `SeaORM` Entity for Node Status table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Node status information from SNMP polling and monitoring
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "node_status")]
pub struct Model {
    /// Unique identifier for the status record
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Foreign key to node table
    pub node_id: String,
    /// Timestamp of last status update
    pub last_updated: String,
    /// Whether the node is currently reachable via SNMP
    pub reachable: bool,
    /// JSON string containing system information (uptime, name, description, etc.)
    pub system_info: Option<String>,
    /// JSON string containing performance metrics (CPU, memory, etc.)
    pub performance: Option<String>,
    /// JSON string containing environmental data (temperature, fans, etc.)
    pub environmental: Option<String>,
    /// JSON string containing vendor-specific metrics
    pub vendor_metrics: Option<String>,
    /// JSON string containing raw SNMP response data
    pub raw_snmp_data: Option<String>,
    /// Timestamp of last successful SNMP poll
    pub last_snmp_success: Option<String>,
    /// Last error message encountered during polling
    pub last_error: Option<String>,
    /// Number of consecutive failed polling attempts
    pub consecutive_failures: i32,
}

/// Database relations for node status entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relation to the parent node
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id"
    )]
    Node,
    /// Relation to interface status records for this node
    #[sea_orm(has_many = "super::interface_status::Entity")]
    InterfaceStatus,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl Related<super::interface_status::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InterfaceStatus.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Iterable;

    #[test]
    fn test_node_status_model_creation() {
        let status = Model {
            id: "status-123".to_string(),
            node_id: "node-456".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
            reachable: true,
            system_info: Some(r#"{"uptime": 3600}"#.to_string()),
            performance: Some(r#"{"cpu": 45.5, "memory": 80.2}"#.to_string()),
            environmental: Some(r#"{"temperature": 25.3}"#.to_string()),
            vendor_metrics: Some(r#"{"fan_speed": 1500}"#.to_string()),
            raw_snmp_data: Some(r#"{"1.3.6.1.2.1.1.1.0": "Cisco Router"}"#.to_string()),
            last_snmp_success: Some("2023-01-01T00:00:00Z".to_string()),
            last_error: None,
            consecutive_failures: 0,
        };

        assert_eq!(status.id, "status-123");
        assert_eq!(status.node_id, "node-456");
        assert_eq!(status.last_updated, "2023-01-01T00:00:00Z");
        assert!(status.reachable);
        assert_eq!(status.system_info, Some(r#"{"uptime": 3600}"#.to_string()));
        assert_eq!(
            status.performance,
            Some(r#"{"cpu": 45.5, "memory": 80.2}"#.to_string())
        );
        assert_eq!(
            status.environmental,
            Some(r#"{"temperature": 25.3}"#.to_string())
        );
        assert_eq!(
            status.vendor_metrics,
            Some(r#"{"fan_speed": 1500}"#.to_string())
        );
        assert_eq!(
            status.raw_snmp_data,
            Some(r#"{"1.3.6.1.2.1.1.1.0": "Cisco Router"}"#.to_string())
        );
        assert_eq!(
            status.last_snmp_success,
            Some("2023-01-01T00:00:00Z".to_string())
        );
        assert_eq!(status.last_error, None);
        assert_eq!(status.consecutive_failures, 0);
    }

    #[test]
    fn test_node_status_model_minimal() {
        let status = Model {
            id: "status-minimal".to_string(),
            node_id: "node-minimal".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
            reachable: false,
            system_info: None,
            performance: None,
            environmental: None,
            vendor_metrics: None,
            raw_snmp_data: None,
            last_snmp_success: None,
            last_error: Some("Connection timeout".to_string()),
            consecutive_failures: 5,
        };

        assert_eq!(status.id, "status-minimal");
        assert_eq!(status.node_id, "node-minimal");
        assert!(!status.reachable);
        assert_eq!(status.system_info, None);
        assert_eq!(status.performance, None);
        assert_eq!(status.environmental, None);
        assert_eq!(status.vendor_metrics, None);
        assert_eq!(status.raw_snmp_data, None);
        assert_eq!(status.last_snmp_success, None);
        assert_eq!(status.last_error, Some("Connection timeout".to_string()));
        assert_eq!(status.consecutive_failures, 5);
    }

    #[test]
    fn test_node_status_model_debug() {
        let status = Model {
            id: "status-debug".to_string(),
            node_id: "node-debug".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
            reachable: false,
            system_info: None,
            performance: None,
            environmental: None,
            vendor_metrics: None,
            raw_snmp_data: None,
            last_snmp_success: None,
            last_error: Some("Debug error".to_string()),
            consecutive_failures: 3,
        };

        let debug_str = format!("{status:?}");
        assert!(debug_str.contains("status-debug"));
        assert!(debug_str.contains("node-debug"));
        assert!(debug_str.contains("Debug error"));
        assert!(debug_str.contains("consecutive_failures: 3"));
    }

    #[test]
    fn test_node_status_model_equality() {
        let status1 = Model {
            id: "status-eq".to_string(),
            node_id: "node-eq".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
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

        // Test equality with itself
        assert_eq!(status1, status1);

        let status3 = Model {
            id: "status-eq-diff".to_string(),
            node_id: "node-equality".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
            reachable: false,
            system_info: None,
            performance: None,
            environmental: None,
            vendor_metrics: None,
            raw_snmp_data: None,
            last_snmp_success: None,
            last_error: None,
            consecutive_failures: 0,
        };
        assert_ne!(status1, status3);
    }

    #[test]
    fn test_node_status_model_serialization() {
        let status = Model {
            id: "status-ser".to_string(),
            node_id: "node-ser".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
            reachable: true,
            system_info: Some(r#"{"hostname": "test-router"}"#.to_string()),
            performance: Some(r#"{"cpu_usage": 12.5}"#.to_string()),
            environmental: None,
            vendor_metrics: None,
            raw_snmp_data: Some(r#"{"raw": "data"}"#.to_string()),
            last_snmp_success: Some("2023-01-01T00:00:00Z".to_string()),
            last_error: None,
            consecutive_failures: 0,
        };

        let serialized = serde_json::to_string(&status).expect("Failed to serialize");
        let deserialized: Model = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(status.id, deserialized.id);
        assert_eq!(status.node_id, deserialized.node_id);
        assert_eq!(status.reachable, deserialized.reachable);
        assert_eq!(status.system_info, deserialized.system_info);
        assert_eq!(status.performance, deserialized.performance);
        assert_eq!(status.raw_snmp_data, deserialized.raw_snmp_data);
        assert_eq!(
            status.consecutive_failures,
            deserialized.consecutive_failures
        );
    }

    #[test]
    fn test_relation_enum() {
        // Test that Relation enum can be iterated
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 2);
        // Verify we can iterate and convert to strings
        let relation_names: Vec<String> = relations.iter().map(|r| format!("{r:?}")).collect();
        assert!(relation_names.iter().any(|name| name.contains("Node")));
        assert!(
            relation_names
                .iter()
                .any(|name| name.contains("InterfaceStatus"))
        );
    }

    #[test]
    fn test_relation_enum_debug() {
        let node_relation = Relation::Node;
        let interface_relation = Relation::InterfaceStatus;

        let node_debug = format!("{node_relation:?}");
        let interface_debug = format!("{interface_relation:?}");

        assert!(node_debug.contains("Node"));
        assert!(interface_debug.contains("InterfaceStatus"));
    }

    #[test]
    fn test_relation_enum_clone_copy() {
        let relation1 = Relation::Node;
        let relation2 = relation1;
        assert!(matches!(relation1, Relation::Node));
        assert!(matches!(relation2, Relation::Node));

        let relation3 = Relation::InterfaceStatus;
        let relation4 = relation3;
        assert!(matches!(relation3, Relation::InterfaceStatus));
        assert!(matches!(relation4, Relation::InterfaceStatus));
    }

    #[test]
    fn test_node_status_boolean_fields() {
        let mut status = Model {
            id: "bool-test".to_string(),
            node_id: "node-bool".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
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

        assert!(status.reachable);

        status.reachable = false;
        assert!(!status.reachable);
    }

    #[test]
    fn test_node_status_counter_fields() {
        let mut status = Model {
            id: "counter-test".to_string(),
            node_id: "node-counter".to_string(),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
            reachable: false,
            system_info: None,
            performance: None,
            environmental: None,
            vendor_metrics: None,
            raw_snmp_data: None,
            last_snmp_success: None,
            last_error: Some("Initial error".to_string()),
            consecutive_failures: 1,
        };

        assert_eq!(status.consecutive_failures, 1);

        status.consecutive_failures = 10;
        assert_eq!(status.consecutive_failures, 10);

        // Test negative values (might happen in some error scenarios)
        status.consecutive_failures = -1;
        assert_eq!(status.consecutive_failures, -1);
    }

    #[test]
    fn test_interface_status_relation() {
        use crate::entities::interface_status;
        use sea_orm::Related;

        // Test that the relation to interface_status entity is properly defined
        let _relation_def = <Entity as Related<interface_status::Entity>>::to();

        // The fact that this compiles and runs means the relation is properly implemented
    }
}
