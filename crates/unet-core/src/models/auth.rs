//! Authentication and Authorization Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub is_admin: bool,
    pub exp: i64,    // Expiration time (Unix timestamp)
    pub iat: i64,    // Issued at (Unix timestamp)
    pub jti: String, // JWT ID (for revocation)
}

/// User creation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    #[validate(regex(
        path = "crate::validation::USERNAME_REGEX",
        message = "Username can only contain alphanumeric characters, hyphens, and underscores"
    ))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 254, message = "Email too long"))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    pub password: String,

    #[validate(length(max = 100, message = "Full name too long"))]
    pub full_name: Option<String>,

    #[validate(length(max = 10, message = "Too many roles specified"))]
    pub roles: Vec<String>,
}

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub is_active: Option<bool>,
    pub roles: Option<Vec<String>>,
}

/// User response (password hash excluded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub roles: Vec<RoleResponse>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Login request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, max = 50, message = "Username required"))]
    pub username: String,

    #[validate(length(min = 1, max = 128, message = "Password required"))]
    pub password: String,
}

/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64, // Seconds until expiration
    pub user: UserResponse,
}

/// Role creation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Role name must be between 1 and 50 characters"
    ))]
    #[validate(regex(
        path = "crate::validation::POLICY_NAME_REGEX",
        message = "Role name can only contain alphanumeric characters, hyphens, and underscores"
    ))]
    pub name: String,

    #[validate(length(max = 255, message = "Description too long"))]
    pub description: Option<String>,

    #[validate(length(max = 50, message = "Too many permissions specified"))]
    pub permissions: Vec<String>,
}

/// Role update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
}

/// Role response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API key creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit: Option<i32>,
}

/// API key response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API key creation response (includes the actual key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub api_key: String, // The actual key - only returned once
    pub details: ApiKeyResponse,
}

/// Password change request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Permission enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    // Node management
    NodesRead,
    NodesWrite,
    NodesDelete,

    // Link management
    LinksRead,
    LinksWrite,
    LinksDelete,

    // Location management
    LocationsRead,
    LocationsWrite,
    LocationsDelete,

    // Template management
    TemplatesRead,
    TemplatesWrite,
    TemplatesDelete,

    // Policy management
    PoliciesRead,
    PoliciesWrite,
    PoliciesDelete,

    // Configuration management
    ConfigRead,
    ConfigWrite,
    ConfigDeploy,

    // Git operations
    GitRead,
    GitWrite,
    GitSync,

    // Change management
    ChangesRead,
    ChangesCreate,
    ChangesApprove,
    ChangesReject,
    ChangesRollback,

    // User management
    UsersRead,
    UsersWrite,
    UsersDelete,

    // Role management
    RolesRead,
    RolesWrite,
    RolesDelete,

    // API key management
    ApiKeysRead,
    ApiKeysWrite,
    ApiKeysDelete,

    // System administration
    SystemRead,
    SystemWrite,
    SystemAdmin,
}

