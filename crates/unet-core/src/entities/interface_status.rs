//! SeaORM Entity for Interface Status table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "interface_status")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub node_status_id: String,
    pub index: i32,
    pub name: String,
    pub interface_type: i32,
    pub mtu: Option<i32>,
    pub speed: Option<i64>,
    pub physical_address: Option<String>,
    pub admin_status: String,
    pub oper_status: String,
    pub last_change: Option<i32>,
    pub input_stats: String,
    pub output_stats: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
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
