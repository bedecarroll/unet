//! `SeaORM` Entity for Locations table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Physical or logical location entity for organizing network devices
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "location")]
pub struct Model {
    /// Unique identifier for the location
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Human-readable location name
    pub name: String,
    /// Type of location (datacenter, building, floor, rack, etc.)
    pub location_type: String,
    /// Hierarchical path for location tree navigation
    pub path: String,
    /// Foreign key to parent location for hierarchy
    pub parent_id: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Physical address or location details
    pub address: Option<String>,
    /// GPS coordinates or position data
    pub coordinates: Option<String>,
    /// JSON string for custom attributes
    pub custom_data: Option<String>,
    /// Timestamp when record was created
    pub created_at: String,
    /// Timestamp when record was last updated
    pub updated_at: String,
}

/// Database relations for location entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relation to nodes located at this location
    #[sea_orm(has_many = "super::nodes::Entity")]
    Nodes,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