impl Permission {
    /// Get all available permissions
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::NodesRead,
            Permission::NodesWrite,
            Permission::NodesDelete,
            Permission::LinksRead,
            Permission::LinksWrite,
            Permission::LinksDelete,
            Permission::LocationsRead,
            Permission::LocationsWrite,
            Permission::LocationsDelete,
            Permission::TemplatesRead,
            Permission::TemplatesWrite,
            Permission::TemplatesDelete,
            Permission::PoliciesRead,
            Permission::PoliciesWrite,
            Permission::PoliciesDelete,
            Permission::ConfigRead,
            Permission::ConfigWrite,
            Permission::ConfigDeploy,
            Permission::GitRead,
            Permission::GitWrite,
            Permission::GitSync,
            Permission::ChangesRead,
            Permission::ChangesCreate,
            Permission::ChangesApprove,
            Permission::ChangesReject,
            Permission::ChangesRollback,
            Permission::UsersRead,
            Permission::UsersWrite,
            Permission::UsersDelete,
            Permission::RolesRead,
            Permission::RolesWrite,
            Permission::RolesDelete,
            Permission::ApiKeysRead,
            Permission::ApiKeysWrite,
            Permission::ApiKeysDelete,
            Permission::SystemRead,
            Permission::SystemWrite,
            Permission::SystemAdmin,
        ]
    }

    /// Convert permission to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::NodesRead => "nodes:read",
            Permission::NodesWrite => "nodes:write",
            Permission::NodesDelete => "nodes:delete",
            Permission::LinksRead => "links:read",
            Permission::LinksWrite => "links:write",
            Permission::LinksDelete => "links:delete",
            Permission::LocationsRead => "locations:read",
            Permission::LocationsWrite => "locations:write",
            Permission::LocationsDelete => "locations:delete",
            Permission::TemplatesRead => "templates:read",
            Permission::TemplatesWrite => "templates:write",
            Permission::TemplatesDelete => "templates:delete",
            Permission::PoliciesRead => "policies:read",
            Permission::PoliciesWrite => "policies:write",
            Permission::PoliciesDelete => "policies:delete",
            Permission::ConfigRead => "config:read",
            Permission::ConfigWrite => "config:write",
            Permission::ConfigDeploy => "config:deploy",
            Permission::GitRead => "git:read",
            Permission::GitWrite => "git:write",
            Permission::GitSync => "git:sync",
            Permission::ChangesRead => "changes:read",
            Permission::ChangesCreate => "changes:create",
            Permission::ChangesApprove => "changes:approve",
            Permission::ChangesReject => "changes:reject",
            Permission::ChangesRollback => "changes:rollback",
            Permission::UsersRead => "users:read",
            Permission::UsersWrite => "users:write",
            Permission::UsersDelete => "users:delete",
            Permission::RolesRead => "roles:read",
            Permission::RolesWrite => "roles:write",
            Permission::RolesDelete => "roles:delete",
            Permission::ApiKeysRead => "api_keys:read",
            Permission::ApiKeysWrite => "api_keys:write",
            Permission::ApiKeysDelete => "api_keys:delete",
            Permission::SystemRead => "system:read",
            Permission::SystemWrite => "system:write",
            Permission::SystemAdmin => "system:admin",
        }
    }

    /// Parse permission from string
    pub fn from_str(s: &str) -> Option<Permission> {
        match s {
            "nodes:read" => Some(Permission::NodesRead),
            "nodes:write" => Some(Permission::NodesWrite),
            "nodes:delete" => Some(Permission::NodesDelete),
            "links:read" => Some(Permission::LinksRead),
            "links:write" => Some(Permission::LinksWrite),
            "links:delete" => Some(Permission::LinksDelete),
            "locations:read" => Some(Permission::LocationsRead),
            "locations:write" => Some(Permission::LocationsWrite),
            "locations:delete" => Some(Permission::LocationsDelete),
            "templates:read" => Some(Permission::TemplatesRead),
            "templates:write" => Some(Permission::TemplatesWrite),
            "templates:delete" => Some(Permission::TemplatesDelete),
            "policies:read" => Some(Permission::PoliciesRead),
            "policies:write" => Some(Permission::PoliciesWrite),
            "policies:delete" => Some(Permission::PoliciesDelete),
            "config:read" => Some(Permission::ConfigRead),
            "config:write" => Some(Permission::ConfigWrite),
            "config:deploy" => Some(Permission::ConfigDeploy),
            "git:read" => Some(Permission::GitRead),
            "git:write" => Some(Permission::GitWrite),
            "git:sync" => Some(Permission::GitSync),
            "changes:read" => Some(Permission::ChangesRead),
            "changes:create" => Some(Permission::ChangesCreate),
            "changes:approve" => Some(Permission::ChangesApprove),
            "changes:reject" => Some(Permission::ChangesReject),
            "changes:rollback" => Some(Permission::ChangesRollback),
            "users:read" => Some(Permission::UsersRead),
            "users:write" => Some(Permission::UsersWrite),
            "users:delete" => Some(Permission::UsersDelete),
            "roles:read" => Some(Permission::RolesRead),
            "roles:write" => Some(Permission::RolesWrite),
            "roles:delete" => Some(Permission::RolesDelete),
            "api_keys:read" => Some(Permission::ApiKeysRead),
            "api_keys:write" => Some(Permission::ApiKeysWrite),
            "api_keys:delete" => Some(Permission::ApiKeysDelete),
            "system:read" => Some(Permission::SystemRead),
            "system:write" => Some(Permission::SystemWrite),
            "system:admin" => Some(Permission::SystemAdmin),
            _ => None,
        }
    }
}

/// Predefined system roles
pub mod system_roles {
    use super::Permission;

    /// Administrator role with all permissions
    pub fn admin_permissions() -> Vec<String> {
        Permission::all()
            .into_iter()
            .map(|p| p.as_str().to_string())
            .collect()
    }

    /// Read-only user role
    pub fn readonly_permissions() -> Vec<String> {
        vec![
            Permission::NodesRead,
            Permission::LinksRead,
            Permission::LocationsRead,
            Permission::TemplatesRead,
            Permission::PoliciesRead,
            Permission::ConfigRead,
            Permission::GitRead,
            Permission::ChangesRead,
            Permission::SystemRead,
        ]
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect()
    }

    /// Network operator role
    pub fn operator_permissions() -> Vec<String> {
        vec![
            Permission::NodesRead,
            Permission::NodesWrite,
            Permission::LinksRead,
            Permission::LinksWrite,
            Permission::LocationsRead,
            Permission::LocationsWrite,
            Permission::TemplatesRead,
            Permission::PoliciesRead,
            Permission::ConfigRead,
            Permission::ConfigWrite,
            Permission::GitRead,
            Permission::GitWrite,
            Permission::GitSync,
            Permission::ChangesRead,
            Permission::ChangesCreate,
            Permission::SystemRead,
        ]
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect()
    }

    /// Configuration manager role
    pub fn config_manager_permissions() -> Vec<String> {
        vec![
            Permission::NodesRead,
            Permission::NodesWrite,
            Permission::LinksRead,
            Permission::LinksWrite,
            Permission::LocationsRead,
            Permission::LocationsWrite,
            Permission::TemplatesRead,
            Permission::TemplatesWrite,
            Permission::PoliciesRead,
            Permission::PoliciesWrite,
            Permission::ConfigRead,
            Permission::ConfigWrite,
            Permission::ConfigDeploy,
            Permission::GitRead,
            Permission::GitWrite,
            Permission::GitSync,
            Permission::ChangesRead,
            Permission::ChangesCreate,
            Permission::ChangesApprove,
            Permission::ChangesReject,
            Permission::ChangesRollback,
            Permission::SystemRead,
        ]
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect()
    }
}
