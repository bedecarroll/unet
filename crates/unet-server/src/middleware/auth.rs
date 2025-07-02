//! Authentication Middleware for Axum

use axum::{
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unet_core::auth::{AuthError, AuthService};
use unet_core::models::auth::{Claims, UserResponse};

/// Authentication context that gets added to request extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user: UserResponse,
    pub claims: Claims,
}

/// Authentication error response
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub message: String,
}

impl AuthContext {
    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.claims.is_admin || self.claims.permissions.contains(&permission.to_string())
    }

    /// Check if user has any of the specified permissions
    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        if self.claims.is_admin {
            return true;
        }

        permissions.iter().any(|&perm| self.has_permission(perm))
    }

    /// Check if user has all of the specified permissions
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        if self.claims.is_admin {
            return true;
        }

        permissions.iter().all(|&perm| self.has_permission(perm))
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role_name: &str) -> bool {
        self.claims.roles.contains(&role_name.to_string())
    }

    /// Check if user is admin
    pub const fn is_admin(&self) -> bool {
        self.claims.is_admin
    }

    /// Get user ID
    pub const fn user_id(&self) -> uuid::Uuid {
        self.user.id
    }

    /// Get username
    pub fn username(&self) -> &str {
        &self.user.username
    }
}

/// JWT authentication middleware
pub async fn jwt_auth(
    State(auth_service): State<AuthService>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = auth_header.token();

    // Validate token
    let claims = match auth_service.validate_token(token) {
        Ok(claims) => claims,
        Err(AuthError::TokenExpired) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(AuthError::InvalidToken) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // For now, create a basic user response from claims
    // In a real implementation, you might want to fetch fresh user data
    let user = UserResponse {
        id: claims.sub.parse().map_err(|_| StatusCode::UNAUTHORIZED)?,
        username: claims.username.clone(),
        email: claims.email.clone(),
        full_name: None,
        is_active: true,
        is_admin: claims.is_admin,
        roles: Vec::new(), // Could be populated from claims.roles if needed
        last_login: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Add auth context to request extensions
    let auth_context = AuthContext { user, claims };
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

/// API key authentication middleware
pub async fn api_key_auth(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract API key from Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate API key
    let user = match auth_service.validate_api_key(auth_header).await {
        Ok(user) => user,
        Err(AuthError::ApiKeyNotFound) => return Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::ApiKeyInactive) => return Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::ApiKeyExpired) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Create claims-like structure for API key authentication
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        email: user.email.clone(),
        roles: user.roles.iter().map(|r| r.name.clone()).collect(),
        permissions: user
            .roles
            .iter()
            .flat_map(|r| r.permissions.clone())
            .collect(),
        is_admin: user.is_admin,
        exp: chrono::Utc::now().timestamp() + 86400, // 24 hours
        iat: chrono::Utc::now().timestamp(),
        jti: uuid::Uuid::new_v4().to_string(),
    };

    // Add auth context to request extensions
    let auth_context = AuthContext { user, claims };
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

/// Optional authentication middleware (allows unauthenticated requests)
pub async fn optional_auth(
    State(auth_service): State<AuthService>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(TypedHeader(auth_header)) = auth_header {
        let token = auth_header.token();

        if let Ok(claims) = auth_service.validate_token(token) {
            if let Ok(user_id) = claims.sub.parse() {
                let user = UserResponse {
                    id: user_id,
                    username: claims.username.clone(),
                    email: claims.email.clone(),
                    full_name: None,
                    is_active: true,
                    is_admin: claims.is_admin,
                    roles: Vec::new(),
                    last_login: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                let auth_context = AuthContext { user, claims };
                request.extensions_mut().insert(auth_context);
            }
        }
    }

    next.run(request).await
}

/// Require specific permission middleware
pub fn require_permission(
    permission: &'static str,
) -> impl Fn(
    Request,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>>
+ Clone {
    move |request: Request, next: Next| {
        let perm = permission;
        Box::pin(async move {
            let auth_context = request
                .extensions()
                .get::<AuthContext>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if !auth_context.has_permission(perm) {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(request).await)
        })
    }
}

/// Require admin middleware
pub async fn require_admin(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_context.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Require any of the specified roles middleware
pub fn require_any_role(
    roles: &'static [&'static str],
) -> impl Fn(
    Request,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>>
+ Clone {
    move |request: Request, next: Next| {
        let required_roles = roles;
        Box::pin(async move {
            let auth_context = request
                .extensions()
                .get::<AuthContext>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            let has_role = required_roles
                .iter()
                .any(|&role| auth_context.has_role(role));

            if !has_role && !auth_context.is_admin() {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(request).await)
        })
    }
}

/// Rate limiting middleware (basic implementation)
pub struct RateLimiter {
    requests: HashMap<String, (u32, chrono::DateTime<chrono::Utc>)>,
    max_requests: u32,
    window_seconds: i64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: i64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_seconds,
        }
    }

    pub fn check_rate_limit(&mut self, key: &str) -> bool {
        let now = chrono::Utc::now();

        match self.requests.get_mut(key) {
            Some((count, last_reset)) => {
                // Reset counter if window has passed
                if (now - *last_reset).num_seconds() >= self.window_seconds {
                    *count = 1;
                    *last_reset = now;
                    true
                } else if *count >= self.max_requests {
                    false
                } else {
                    *count += 1;
                    true
                }
            }
            None => {
                self.requests.insert(key.to_string(), (1, now));
                true
            }
        }
    }
}
