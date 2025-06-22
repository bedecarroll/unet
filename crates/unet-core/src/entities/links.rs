//! SeaORM Entity for Links table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub node_a_id: String,
    pub interface_a: String,
    pub node_b_id: Option<String>,
    pub interface_b: Option<String>,
    pub capacity: Option<i64>,
    pub utilization: Option<f64>,
    pub is_internet_circuit: i32,
    pub circuit_id: Option<String>,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub custom_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeAId",
        to = "super::nodes::Column::Id"
    )]
    NodeA,
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeBId",
        to = "super::nodes::Column::Id"
    )]
    NodeB,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeA.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
