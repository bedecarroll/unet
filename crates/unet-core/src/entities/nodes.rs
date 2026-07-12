//! `SeaORM` Entity for Nodes table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Network node entity representing managed network devices
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    /// Unique identifier for the node
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Node name (typically hostname)
    pub name: String,
    /// Fully qualified domain name
    pub fqdn: Option<String>,
    /// DNS domain
    pub domain: Option<String>,
    /// Device vendor (e.g., Cisco, Juniper, Arista)
    pub vendor: String,
    /// Device model number
    pub model: String,
    /// Device role (router, switch, firewall, etc.)
    pub role: String,
    /// Device lifecycle stage (live, staging, retired, etc.)
    pub lifecycle: String,
    /// Device serial number
    pub serial_number: Option<String>,
    /// Asset tag for inventory tracking
    pub asset_tag: Option<String>,
    /// Foreign key to location
    pub location_id: Option<String>,
    /// Primary management IP address
    pub management_ip: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// JSON string for custom attributes
    pub custom_data: Option<String>,
    /// Timestamp when record was created
    pub created_at: String,
    /// Timestamp when record was last updated
    pub updated_at: String,
}

/// Database relations for node entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relation to location where this node is placed
    #[sea_orm(
        belongs_to = "super::locations::Entity",
        from = "Column::LocationId",
        to = "super::locations::Column::Id"
    )]
    Location,
    /// Relation to links where this node is the A side
    #[sea_orm(
        has_many = "super::links::Entity",
        from = "Column::Id",
        to = "super::links::Column::NodeAId"
    )]
    LinksAsNodeA,
    /// Relation to links where this node is the B side
    #[sea_orm(
        has_many = "super::links::Entity",
        from = "Column::Id",
        to = "super::links::Column::NodeBId"
    )]
    LinksAsNodeB,
    /// Relation to status records for this node
    #[sea_orm(has_many = "super::node_status::Entity")]
    NodeStatus,
    /// Relation to polling tasks for this node
    #[sea_orm(has_many = "super::polling_tasks::Entity")]
    PollingTasks,
}

impl Related<super::locations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Location.def()
    }
}

impl Related<super::node_status::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeStatus.def()
    }
}

impl Related<super::polling_tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PollingTasks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
