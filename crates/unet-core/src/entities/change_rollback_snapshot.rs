//! SeaORM Entity for Change Rollback Snapshot table

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "change_rollback_snapshot")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub change_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub snapshot_type: String,
    pub state_snapshot: String,
    pub checksum: String,
    pub dependencies: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
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
