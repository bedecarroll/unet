//! SeaORM Entity for Locations table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "location")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub location_type: String,
    pub path: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub coordinates: Option<String>,
    pub custom_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::nodes::Entity")]
    Nodes,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
