//! Authentication API handlers

use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::middleware::auth::AuthContext;
use crate::server::AppState;
use crate::validation::ValidationHelper;
use unet_core::auth::AuthError;
use unet_core::models::auth::*;

/// Login endpoint
pub async fn login(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Validate input
    if let Err(_validation_error) = ValidationHelper::validate(&request) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let auth_service = &app_state.auth_service;

    let user = auth_service
        .authenticate_user(&request.username, &request.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::UserNotActive => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    let access_token = auth_service
        .generate_token(&user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = LoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 86400, // 24 hours
        user,
    };

    Ok(Json(response))
}

/// Get current user endpoint
pub async fn get_current_user(
    Extension(auth_context): Extension<AuthContext>,
) -> Json<UserResponse> {
    Json(auth_context.user)
}

/// Change password endpoint
pub async fn change_password(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, StatusCode> {
    let auth_service = &app_state.auth_service;

    // For demo purposes, just verify that the user provided "admin" as current password
    // and allow any new password
    if request.current_password != "admin" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Hash new password (for demonstration)
    let _new_password_hash = auth_service
        .hash_password(&request.new_password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // In a real implementation, this would update the password in the database
    // For now, we just return success

    Ok(StatusCode::NO_CONTENT)
}

/// Create user endpoint (admin only)
pub async fn create_user(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // Validate input
    if let Err(_validation_error) = ValidationHelper::validate(&request) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check admin permission
    if !auth_context.has_permission("users:write") {
        return Err(StatusCode::FORBIDDEN);
    }

    let auth_service = &app_state.auth_service;

    let user = auth_service
        .create_user(request)
        .await
        .map_err(|e| match e {
            AuthError::UsernameExists => StatusCode::CONFLICT,
            AuthError::EmailExists => StatusCode::CONFLICT,
            AuthError::RoleNotFound => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(user))
}

/// Get user by ID endpoint
pub async fn get_user(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, StatusCode> {
    // Check if user is requesting their own info or has read permission
    if auth_context.user.id != user_id && !auth_context.has_permission("users:read") {
        return Err(StatusCode::FORBIDDEN);
    }

    let auth_service = &app_state.auth_service;

    let user = auth_service.get_user(user_id).await.map_err(|e| match e {
        AuthError::UserNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(user))
}

/// Update user endpoint
pub async fn update_user(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // Check if user is updating their own info or has write permission
    if auth_context.user.id != user_id && !auth_context.has_permission("users:write") {
        return Err(StatusCode::FORBIDDEN);
    }

    let auth_service = &app_state.auth_service;

    // For demo purposes, just return the existing user with some fields potentially updated
    // In a real implementation, this would update the user in the database
    let mut updated_user = auth_service.get_user(user_id).await.map_err(|e| match e {
        AuthError::UserNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    // Simulate updating fields
    if let Some(email) = request.email {
        updated_user.email = email;
    }
    if let Some(full_name) = request.full_name {
        updated_user.full_name = Some(full_name);
    }
    if let Some(is_active) = request.is_active {
        // Only admins can change active status
        if auth_context.has_permission("users:write") {
            updated_user.is_active = is_active;
        }
    }

    // Handle role updates (admin only) - for demo purposes, just log the request
    if let Some(_role_names) = request.roles {
        if !auth_context.has_permission("users:write") {
            return Err(StatusCode::FORBIDDEN);
        }
        // In a real implementation, this would update roles in the database
    }

    let updated_user_response = updated_user;

    Ok(Json(updated_user_response))
}

/// List users endpoint (admin only)
#[derive(Deserialize)]
pub struct ListUsersQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    active_only: Option<bool>,
}

#[derive(Serialize)]
pub struct ListUsersResponse {
    users: Vec<UserResponse>,
    total: u64,
    page: u32,
    per_page: u32,
}

pub async fn list_users(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ListUsersResponse>, StatusCode> {
    if !auth_context.has_permission("users:read") {
        return Err(StatusCode::FORBIDDEN);
    }

    let auth_service = &app_state.auth_service;
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50).min(100);

    // For demo purposes, return a single admin user
    // In a real implementation, this would query the database
    let admin_user = auth_service
        .get_user(Uuid::new_v4())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_responses = if query.active_only.unwrap_or(false) && !admin_user.is_active {
        Vec::new()
    } else {
        vec![admin_user]
    };

    let total = user_responses.len() as u64;

    let response = ListUsersResponse {
        users: user_responses,
        total,
        page,
        per_page,
    };

    Ok(Json(response))
}

/// Create role endpoint (admin only)
pub async fn create_role(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<Json<RoleResponse>, StatusCode> {
    if !auth_context.has_permission("roles:write") {
        return Err(StatusCode::FORBIDDEN);
    }

    let auth_service = &app_state.auth_service;

    let role = auth_service
        .create_role(request)
        .await
        .map_err(|e| match e {
            AuthError::RoleNameExists => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(role))
}

/// List roles endpoint
pub async fn list_roles(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<Vec<RoleResponse>>, StatusCode> {
    if !auth_context.has_permission("roles:read") {
        return Err(StatusCode::FORBIDDEN);
    }

    // For demo purposes, return the predefined system roles
    // In a real implementation, this would query the database
    use chrono::Utc;
    use unet_core::models::auth::{RoleResponse, system_roles};

    let role_responses = vec![
        RoleResponse {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: Some("Administrator".to_string()),
            permissions: system_roles::admin_permissions(),
            is_system: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        RoleResponse {
            id: Uuid::new_v4(),
            name: "readonly".to_string(),
            description: Some("Read-only user".to_string()),
            permissions: system_roles::readonly_permissions(),
            is_system: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        RoleResponse {
            id: Uuid::new_v4(),
            name: "operator".to_string(),
            description: Some("Network operator".to_string()),
            permissions: system_roles::operator_permissions(),
            is_system: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        RoleResponse {
            id: Uuid::new_v4(),
            name: "config_manager".to_string(),
            description: Some("Configuration manager".to_string()),
            permissions: system_roles::config_manager_permissions(),
            is_system: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    Ok(Json(role_responses))
}

/// Create API key endpoint
pub async fn create_api_key(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, StatusCode> {
    let auth_service = &app_state.auth_service;

    let api_key_response = auth_service
        .create_api_key(auth_context.user.id, request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(api_key_response))
}

/// List user's API keys endpoint
pub async fn list_api_keys(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<Vec<ApiKeyResponse>>, StatusCode> {
    // For demo purposes, return an empty list
    // In a real implementation, this would query the database for the user's API keys
    let api_key_responses = Vec::new();

    Ok(Json(api_key_responses))
}

/// Delete API key endpoint
pub async fn delete_api_key(
    State(app_state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Path(api_key_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // For demo purposes, just return success
    // In a real implementation, this would delete the API key from the database
    // after checking permissions
    if !auth_context.has_permission("api_keys:delete") {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Get available permissions endpoint
pub async fn get_permissions() -> Json<Vec<String>> {
    let permissions = Permission::all()
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect();

    Json(permissions)
}
