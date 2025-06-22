//! SeaORM Entity for Node Status table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "node_status")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub node_id: String,
    pub last_updated: String,
    pub reachable: bool,
    pub system_info: Option<String>,
    pub performance: Option<String>,
    pub environmental: Option<String>,
    pub vendor_metrics: Option<String>,
    pub raw_snmp_data: Option<String>,
    pub last_snmp_success: Option<String>,
    pub last_error: Option<String>,
    pub consecutive_failures: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id"
    )]
    Node,
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
