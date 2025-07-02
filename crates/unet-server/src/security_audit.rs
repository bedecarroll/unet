//! Security audit logging system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Security event types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEventType {
    /// Authentication events
    AuthenticationSuccess,
    AuthenticationFailure,
    TokenValidationFailure,
    ApiKeyUsage,
    ApiKeyCreated,
    ApiKeyDeleted,
    PasswordChanged,

    /// Authorization events
    PermissionDenied,
    RoleAssigned,
    RoleRevoked,
    PrivilegeEscalationAttempt,

    /// Access control events
    ResourceAccessed,
    ResourceModified,
    ResourceDeleted,
    UnauthorizedAccess,
    SuspiciousActivity,

    /// Rate limiting and DOS protection
    RateLimitExceeded,
    DosAttackDetected,
    IpBlocked,
    IpUnblocked,

    /// System security events
    ConfigurationChanged,
    SecurityPolicyUpdated,
    TlsCertificateIssue,
    SecurityScanDetected,

    /// Administrative events
    AdminActionPerformed,
    UserCreated,
    UserDeleted,
    UserStatusChanged,
    SystemMaintenance,
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    /// Type of security event
    pub event_type: SecurityEventType,
    /// User ID if authenticated
    pub user_id: Option<Uuid>,
    /// Username if available
    pub username: Option<String>,
    /// Source IP address
    pub source_ip: Option<IpAddr>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Resource that was accessed or modified
    pub resource: Option<String>,
    /// Action that was performed
    pub action: Option<String>,
    /// Event description
    pub description: String,
    /// Additional context data
    pub metadata: HashMap<String, serde_json::Value>,
    /// Risk level of the event
    pub risk_level: RiskLevel,
    /// Whether this event requires immediate attention
    pub requires_investigation: bool,
    /// Session ID if available
    pub session_id: Option<String>,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(event_type: SecurityEventType, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            user_id: None,
            username: None,
            source_ip: None,
            user_agent: None,
            resource: None,
            action: None,
            description,
            metadata: HashMap::new(),
            risk_level: RiskLevel::Info,
            requires_investigation: false,
            session_id: None,
        }
    }

    /// Set user context
    pub fn with_user(mut self, user_id: Uuid, username: String) -> Self {
        self.user_id = Some(user_id);
        self.username = Some(username);
        self
    }

    /// Set source IP
    pub const fn with_source_ip(mut self, ip: IpAddr) -> Self {
        self.source_ip = Some(ip);
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set resource information
    pub fn with_resource(mut self, resource: String, action: String) -> Self {
        self.resource = Some(resource);
        self.action = Some(action);
        self
    }

    /// Set risk level
    pub const fn with_risk_level(mut self, risk_level: RiskLevel) -> Self {
        self.risk_level = risk_level;
        self
    }

    /// Mark as requiring investigation
    pub const fn requires_investigation(mut self) -> Self {
        self.requires_investigation = true;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Security audit configuration
#[derive(Debug, Clone)]
pub struct SecurityAuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log file path (None for stdout)
    pub log_file: Option<String>,
    /// Maximum events to keep in memory
    pub max_events_in_memory: usize,
    /// Enable real-time alerting for critical events
    pub enable_alerting: bool,
    /// Minimum risk level to log
    pub min_risk_level: RiskLevel,
    /// Retention period in days
    pub retention_days: u32,
}

impl Default for SecurityAuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_file: Some("security_audit.log".to_string()),
            max_events_in_memory: 10000,
            enable_alerting: true,
            min_risk_level: RiskLevel::Info,
            retention_days: 90,
        }
    }
}

/// Security audit logger
#[derive(Debug)]
pub struct SecurityAuditLogger {
    config: SecurityAuditConfig,
    events: RwLock<Vec<SecurityEvent>>,
    alert_handlers: Vec<Box<dyn AlertHandler + Send + Sync>>,
}

