//! Security audit middleware integration

use axum::{
    extract::{ConnectInfo, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use tracing::error;

use crate::security_audit::{RiskLevel, SecurityEvent, SecurityEventType, log_security_event};

/// Security audit middleware that logs all requests
pub async fn security_audit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Process request
    let response = next.run(request).await;
    let status = response.status();
    let duration = start_time.elapsed();

    // Log security events based on response
    tokio::spawn(async move {
        let source_ip = addr.ip();

        // Determine event type and risk level based on status and URI
        let (event_type, risk_level, requires_investigation) = match status {
            StatusCode::UNAUTHORIZED => (
                SecurityEventType::AuthenticationFailure,
                RiskLevel::Medium,
                false,
            ),
            StatusCode::FORBIDDEN => (
                SecurityEventType::PermissionDenied,
                RiskLevel::Medium,
                false,
            ),
            StatusCode::TOO_MANY_REQUESTS => {
                (SecurityEventType::RateLimitExceeded, RiskLevel::High, true)
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                (SecurityEventType::SuspiciousActivity, RiskLevel::High, true)
            }
            _ if status.is_success() => {
                // Log successful resource access for sensitive endpoints
                if is_sensitive_endpoint(&uri.path()) {
                    (SecurityEventType::ResourceAccessed, RiskLevel::Low, false)
                } else {
                    return; // Don't log normal successful requests
                }
            }
            _ => return, // Don't log other status codes
        };

        let description = format!(
            "{} {} - {} ({}ms)",
            method,
            uri,
            status,
            duration.as_millis()
        );

        let mut event = SecurityEvent::new(event_type, description)
            .with_source_ip(source_ip)
            .with_resource(uri.path().to_string(), method.to_string())
            .with_risk_level(risk_level);

        if let Some(ua) = user_agent {
            event = event.with_user_agent(ua);
        }

        if requires_investigation {
            event = event.requires_investigation();
        }

        // Add metadata
        event = event
            .with_metadata(
                "response_time_ms".to_string(),
                (duration.as_millis() as u64).into(),
            )
            .with_metadata("status_code".to_string(), status.as_u16().into());

        log_security_event(event).await;
    });

    Ok(response)
}

/// Check if an endpoint is considered sensitive
fn is_sensitive_endpoint(path: &str) -> bool {
    let sensitive_patterns = [
        "/api/v1/auth/",
        "/api/v1/users/",
        "/api/v1/roles/",
        "/api/v1/api-keys/",
        "/api/v1/changes/",
        "/api/v1/git/",
        "/api/v1/certificates/",
    ];

    sensitive_patterns
        .iter()
        .any(|pattern| path.starts_with(pattern))
}

/// Helper functions for logging specific security events from handlers
pub mod audit_helpers {
    use super::*;
    use uuid::Uuid;

    /// Log successful authentication
    pub async fn log_authentication_success(
        user_id: Uuid,
        username: String,
        source_ip: std::net::IpAddr,
    ) {
        let event = SecurityEvent::new(
            SecurityEventType::AuthenticationSuccess,
            format!("User '{}' successfully authenticated", username),
        )
        .with_user(user_id, username)
        .with_source_ip(source_ip)
        .with_risk_level(RiskLevel::Info);

        log_security_event(event).await;
    }

    /// Log failed authentication
    pub async fn log_authentication_failure(
        username: String,
        source_ip: std::net::IpAddr,
        reason: String,
    ) {
        let event = SecurityEvent::new(
            SecurityEventType::AuthenticationFailure,
            format!("Authentication failed for user '{}': {}", username, reason),
        )
        .with_source_ip(source_ip)
        .with_risk_level(RiskLevel::Medium)
        .with_metadata("attempted_username".to_string(), username.into())
        .with_metadata("failure_reason".to_string(), reason.into());

        log_security_event(event).await;
    }

    /// Log API key usage
    pub async fn log_api_key_usage(
        api_key_name: String,
        user_id: Uuid,
        source_ip: std::net::IpAddr,
        endpoint: String,
    ) {
        let event = SecurityEvent::new(
            SecurityEventType::ApiKeyUsage,
            format!("API key '{}' used to access {}", api_key_name, endpoint),
        )
        .with_user(user_id, format!("api_key:{}", api_key_name))
        .with_source_ip(source_ip)
        .with_resource(endpoint, "API_KEY".to_string())
        .with_risk_level(RiskLevel::Info);

        log_security_event(event).await;
    }

    /// Log permission denied
    pub async fn log_permission_denied(
        user_id: Option<Uuid>,
        username: Option<String>,
        source_ip: std::net::IpAddr,
        required_permission: String,
        resource: String,
    ) {
        let mut event = SecurityEvent::new(
            SecurityEventType::PermissionDenied,
            format!(
                "Access denied to '{}' - requires permission '{}'",
                resource, required_permission
            ),
        )
        .with_source_ip(source_ip)
        .with_resource(resource, "ACCESS".to_string())
        .with_risk_level(RiskLevel::Medium)
        .with_metadata(
            "required_permission".to_string(),
            required_permission.into(),
        );

        if let (Some(uid), Some(uname)) = (user_id, username) {
            event = event.with_user(uid, uname);
        }

        log_security_event(event).await;
    }

    /// Log user creation
    pub async fn log_user_created(
        admin_user_id: Uuid,
        admin_username: String,
        new_user_id: Uuid,
        new_username: String,
        source_ip: std::net::IpAddr,
    ) {
        let event = SecurityEvent::new(
            SecurityEventType::UserCreated,
            format!(
                "User '{}' created by admin '{}'",
                new_username, admin_username
            ),
        )
        .with_user(admin_user_id, admin_username)
        .with_source_ip(source_ip)
        .with_risk_level(RiskLevel::Medium)
        .with_metadata("new_user_id".to_string(), new_user_id.to_string().into())
        .with_metadata("new_username".to_string(), new_username.into());

        log_security_event(event).await;
    }

    /// Log configuration change
    pub async fn log_configuration_change(
        user_id: Uuid,
        username: String,
        source_ip: std::net::IpAddr,
        resource: String,
        change_type: String,
        description: String,
    ) {
        let event = SecurityEvent::new(
            SecurityEventType::ConfigurationChanged,
            format!("Configuration change: {} - {}", change_type, description),
        )
        .with_user(user_id, username)
        .with_source_ip(source_ip)
        .with_resource(resource, change_type)
        .with_risk_level(RiskLevel::High)
        .requires_investigation()
        .with_metadata("change_description".to_string(), description.into());

        log_security_event(event).await;
    }

    /// Log suspicious activity
    pub async fn log_suspicious_activity(
        source_ip: std::net::IpAddr,
        activity_type: String,
        description: String,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) {
        let mut event = SecurityEvent::new(
            SecurityEventType::SuspiciousActivity,
            format!(
                "Suspicious activity detected: {} - {}",
                activity_type, description
            ),
        )
        .with_source_ip(source_ip)
        .with_risk_level(RiskLevel::High)
        .requires_investigation()
        .with_metadata("activity_type".to_string(), activity_type.into());

        for (key, value) in metadata {
            event = event.with_metadata(key, value);
        }

        log_security_event(event).await;
    }

    /// Log DOS attack detection
    pub async fn log_dos_attack(source_ip: std::net::IpAddr, attack_details: String) {
        let event = SecurityEvent::new(
            SecurityEventType::DosAttackDetected,
            format!("DOS attack detected from {}: {}", source_ip, attack_details),
        )
        .with_source_ip(source_ip)
        .with_risk_level(RiskLevel::Critical)
        .requires_investigation()
        .with_metadata("attack_details".to_string(), attack_details.into());

        log_security_event(event).await;
    }

    /// Log IP blocking
    pub async fn log_ip_blocked(source_ip: std::net::IpAddr, reason: String, duration: String) {
        let event = SecurityEvent::new(
            SecurityEventType::IpBlocked,
            format!("IP {} blocked for {}: {}", source_ip, duration, reason),
        )
        .with_source_ip(source_ip)
        .with_risk_level(RiskLevel::High)
        .with_metadata("block_reason".to_string(), reason.into())
        .with_metadata("block_duration".to_string(), duration.into());

        log_security_event(event).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_endpoint_detection() {
        assert!(is_sensitive_endpoint("/api/v1/auth/login"));
        assert!(is_sensitive_endpoint("/api/v1/users/123"));
        assert!(is_sensitive_endpoint("/api/v1/changes/456"));
        assert!(!is_sensitive_endpoint("/api/v1/nodes"));
        assert!(!is_sensitive_endpoint("/health"));
    }
}
