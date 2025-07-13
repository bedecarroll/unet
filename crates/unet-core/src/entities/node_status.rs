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