/// Alert handler trait for security events
pub trait AlertHandler: std::fmt::Debug {
    /// Handle a security alert
    fn handle_alert(&self, event: &SecurityEvent) -> Result<(), SecurityAuditError>;
}

/// Email alert handler
#[derive(Debug)]
pub struct EmailAlertHandler {
    recipients: Vec<String>,
}

impl EmailAlertHandler {
    pub const fn new(recipients: Vec<String>) -> Self {
        Self { recipients }
    }
}

impl AlertHandler for EmailAlertHandler {
    fn handle_alert(&self, event: &SecurityEvent) -> Result<(), SecurityAuditError> {
        // In a real implementation, this would send an email
        warn!(
            "Security alert: {} - {} (recipients: {:?})",
            event.event_type.as_ref(),
            event.description,
            self.recipients
        );
        Ok(())
    }
}

/// Webhook alert handler
#[derive(Debug)]
pub struct WebhookAlertHandler {
    webhook_url: String,
    client: reqwest::Client,
}

impl WebhookAlertHandler {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

impl AlertHandler for WebhookAlertHandler {
    fn handle_alert(&self, event: &SecurityEvent) -> Result<(), SecurityAuditError> {
        // In a real implementation, this would send a webhook
        warn!(
            "Security webhook alert: {} - {} (URL: {})",
            event.event_type.as_ref(),
            event.description,
            self.webhook_url
        );
        Ok(())
    }
}

/// Security audit error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityAuditError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Alert handler error: {0}")]
    AlertHandlerError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl SecurityAuditLogger {
    /// Create a new security audit logger
    pub fn new(config: SecurityAuditConfig) -> Self {
        Self {
            config,
            events: RwLock::new(Vec::new()),
            alert_handlers: Vec::new(),
        }
    }

    /// Add an alert handler
    pub fn add_alert_handler(mut self, handler: Box<dyn AlertHandler + Send + Sync>) -> Self {
        self.alert_handlers.push(handler);
        self
    }

    /// Log a security event
    pub async fn log_event(&self, event: SecurityEvent) -> Result<(), SecurityAuditError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check if event meets minimum risk level
        if event.risk_level < self.config.min_risk_level {
            return Ok(());
        }

        // Log to structured logger
        self.log_structured_event(&event).await;

        // Store in memory (with rotation)
        {
            let mut events = self.events.write().await;
            events.push(event.clone());

            // Rotate if necessary
            if events.len() > self.config.max_events_in_memory {
                events.remove(0);
            }
        }

        // Handle alerts for critical events
        if self.config.enable_alerting && event.requires_investigation {
            self.send_alerts(&event).await;
        }

        Ok(())
    }

    /// Log structured event to tracing system
    async fn log_structured_event(&self, event: &SecurityEvent) {
        let event_json = serde_json::to_string(event)
            .unwrap_or_else(|_| "Failed to serialize event".to_string());

        match event.risk_level {
            RiskLevel::Critical => {
                error!(
                    event_type = ?event.event_type,
                    user_id = ?event.user_id,
                    source_ip = ?event.source_ip,
                    risk_level = ?event.risk_level,
                    requires_investigation = event.requires_investigation,
                    event_data = event_json,
                    "SECURITY EVENT: {}",
                    event.description
                );
            }
            RiskLevel::High => {
                warn!(
                    event_type = ?event.event_type,
                    user_id = ?event.user_id,
                    source_ip = ?event.source_ip,
                    risk_level = ?event.risk_level,
                    requires_investigation = event.requires_investigation,
                    event_data = event_json,
                    "SECURITY EVENT: {}",
                    event.description
                );
            }
            _ => {
                info!(
                    event_type = ?event.event_type,
                    user_id = ?event.user_id,
                    source_ip = ?event.source_ip,
                    risk_level = ?event.risk_level,
                    requires_investigation = event.requires_investigation,
                    event_data = event_json,
                    "SECURITY EVENT: {}",
                    event.description
                );
            }
        }
    }

