//! SeaORM Entity for Template Versions table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "template_version")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub template_id: String,
    pub version: String,
    pub git_commit: String,
    pub content_hash: String,
    pub content: String,
    pub change_log: Option<String>,
    pub is_stable: bool,
    pub custom_data: Option<String>,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::templates::Entity",
        from = "Column::TemplateId",
        to = "super::templates::Column::Id"
    )]
    Template,
}

impl Related<super::templates::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
