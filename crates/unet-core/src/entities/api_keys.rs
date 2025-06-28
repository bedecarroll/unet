//! API Keys Entity for API Authentication

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub user_id: Uuid,

    pub name: String, // Human-readable name for the API key

    #[sea_orm(unique)]
    pub key_hash: String, // Hashed version of the API key

    pub key_prefix: String, // First few characters of the key for identification

    pub scopes: serde_json::Value, // JSON array of allowed scopes/permissions

    pub is_active: bool,

    pub last_used: Option<DateTime<Utc>>,

    pub usage_count: i64,

    pub rate_limit: Option<i32>, // Requests per hour, None for unlimited

    pub expires_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
