//! SeaORM Entity for Change Audit Log table

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "change_audit_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub change_id: String,
    pub action: String,
    pub actor_id: Option<String>,
    pub actor_type: String,
    pub details: Option<String>,
    pub metadata: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: String,
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
