//! Middleware modules for the μNet server

pub mod auth;
pub mod performance;
pub mod rate_limit;
pub mod security_audit;
pub mod validation;

pub use auth::{
    AuthContext, AuthErrorResponse, RateLimiter, api_key_auth, jwt_auth, optional_auth,
    require_admin, require_any_role, require_permission,
};

pub use rate_limit::{
    EnhancedRateLimiter, RateLimitConfig, RateLimitResult, RateLimitStatus, rate_limit_middleware,
};

pub use security_audit::{audit_helpers, security_audit_middleware};

pub use performance::{
    performance_monitoring_middleware, time_database_operation, time_policy_evaluation,
    time_template_rendering,
};

pub use validation::{
    security_headers_middleware, validate_headers_middleware, validate_request_middleware,
    validation_error_response,
};

// Re-export network access middleware from parent module
pub use crate::network_access::{
    NetworkAccessConfig, NetworkAccessControl, NetworkAction, network_access_middleware,
};