    /// Send alerts for critical events
    async fn send_alerts(&self, event: &SecurityEvent) {
        for handler in &self.alert_handlers {
            if let Err(e) = handler.handle_alert(event) {
                error!("Failed to send security alert: {}", e);
            }
        }
    }

    /// Get recent security events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<SecurityEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get events by type
    pub async fn get_events_by_type(&self, event_type: SecurityEventType) -> Vec<SecurityEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| {
                std::mem::discriminant(&e.event_type) == std::mem::discriminant(&event_type)
            })
            .cloned()
            .collect()
    }

    /// Get events by user
    pub async fn get_events_by_user(&self, user_id: Uuid) -> Vec<SecurityEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.user_id == Some(user_id))
            .cloned()
            .collect()
    }

    /// Get events by IP address
    pub async fn get_events_by_ip(&self, ip: IpAddr) -> Vec<SecurityEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.source_ip == Some(ip))
            .cloned()
            .collect()
    }

    /// Get events requiring investigation
    pub async fn get_events_requiring_investigation(&self) -> Vec<SecurityEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.requires_investigation)
            .cloned()
            .collect()
    }

    /// Generate security summary report
    pub async fn generate_summary_report(&self) -> SecuritySummaryReport {
        let events = self.events.read().await;

        let total_events = events.len();
        let critical_events = events
            .iter()
            .filter(|e| e.risk_level == RiskLevel::Critical)
            .count();
        let high_risk_events = events
            .iter()
            .filter(|e| e.risk_level == RiskLevel::High)
            .count();
        let events_requiring_investigation =
            events.iter().filter(|e| e.requires_investigation).count();

        let mut event_type_counts = HashMap::new();
        let mut top_source_ips = HashMap::new();
        for event in events.iter() {
            let key = format!("{:?}", event.event_type);
            *event_type_counts.entry(key).or_insert(0) += 1;

            if let Some(ip) = event.source_ip {
                *top_source_ips.entry(ip).or_insert(0) += 1;
            }
        }
        drop(events);

        SecuritySummaryReport {
            total_events,
            critical_events,
            high_risk_events,
            events_requiring_investigation,
            event_type_counts,
            top_source_ips,
            report_generated_at: Utc::now(),
        }
    }
}

/// Security summary report
#[derive(Debug, Serialize, Deserialize)]
pub struct SecuritySummaryReport {
    pub total_events: usize,
    pub critical_events: usize,
    pub high_risk_events: usize,
    pub events_requiring_investigation: usize,
    pub event_type_counts: HashMap<String, usize>,
    pub top_source_ips: HashMap<IpAddr, usize>,
    pub report_generated_at: DateTime<Utc>,
}

/// Helper trait for SecurityEventType serialization
impl AsRef<str> for SecurityEventType {
    fn as_ref(&self) -> &str {
        match self {
            Self::AuthenticationSuccess => "authentication_success",
            Self::AuthenticationFailure => "authentication_failure",
            Self::TokenValidationFailure => "token_validation_failure",
            Self::ApiKeyUsage => "api_key_usage",
            Self::ApiKeyCreated => "api_key_created",
            Self::ApiKeyDeleted => "api_key_deleted",
            Self::PasswordChanged => "password_changed",
            Self::PermissionDenied => "permission_denied",
            Self::RoleAssigned => "role_assigned",
            Self::RoleRevoked => "role_revoked",
            Self::PrivilegeEscalationAttempt => "privilege_escalation_attempt",
            Self::ResourceAccessed => "resource_accessed",
            Self::ResourceModified => "resource_modified",
            Self::ResourceDeleted => "resource_deleted",
            Self::UnauthorizedAccess => "unauthorized_access",
            Self::SuspiciousActivity => "suspicious_activity",
            Self::RateLimitExceeded => "rate_limit_exceeded",
            Self::DosAttackDetected => "dos_attack_detected",
            Self::IpBlocked => "ip_blocked",
            Self::IpUnblocked => "ip_unblocked",
            Self::ConfigurationChanged => "configuration_changed",
            Self::SecurityPolicyUpdated => "security_policy_updated",
            Self::TlsCertificateIssue => "tls_certificate_issue",
            Self::SecurityScanDetected => "security_scan_detected",
            Self::AdminActionPerformed => "admin_action_performed",
            Self::UserCreated => "user_created",
            Self::UserDeleted => "user_deleted",
            Self::UserStatusChanged => "user_status_changed",
            Self::SystemMaintenance => "system_maintenance",
        }
    }
}

