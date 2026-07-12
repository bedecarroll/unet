//! `SeaORM` Entity for Interface Status table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Interface status information from SNMP polling
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "interface_status")]
pub struct Model {
    /// Unique identifier for the interface status record
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Foreign key to `node_status` table
    pub node_status_id: String,
    /// Interface index from SNMP ifTable
    pub index: i32,
    /// Interface name or description
    pub name: String,
    /// Interface type code from SNMP ifType
    pub interface_type: i32,
    /// Maximum transmission unit in bytes
    pub mtu: Option<i32>,
    /// Interface speed in bits per second
    pub speed: Option<i64>,
    /// Physical MAC address of the interface
    pub physical_address: Option<String>,
    /// Administrative status (up/down/testing)
    pub admin_status: String,
    /// Operational status (up/down/testing/unknown/dormant/notPresent/lowerLayerDown)
    pub oper_status: String,
    /// Time when the interface last changed state
    pub last_change: Option<i32>,
    /// JSON string containing input statistics (packets, bytes, errors, etc.)
    pub input_stats: String,
    /// JSON string containing output statistics (packets, bytes, errors, etc.)
    pub output_stats: String,
}

/// Database relations for interface status entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relation to parent node status
    #[sea_orm(
        belongs_to = "super::node_status::Entity",
        from = "Column::NodeStatusId",
        to = "super::node_status::Column::Id"
    )]
    NodeStatus,
}

impl Related<super::node_status::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeStatus.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
