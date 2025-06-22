//! SeaORM Entity for Nodes table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub fqdn: Option<String>,
    pub domain: Option<String>,
    pub vendor: String,
    pub model: String,
    pub role: String,
    pub lifecycle: String,
    pub serial_number: Option<String>,
    pub asset_tag: Option<String>,
    pub location_id: Option<String>,
    pub management_ip: Option<String>,
    pub description: Option<String>,
    pub custom_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::locations::Entity",
        from = "Column::LocationId",
        to = "super::locations::Column::Id"
    )]
    Location,
    #[sea_orm(
        has_many = "super::links::Entity",
        from = "Column::Id",
        to = "super::links::Column::NodeAId"
    )]
    LinksAsNodeA,
    #[sea_orm(
        has_many = "super::links::Entity",
        from = "Column::Id",
        to = "super::links::Column::NodeBId"
    )]
    LinksAsNodeB,
    #[sea_orm(has_many = "super::node_status::Entity")]
    NodeStatus,
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