/// Global security audit logger instance
static SECURITY_AUDIT_LOGGER: tokio::sync::OnceCell<Arc<SecurityAuditLogger>> =
    tokio::sync::OnceCell::const_new();

/// Initialize the global security audit logger
pub async fn init_security_audit_logger(config: SecurityAuditConfig) -> Arc<SecurityAuditLogger> {
    SECURITY_AUDIT_LOGGER
        .get_or_init(|| async {
            let logger = SecurityAuditLogger::new(config)
                .add_alert_handler(Box::new(EmailAlertHandler::new(vec![
                    "security@example.com".to_string(),
                ])))
                .add_alert_handler(Box::new(WebhookAlertHandler::new(
                    "https://hooks.slack.com/services/security".to_string(),
                )));

            Arc::new(logger)
        })
        .await
        .clone()
}

/// Get the global security audit logger
pub async fn get_security_audit_logger() -> Option<Arc<SecurityAuditLogger>> {
    SECURITY_AUDIT_LOGGER.get().cloned()
}

/// Convenience function to log a security event
pub async fn log_security_event(event: SecurityEvent) {
    if let Some(logger) = get_security_audit_logger().await {
        if let Err(e) = logger.log_event(event).await {
            error!("Failed to log security event: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_event_creation() {
        let event = SecurityEvent::new(
            SecurityEventType::AuthenticationFailure,
            "User login failed".to_string(),
        )
        .with_user(Uuid::new_v4(), "testuser".to_string())
        .with_source_ip("192.168.1.1".parse().unwrap())
        .with_risk_level(RiskLevel::Medium)
        .requires_investigation();

        assert_eq!(event.risk_level, RiskLevel::Medium);
        assert!(event.requires_investigation);
        assert!(event.user_id.is_some());
        assert!(event.source_ip.is_some());
    }

    #[tokio::test]
    async fn test_security_audit_logger() {
        let config = SecurityAuditConfig::default();
        let logger = SecurityAuditLogger::new(config);

        let event = SecurityEvent::new(
            SecurityEventType::AuthenticationSuccess,
            "User login successful".to_string(),
        );

        logger.log_event(event).await.unwrap();

        let events = logger.get_recent_events(10).await;
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].event_type,
            SecurityEventType::AuthenticationSuccess
        ));
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let config = SecurityAuditConfig::default();
        let logger = SecurityAuditLogger::new(config);

        let user_id = Uuid::new_v4();
        let ip = "192.168.1.1".parse().unwrap();

        // Add various events
        logger
            .log_event(
                SecurityEvent::new(
                    SecurityEventType::AuthenticationSuccess,
                    "User login".to_string(),
                )
                .with_user(user_id, "testuser".to_string())
                .with_source_ip(ip),
            )
            .await
            .unwrap();

        logger
            .log_event(
                SecurityEvent::new(
                    SecurityEventType::PermissionDenied,
                    "Access denied".to_string(),
                )
                .with_source_ip(ip),
            )
            .await
            .unwrap();

        // Test filtering
        let user_events = logger.get_events_by_user(user_id).await;
        assert_eq!(user_events.len(), 1);

        let ip_events = logger.get_events_by_ip(ip).await;
        assert_eq!(ip_events.len(), 2);
    }
}
