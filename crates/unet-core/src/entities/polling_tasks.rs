//! `SeaORM` Entity for Polling Tasks table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SNMP polling task configuration for scheduled data collection
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "polling_tasks")]
pub struct Model {
    /// Unique identifier for the polling task
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Foreign key to node being polled
    pub node_id: String,
    /// Target IP address or hostname for SNMP polling
    pub target: String,
    /// JSON array of SNMP OIDs to poll
    pub oids: String,
    /// Polling interval in seconds
    pub interval_seconds: i64,
    /// JSON configuration for SNMP session (community, version, etc.)
    pub session_config: String,
    /// Task priority for scheduling (higher numbers = higher priority)
    pub priority: i16,
    /// Whether this polling task is active
    pub enabled: bool,
    /// Timestamp when task was created
    pub created_at: String,
    /// Timestamp of last successful poll
    pub last_success: Option<String>,
    /// Last error message encountered
    pub last_error: Option<String>,
    /// Number of consecutive failed polling attempts
    pub consecutive_failures: i32,
}

/// Database relations for polling task entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relation to the node being polled
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id"
    )]
    Node,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
