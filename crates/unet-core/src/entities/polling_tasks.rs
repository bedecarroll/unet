//! SeaORM Entity for Polling Tasks table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "polling_tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub node_id: String,
    pub target: String,
    pub oids: String,
    pub interval_seconds: i64,
    pub session_config: String,
    pub priority: i16,
    pub enabled: bool,
    pub created_at: String,
    pub last_success: Option<String>,
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
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
