//! SeaORM Entity for Change Approval Workflow table

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "change_approval_workflow")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub change_id: String,
    pub workflow_type: String,
    pub status: String,
    pub required_approvers: Option<String>,
    pub current_approvers: Option<String>,
    pub rules: Option<String>,
    pub comments: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::configuration_changes::Entity",
        from = "Column::ChangeId",
        to = "super::configuration_changes::Column::Id",
        on_delete = "Cascade"
    )]
    ConfigurationChange,
}

impl Related<super::configuration_changes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ConfigurationChange.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
