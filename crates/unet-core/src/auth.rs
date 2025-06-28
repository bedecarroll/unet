//! Authentication and Authorization Service

use crate::models::auth::*;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("User not active")]
    UserNotActive,
    #[error("Role not found")]
    RoleNotFound,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("API key not found")]
    ApiKeyNotFound,
    #[error("API key inactive")]
    ApiKeyInactive,
    #[error("API key expired")]
    ApiKeyExpired,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Username already exists")]
    UsernameExists,
    #[error("Email already exists")]
    EmailExists,
    #[error("Role name already exists")]
    RoleNameExists,
    #[error("Password hashing failed: {0}")]
    PasswordHashError(String),
    #[error("JWT error: {0}")]
    JwtError(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type AuthResult<T> = Result<T, AuthError>;

/// JWT token configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_ttl: Duration,
    pub algorithm: Algorithm,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-secret-key".to_string(),
            issuer: "unet".to_string(),
            audience: "unet-api".to_string(),
            access_token_ttl: Duration::hours(24),
            algorithm: Algorithm::HS256,
        }
    }
}

/// Main authentication service
#[derive(Clone)]
pub struct AuthService {
    jwt_config: JwtConfig,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(jwt_config: JwtConfig) -> Self {
        Self { jwt_config }
    }

    /// Get encoding key
    fn encoding_key(&self) -> EncodingKey {
        EncodingKey::from_secret(self.jwt_config.secret.as_ref())
    }

    /// Get decoding key
    fn decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(self.jwt_config.secret.as_ref())
    }

    /// Get validation settings
    fn validation(&self) -> Validation {
        let mut validation = Validation::new(self.jwt_config.algorithm);
        validation.set_issuer(&[&self.jwt_config.issuer]);
        validation.set_audience(&[&self.jwt_config.audience]);
        validation
    }

    /// Hash a password using bcrypt
    pub fn hash_password(&self, password: &str) -> AuthResult<String> {
        hash(password, DEFAULT_COST).map_err(|e| AuthError::PasswordHashError(e.to_string()))
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> AuthResult<bool> {
        verify(password, hash).map_err(|e| AuthError::PasswordHashError(e.to_string()))
    }

    /// Generate a JWT token for a user
    pub fn generate_token(&self, user: &UserResponse) -> AuthResult<String> {
        let now = Utc::now();
        let exp = now + self.jwt_config.access_token_ttl;

        let permissions: Vec<String> = user
            .roles
            .iter()
            .flat_map(|role| role.permissions.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let role_names: Vec<String> = user.roles.iter().map(|role| role.name.clone()).collect();

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            roles: role_names,
            permissions,
            is_admin: user.is_admin,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::new(self.jwt_config.algorithm),
            &claims,
            &self.encoding_key(),
        )
        .map_err(|e| AuthError::JwtError(e.to_string()))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> AuthResult<Claims> {
        decode::<Claims>(token, &self.decoding_key(), &self.validation())
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })
    }

    /// Authenticate a user with username/password
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> AuthResult<UserResponse> {
        // For now, create a basic admin user for demo purposes
        if username == "admin" && password == "admin" {
            let admin_permissions = system_roles::admin_permissions();
            let admin_roles = vec![RoleResponse {
                id: Uuid::new_v4(),
                name: "admin".to_string(),
                description: Some("Administrator".to_string()),
                permissions: admin_permissions,
                is_system: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }];

            return Ok(UserResponse {
                id: Uuid::new_v4(),
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                full_name: Some("Administrator".to_string()),
                is_active: true,
                is_admin: true,
                roles: admin_roles,
                last_login: Some(Utc::now()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Err(AuthError::InvalidCredentials)
    }

    /// Create a new user (placeholder implementation)
    pub async fn create_user(&self, request: CreateUserRequest) -> AuthResult<UserResponse> {
        // For now, return a basic user for demo purposes
        // TODO: Implement proper database integration

        let user_id = Uuid::new_v4();
        let roles = vec![RoleResponse {
            id: Uuid::new_v4(),
            name: "readonly".to_string(),
            description: Some("Read-only user".to_string()),
            permissions: system_roles::readonly_permissions(),
            is_system: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        Ok(UserResponse {
            id: user_id,
            username: request.username,
            email: request.email,
            full_name: request.full_name,
            is_active: true,
            is_admin: false,
            roles,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Get user by ID (placeholder implementation)
    pub async fn get_user(&self, user_id: Uuid) -> AuthResult<UserResponse> {
        // For now, return admin user for demo
        // TODO: Implement proper database integration
        let admin_permissions = system_roles::admin_permissions();
        let admin_roles = vec![RoleResponse {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: Some("Administrator".to_string()),
            permissions: admin_permissions,
            is_system: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        Ok(UserResponse {
            id: user_id,
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            full_name: Some("Administrator".to_string()),
            is_active: true,
            is_admin: true,
            roles: admin_roles,
            last_login: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Create a new role (placeholder implementation)
    pub async fn create_role(&self, request: CreateRoleRequest) -> AuthResult<RoleResponse> {
        // For now, return a role for demo purposes
        // TODO: Implement proper database integration

        Ok(RoleResponse {
            id: Uuid::new_v4(),
            name: request.name,
            description: request.description,
            permissions: request.permissions,
            is_system: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Initialize default system roles (placeholder implementation)
    pub async fn initialize_system_roles(&self) -> AuthResult<()> {
        // For demo purposes, this is a no-op
        // TODO: Implement proper database integration
        Ok(())
    }

    /// Check if user has permission (placeholder implementation)
    pub async fn check_permission(&self, _user_id: Uuid, _permission: &str) -> AuthResult<bool> {
        // For demo purposes, always return true for admin
        // TODO: Implement proper permission checking
        Ok(true)
    }

    /// Generate API key for user (placeholder implementation)
    pub async fn create_api_key(
        &self,
        _user_id: Uuid,
        request: CreateApiKeyRequest,
    ) -> AuthResult<CreateApiKeyResponse> {
        // Generate random API key
        let api_key = format!("unet_{}", Uuid::new_v4().to_string().replace('-', ""));
        let key_prefix = api_key.chars().take(8).collect::<String>();

        let details = ApiKeyResponse {
            id: Uuid::new_v4(),
            name: request.name,
            key_prefix,
            scopes: request.scopes,
            is_active: true,
            last_used: None,
            usage_count: 0,
            rate_limit: request.rate_limit,
            expires_at: request.expires_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(CreateApiKeyResponse { api_key, details })
    }

    /// Validate API key (placeholder implementation)
    pub async fn validate_api_key(&self, api_key: &str) -> AuthResult<UserResponse> {
        // For demo purposes, accept any API key starting with "unet_"
        if api_key.starts_with("unet_") {
            return self.get_user(Uuid::new_v4()).await;
        }

        Err(AuthError::ApiKeyNotFound)
    }
}
