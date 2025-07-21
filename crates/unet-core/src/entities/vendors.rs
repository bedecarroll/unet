//! `SeaORM` Entity for vendors table

use sea_orm::entity::prelude::*;

/// `SeaORM` entity model for the vendor table
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "vendor")]
pub struct Model {
    /// Vendor name (primary key)
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
}

/// Database relations for the vendor entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
