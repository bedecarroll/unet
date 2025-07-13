//! `SeaORM` Entity for Links table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Network link entity representing connections between devices
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "link")]
pub struct Model {
    /// Unique identifier for the link
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Human-readable link name
    pub name: String,
    /// Foreign key to first node (required)
    pub node_a_id: String,
    /// Interface name on first node
    pub interface_a: String,
    /// Foreign key to second node (optional for internet circuits)
    pub node_b_id: Option<String>,
    /// Interface name on second node
    pub interface_b: Option<String>,
    /// Link capacity in bits per second
    pub capacity: Option<i64>,
    /// Current utilization as percentage (0.0-1.0)
    pub utilization: Option<f64>,
    /// Whether this is an internet circuit (1) or internal link (0)
    pub is_internet_circuit: i32,
    /// Provider circuit identifier
    pub circuit_id: Option<String>,
    /// Service provider name
    pub provider: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// JSON string for custom attributes
    pub custom_data: Option<String>,
    /// Timestamp when record was created
    pub created_at: String,
    /// Timestamp when record was last updated
    pub updated_at: String,
}

/// Database relations for link entity
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relation to first node in the link
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeAId",
        to = "super::nodes::Column::Id"
    )]
    NodeA,
    /// Relation to second node in the link
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
