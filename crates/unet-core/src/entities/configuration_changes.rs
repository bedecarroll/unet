//! SeaORM Entity for Configuration Changes table

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "configuration_changes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub change_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub user_id: Option<String>,
    pub source: String,
    pub description: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub diff_content: Option<String>,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub status: String,
    pub approval_required: bool,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub applied_at: Option<String>,
    pub rolled_back_at: Option<String>,
    pub custom_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::change_audit_log::Entity")]
    ChangeAuditLogs,
    #[sea_orm(has_many = "super::change_approval_workflow::Entity")]
    ChangeApprovalWorkflows,
    #[sea_orm(has_many = "super::change_rollback_snapshot::Entity")]
    ChangeRollbackSnapshots,
}

impl Related<super::change_audit_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChangeAuditLogs.def()
    }
}

impl Related<super::change_approval_workflow::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChangeApprovalWorkflows.def()
    }
}

impl Related<super::change_rollback_snapshot::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChangeRollbackSnapshots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
