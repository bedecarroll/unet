//! SeaORM Entity for Templates table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "template")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub vendor: Option<String>,
    pub template_type: String,
    pub version: String,
    pub git_repository: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub content_hash: Option<String>,
    pub match_headers: Option<String>,
    pub is_active: bool,
    pub custom_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::template_assignments::Entity")]
    TemplateAssignments,
    #[sea_orm(has_many = "super::template_versions::Entity")]
    TemplateVersions,
    #[sea_orm(has_many = "super::template_usage::Entity")]
    TemplateUsage,
}

impl Related<super::template_assignments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TemplateAssignments.def()
    }
}

impl Related<super::template_versions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TemplateVersions.def()
    }
}

impl Related<super::template_usage::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TemplateUsage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
