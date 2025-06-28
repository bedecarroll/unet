//! Alerting and notification system for μNet Core
//!
//! This module provides comprehensive alerting capabilities including:
//! - Configurable alerting rules and thresholds
//! - Multiple notification channels (email, Slack, webhook, PagerDuty)
//! - Escalation procedures for unacknowledged alerts
//! - Alert management and tracking

use crate::config::{AlertChannel, LogAlertRule};
use crate::error::{Error, Result};
use crate::metrics::MetricsSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alerts
    Info,
    /// Low priority alerts
    Low,
    /// Medium priority alerts
    Medium,
    /// High priority alerts
    High,
    /// Critical alerts requiring immediate attention
    Critical,
}

impl AlertSeverity {
    /// Convert string to AlertSeverity
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "info" | "information" => Ok(AlertSeverity::Info),
            "low" => Ok(AlertSeverity::Low),
            "medium" | "med" => Ok(AlertSeverity::Medium),
            "high" => Ok(AlertSeverity::High),
            "critical" | "crit" => Ok(AlertSeverity::Critical),
            _ => Err(Error::validation(
                "invalid_severity",
                &format!("Invalid alert severity: {}", s),
            )),
        }
    }

    /// Convert AlertSeverity to string
    pub fn to_string(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Low => "low",
            AlertSeverity::Medium => "medium",
            AlertSeverity::High => "high",
            AlertSeverity::Critical => "critical",
        }
    }
}

/// Alert status tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is active and unacknowledged
    Active,
    /// Alert has been acknowledged by an operator
    Acknowledged,
    /// Alert condition is resolved
    Resolved,
    /// Alert has been manually suppressed
    Suppressed,
}

/// Alert types supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    /// Metric threshold violations
    MetricThreshold,
    /// Log pattern matching alerts
    LogPattern,
    /// System health alerts
    SystemHealth,
    /// Business logic alerts
    Business,
    /// Security-related alerts
    Security,
    /// Custom user-defined alerts
    Custom,
}

/// Alert condition types for metric-based alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Value is greater than threshold
    GreaterThan(f64),
    /// Value is less than threshold
    LessThan(f64),
    /// Value equals threshold (exact match)
    Equals(f64),
    /// Value is between two thresholds
    Between(f64, f64),
    /// Rate of change exceeds threshold
    RateOfChange(f64),
    /// Percentage change exceeds threshold
    PercentageChange(f64),
}

impl AlertCondition {
    /// Evaluate the condition against a value
    pub fn evaluate(&self, value: f64, previous_value: Option<f64>) -> bool {
        match self {
            AlertCondition::GreaterThan(threshold) => value > *threshold,
            AlertCondition::LessThan(threshold) => value < *threshold,
            AlertCondition::Equals(threshold) => (value - threshold).abs() < f64::EPSILON,
            AlertCondition::Between(low, high) => value >= *low && value <= *high,
            AlertCondition::RateOfChange(threshold) => {
                if let Some(prev) = previous_value {
                    let rate = value - prev;
                    rate.abs() > *threshold
                } else {
                    false
                }
            }
            AlertCondition::PercentageChange(threshold) => {
                if let Some(prev) = previous_value {
                    if prev != 0.0 {
                        let change = ((value - prev) / prev * 100.0).abs();
                        change > *threshold
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

/// Alerting rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique rule identifier
    pub id: String,
    /// Human-readable rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Metric name for metric-based alerts
    pub metric_name: Option<String>,
    /// Alert condition
    pub condition: Option<AlertCondition>,
    /// Log pattern for log-based alerts
    pub log_pattern: Option<String>,
    /// Evaluation window in seconds
    pub window_seconds: u64,
    /// Number of evaluation periods before triggering
    pub evaluation_periods: u32,
    /// Notification channels for this rule
    pub notification_channels: Vec<String>,
    /// Tags for categorization and filtering
    pub tags: HashMap<String, String>,
    /// Auto-resolution timeout in seconds
    pub auto_resolve_timeout: Option<u64>,
}

/// Active alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert identifier
    pub id: String,
    /// Rule that triggered this alert
    pub rule_id: String,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert status
    pub status: AlertStatus,
    /// Timestamp when alert was created
    pub created_at: u64,
    /// Timestamp when alert was last updated
    pub updated_at: u64,
    /// Timestamp when alert was acknowledged
    pub acknowledged_at: Option<u64>,
    /// User who acknowledged the alert
    pub acknowledged_by: Option<String>,
    /// Timestamp when alert was resolved
    pub resolved_at: Option<u64>,
    /// Current metric value that triggered the alert
    pub current_value: Option<f64>,
    /// Threshold value that was exceeded
    pub threshold_value: Option<f64>,
    /// Additional context data
    pub context: HashMap<String, String>,
    /// Escalation level (0 = initial, higher = escalated)
    pub escalation_level: u32,
    /// Last escalation timestamp
    pub last_escalation_at: Option<u64>,
}

/// Escalation policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    /// Policy identifier
    pub id: String,
    /// Policy name
    pub name: String,
    /// Escalation steps
    pub steps: Vec<EscalationStep>,
    /// Default escalation timeout in minutes
    pub default_timeout_minutes: u32,
}

/// Individual escalation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStep {
    /// Step number (0-based)
    pub step: u32,
    /// Timeout before escalating to next step (minutes)
    pub timeout_minutes: u32,
    /// Notification channels for this step
    pub notification_channels: Vec<String>,
    /// Whether this step requires acknowledgment
    pub requires_acknowledgment: bool,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel identifier
    pub id: String,
    /// Channel name
    pub name: String,
    /// Channel type (email, slack, webhook, pagerduty)
    pub channel_type: String,
    /// Channel configuration
    pub config: HashMap<String, String>,
    /// Whether the channel is enabled
    pub enabled: bool,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

/// Rate limiting configuration for notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum notifications per window
    pub max_notifications: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

/// Alert manager state and tracking
#[derive(Debug, Default)]
struct AlertManagerState {
    /// Active alerts by ID
    active_alerts: HashMap<String, Alert>,
    /// Alert rules by ID
    rules: HashMap<String, AlertRule>,
    /// Notification channels by ID
    channels: HashMap<String, NotificationChannel>,
    /// Escalation policies by ID
    escalation_policies: HashMap<String, EscalationPolicy>,
    /// Metric value history for rate calculations
    metric_history: HashMap<String, Vec<(u64, f64)>>,
    /// Channel rate limiting state
    channel_rate_limits: HashMap<String, Vec<u64>>,
}

const METRIC_HISTORY_RETENTION: usize = 1000;
const RATE_LIMIT_CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Comprehensive alerting manager
#[derive(Clone)]
pub struct AlertingManager {
    /// Internal state
    state: Arc<RwLock<AlertManagerState>>,
    /// Configuration snapshot
    config: AlertingConfig,
}

/// Alerting system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Whether alerting is enabled
    pub enabled: bool,
    /// Default notification channels
    pub default_channels: Vec<String>,
    /// Alert evaluation interval in seconds
    pub evaluation_interval: u64,
    /// Maximum number of active alerts before suppression
    pub max_active_alerts: u32,
    /// Auto-resolve timeout for unacknowledged alerts (seconds)
    pub auto_resolve_timeout: u64,
    /// Enable alert grouping
    pub enable_grouping: bool,
    /// Grouping window in seconds
    pub grouping_window: u64,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_channels: vec!["email".to_string()],
            evaluation_interval: 60,
            max_active_alerts: 1000,
            auto_resolve_timeout: 3600, // 1 hour
            enable_grouping: true,
            grouping_window: 300, // 5 minutes
        }
    }
}

impl AlertingManager {
    /// Create a new alerting manager
    pub fn new(config: AlertingConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(AlertManagerState::default())),
            config,
        }
    }

    /// Add or update an alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut state = self.state.write().await;
        state.rules.insert(rule.id.clone(), rule.clone());

        info!("Alert rule added: {} ({})", rule.name, rule.id);
        Ok(())
    }

    /// Remove an alert rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        if state.rules.remove(rule_id).is_some() {
            info!("Alert rule removed: {}", rule_id);
        }
        Ok(())
    }

    /// Add or update a notification channel
    pub async fn add_notification_channel(&self, channel: NotificationChannel) -> Result<()> {
        let mut state = self.state.write().await;
        state.channels.insert(channel.id.clone(), channel.clone());

        info!(
            "Notification channel added: {} ({})",
            channel.name, channel.id
        );
        Ok(())
    }

    /// Remove a notification channel
    pub async fn remove_notification_channel(&self, channel_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        if state.channels.remove(channel_id).is_some() {
            info!("Notification channel removed: {}", channel_id);
        }
        Ok(())
    }

    /// Add or update an escalation policy
    pub async fn add_escalation_policy(&self, policy: EscalationPolicy) -> Result<()> {
        let mut state = self.state.write().await;
        state
            .escalation_policies
            .insert(policy.id.clone(), policy.clone());

        info!("Escalation policy added: {} ({})", policy.name, policy.id);
        Ok(())
    }

    /// Evaluate alerts based on current metrics
    pub async fn evaluate_alerts(&self, metrics: &MetricsSnapshot) -> Result<Vec<Alert>> {
        if !self.config.enabled {
            return Ok(vec![]);
        }

        let mut state = self.state.write().await;
        let mut new_alerts = Vec::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update metric history
        self.update_metric_history(&mut state, metrics, current_time);

        // Evaluate each rule
        for rule in state.rules.values() {
            if !rule.enabled {
                continue;
            }

            match rule.alert_type {
                AlertType::MetricThreshold => {
                    if let Some(alert) = self
                        .evaluate_metric_rule(rule, metrics, &state, current_time)
                        .await?
                    {
                        new_alerts.push(alert);
                    }
                }
                AlertType::SystemHealth => {
                    if let Some(alert) = self
                        .evaluate_system_health_rule(rule, metrics, current_time)
                        .await?
                    {
                        new_alerts.push(alert);
                    }
                }
                _ => {
                    // Other alert types will be implemented in subsequent phases
                    debug!(
                        "Alert type {:?} evaluation not yet implemented",
                        rule.alert_type
                    );
                }
            }
        }

        // Add new alerts to active alerts
        for alert in &new_alerts {
            state.active_alerts.insert(alert.id.clone(), alert.clone());
        }

        // Clean up resolved alerts
        state.active_alerts.retain(|_, alert| {
            alert.status != AlertStatus::Resolved
                || current_time - alert.resolved_at.unwrap_or(0) < 3600 // Keep resolved alerts for 1 hour
        });

        Ok(new_alerts)
    }

    /// Evaluate a metric threshold rule
    async fn evaluate_metric_rule(
        &self,
        rule: &AlertRule,
        metrics: &MetricsSnapshot,
        state: &AlertManagerState,
        current_time: u64,
    ) -> Result<Option<Alert>> {
        let metric_name = rule.metric_name.as_ref().ok_or_else(|| {
            Error::validation(
                "missing_metric",
                "Metric name required for metric threshold rule",
            )
        })?;

        let condition = rule.condition.as_ref().ok_or_else(|| {
            Error::validation(
                "missing_condition",
                "Condition required for metric threshold rule",
            )
        })?;

        // Get current metric value
        let current_value = self.get_metric_value(metric_name, metrics)?;

        // Get previous value for rate-based conditions
        let previous_value = state
            .metric_history
            .get(metric_name)
            .and_then(|history| history.last())
            .map(|(_, value)| *value);

        // Evaluate condition
        if condition.evaluate(current_value, previous_value) {
            // Check if alert already exists and is active
            let existing_alert = state.active_alerts.values().find(|alert| {
                alert.rule_id == rule.id
                    && (alert.status == AlertStatus::Active
                        || alert.status == AlertStatus::Acknowledged)
            });

            if existing_alert.is_none() {
                // Create new alert
                let alert = Alert {
                    id: Uuid::new_v4().to_string(),
                    rule_id: rule.id.clone(),
                    title: format!("Metric Alert: {}", rule.name),
                    description: format!(
                        "Metric '{}' value {:.2} violates threshold condition",
                        metric_name, current_value
                    ),
                    severity: rule.severity,
                    status: AlertStatus::Active,
                    created_at: current_time,
                    updated_at: current_time,
                    acknowledged_at: None,
                    acknowledged_by: None,
                    resolved_at: None,
                    current_value: Some(current_value),
                    threshold_value: match condition {
                        AlertCondition::GreaterThan(t)
                        | AlertCondition::LessThan(t)
                        | AlertCondition::Equals(t) => Some(*t),
                        AlertCondition::Between(low, _) => Some(*low),
                        AlertCondition::RateOfChange(t) | AlertCondition::PercentageChange(t) => {
                            Some(*t)
                        }
                    },
                    context: HashMap::new(),
                    escalation_level: 0,
                    last_escalation_at: None,
                };

                info!(
                    "New alert triggered: {} (metric: {}, value: {:.2})",
                    alert.title, metric_name, current_value
                );
                return Ok(Some(alert));
            }
        }

        Ok(None)
    }

    /// Evaluate a system health rule
    async fn evaluate_system_health_rule(
        &self,
        rule: &AlertRule,
        metrics: &MetricsSnapshot,
        current_time: u64,
    ) -> Result<Option<Alert>> {
        // Example system health checks
        let high_cpu = metrics.system.cpu_usage_percent > 90.0;
        let high_memory = metrics.system.memory_usage_percent > 90.0;
        let no_disk_space = metrics.system.disk_usage_percent > 95.0;

        if high_cpu || high_memory || no_disk_space {
            let alert = Alert {
                id: Uuid::new_v4().to_string(),
                rule_id: rule.id.clone(),
                title: "System Health Alert".to_string(),
                description: format!(
                    "System health degraded: CPU {:.1}%, Memory {:.1}%, Disk {:.1}%",
                    metrics.system.cpu_usage_percent,
                    metrics.system.memory_usage_percent,
                    metrics.system.disk_usage_percent
                ),
                severity: if no_disk_space {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::High
                },
                status: AlertStatus::Active,
                created_at: current_time,
                updated_at: current_time,
                acknowledged_at: None,
                acknowledged_by: None,
                resolved_at: None,
                current_value: None,
                threshold_value: None,
                context: HashMap::new(),
                escalation_level: 0,
                last_escalation_at: None,
            };

            return Ok(Some(alert));
        }

        Ok(None)
    }

    /// Get metric value by name from metrics snapshot
    fn get_metric_value(&self, metric_name: &str, metrics: &MetricsSnapshot) -> Result<f64> {
        match metric_name {
            "http_requests_total" => Ok(metrics.business.http_requests_total),
            "nodes_total" => Ok(metrics.business.nodes_total),
            "users_total" => Ok(metrics.business.users_total),
            "cpu_usage_percent" => Ok(metrics.system.cpu_usage_percent),
            "memory_usage_percent" => Ok(metrics.system.memory_usage_percent),
            "memory_usage_bytes" => Ok(metrics.system.memory_usage_bytes),
            "active_connections" => Ok(metrics.system.active_connections),
            "auth_failures_total" => Ok(metrics.business.auth_failures_total),
            "policy_evaluations_total" => Ok(metrics.business.policy_evaluations_total),
            _ => Err(Error::validation(
                "unknown_metric",
                &format!("Unknown metric name: {}", metric_name),
            )),
        }
    }

    /// Update metric history for rate-based calculations
    fn update_metric_history(
        &self,
        state: &mut AlertManagerState,
        metrics: &MetricsSnapshot,
        current_time: u64,
    ) {
        let metric_values = vec![
            ("http_requests_total", metrics.business.http_requests_total),
            ("nodes_total", metrics.business.nodes_total),
            ("users_total", metrics.business.users_total),
            ("cpu_usage_percent", metrics.system.cpu_usage_percent),
            ("memory_usage_percent", metrics.system.memory_usage_percent),
            ("active_connections", metrics.system.active_connections),
        ];

        for (metric_name, value) in metric_values {
            let history = state
                .metric_history
                .entry(metric_name.to_string())
                .or_insert_with(Vec::new);
            history.push((current_time, value));

            // Keep only recent history
            if history.len() > METRIC_HISTORY_RETENTION {
                history.remove(0);
            }
        }
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str, user: &str) -> Result<()> {
        let mut state = self.state.write().await;

        if let Some(alert) = state.active_alerts.get_mut(alert_id) {
            if alert.status == AlertStatus::Active {
                alert.status = AlertStatus::Acknowledged;
                alert.acknowledged_at = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
                alert.acknowledged_by = Some(user.to_string());
                alert.updated_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                info!("Alert acknowledged: {} by {}", alert_id, user);
            }
        }

        Ok(())
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        let mut state = self.state.write().await;

        if let Some(alert) = state.active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            alert.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            info!("Alert resolved: {}", alert_id);
        }

        Ok(())
    }

    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let state = self.state.read().await;
        Ok(state
            .active_alerts
            .values()
            .filter(|alert| {
                alert.status == AlertStatus::Active || alert.status == AlertStatus::Acknowledged
            })
            .cloned()
            .collect())
    }

    /// Get all alert rules
    pub async fn get_rules(&self) -> Result<Vec<AlertRule>> {
        let state = self.state.read().await;
        Ok(state.rules.values().cloned().collect())
    }

    /// Get all notification channels
    pub async fn get_notification_channels(&self) -> Result<Vec<NotificationChannel>> {
        let state = self.state.read().await;
        Ok(state.channels.values().cloned().collect())
    }

    /// Start background alert evaluation and escalation
    pub fn start_background_processing(&self) {
        let manager = self.clone();
        let interval = Duration::from_secs(self.config.evaluation_interval);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            info!(
                "Starting background alert processing with {}s interval",
                manager.config.evaluation_interval
            );

            loop {
                interval_timer.tick().await;

                if let Err(e) = manager.process_escalations().await {
                    error!("Failed to process alert escalations: {}", e);
                }

                if let Err(e) = manager.cleanup_rate_limits().await {
                    error!("Failed to cleanup rate limits: {}", e);
                }
            }
        });
    }

    /// Process alert escalations
    async fn process_escalations(&self) -> Result<()> {
        let mut state = self.state.write().await;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for alert in state.active_alerts.values_mut() {
            if alert.status != AlertStatus::Active {
                continue;
            }

            // Check if alert needs escalation
            let escalation_timeout = 900; // 15 minutes default
            let last_escalation = alert.last_escalation_at.unwrap_or(alert.created_at);

            if current_time - last_escalation > escalation_timeout {
                alert.escalation_level += 1;
                alert.last_escalation_at = Some(current_time);
                alert.updated_at = current_time;

                info!(
                    "Alert escalated: {} to level {}",
                    alert.id, alert.escalation_level
                );
            }
        }

        Ok(())
    }

    /// Clean up old rate limit entries
    async fn cleanup_rate_limits(&self) -> Result<()> {
        let mut state = self.state.write().await;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for (_, timestamps) in state.channel_rate_limits.iter_mut() {
            timestamps.retain(|&timestamp| current_time - timestamp < 3600); // Keep 1 hour
        }

        Ok(())
    }
}

/// Pre-defined system alert rules
pub fn create_default_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            id: "high_cpu_usage".to_string(),
            name: "High CPU Usage".to_string(),
            description: "Alert when CPU usage exceeds 80%".to_string(),
            alert_type: AlertType::MetricThreshold,
            severity: AlertSeverity::High,
            enabled: true,
            metric_name: Some("cpu_usage_percent".to_string()),
            condition: Some(AlertCondition::GreaterThan(80.0)),
            log_pattern: None,
            window_seconds: 300,
            evaluation_periods: 2,
            notification_channels: vec!["email".to_string()],
            tags: HashMap::from([("category".to_string(), "system".to_string())]),
            auto_resolve_timeout: Some(1800),
        },
        AlertRule {
            id: "high_memory_usage".to_string(),
            name: "High Memory Usage".to_string(),
            description: "Alert when memory usage exceeds 85%".to_string(),
            alert_type: AlertType::MetricThreshold,
            severity: AlertSeverity::High,
            enabled: true,
            metric_name: Some("memory_usage_percent".to_string()),
            condition: Some(AlertCondition::GreaterThan(85.0)),
            log_pattern: None,
            window_seconds: 300,
            evaluation_periods: 2,
            notification_channels: vec!["email".to_string()],
            tags: HashMap::from([("category".to_string(), "system".to_string())]),
            auto_resolve_timeout: Some(1800),
        },
        AlertRule {
            id: "high_auth_failures".to_string(),
            name: "High Authentication Failures".to_string(),
            description: "Alert when authentication failure rate is high".to_string(),
            alert_type: AlertType::MetricThreshold,
            severity: AlertSeverity::Medium,
            enabled: true,
            metric_name: Some("auth_failures_total".to_string()),
            condition: Some(AlertCondition::RateOfChange(10.0)),
            log_pattern: None,
            window_seconds: 300,
            evaluation_periods: 1,
            notification_channels: vec!["email".to_string()],
            tags: HashMap::from([("category".to_string(), "security".to_string())]),
            auto_resolve_timeout: Some(3600),
        },
        AlertRule {
            id: "system_health_degraded".to_string(),
            name: "System Health Degraded".to_string(),
            description: "Alert when overall system health is degraded".to_string(),
            alert_type: AlertType::SystemHealth,
            severity: AlertSeverity::High,
            enabled: true,
            metric_name: None,
            condition: None,
            log_pattern: None,
            window_seconds: 120,
            evaluation_periods: 1,
            notification_channels: vec!["email".to_string()],
            tags: HashMap::from([("category".to_string(), "health".to_string())]),
            auto_resolve_timeout: Some(900),
        },
    ]
}

/// Pre-defined notification channels
pub fn create_default_notification_channels() -> Vec<NotificationChannel> {
    vec![
        NotificationChannel {
            id: "email".to_string(),
            name: "Default Email".to_string(),
            channel_type: "email".to_string(),
            config: HashMap::from([
                ("smtp_server".to_string(), "localhost:587".to_string()),
                (
                    "from_address".to_string(),
                    "alerts@unet.example.com".to_string(),
                ),
                ("to_addresses".to_string(), "admin@example.com".to_string()),
            ]),
            enabled: true,
            rate_limit: Some(RateLimitConfig {
                max_notifications: 10,
                window_seconds: 300,
            }),
        },
        NotificationChannel {
            id: "slack".to_string(),
            name: "Slack Alerts".to_string(),
            channel_type: "slack".to_string(),
            config: HashMap::from([
                (
                    "webhook_url".to_string(),
                    "https://hooks.slack.com/services/...".to_string(),
                ),
                ("channel".to_string(), "#alerts".to_string()),
                ("username".to_string(), "μNet Alerts".to_string()),
            ]),
            enabled: false, // Disabled by default until configured
            rate_limit: Some(RateLimitConfig {
                max_notifications: 20,
                window_seconds: 300,
            }),
        },
        NotificationChannel {
            id: "webhook".to_string(),
            name: "Generic Webhook".to_string(),
            channel_type: "webhook".to_string(),
            config: HashMap::from([
                ("url".to_string(), "https://example.com/webhook".to_string()),
                ("method".to_string(), "POST".to_string()),
                ("timeout_seconds".to_string(), "30".to_string()),
            ]),
            enabled: false, // Disabled by default until configured
            rate_limit: Some(RateLimitConfig {
                max_notifications: 50,
                window_seconds: 300,
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_severity_conversion() {
        assert_eq!(
            AlertSeverity::from_str("info").unwrap(),
            AlertSeverity::Info
        );
        assert_eq!(
            AlertSeverity::from_str("critical").unwrap(),
            AlertSeverity::Critical
        );
        assert!(AlertSeverity::from_str("invalid").is_err());
    }

    #[test]
    fn test_alert_condition_evaluation() {
        let condition = AlertCondition::GreaterThan(10.0);
        assert!(condition.evaluate(15.0, None));
        assert!(!condition.evaluate(5.0, None));

        let rate_condition = AlertCondition::RateOfChange(5.0);
        assert!(rate_condition.evaluate(15.0, Some(5.0)));
        assert!(!rate_condition.evaluate(15.0, Some(12.0)));
    }

    #[tokio::test]
    async fn test_alerting_manager_creation() {
        let config = AlertingConfig::default();
        let manager = AlertingManager::new(config);

        // Add a test rule
        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test description".to_string(),
            alert_type: AlertType::MetricThreshold,
            severity: AlertSeverity::Medium,
            enabled: true,
            metric_name: Some("cpu_usage_percent".to_string()),
            condition: Some(AlertCondition::GreaterThan(50.0)),
            log_pattern: None,
            window_seconds: 300,
            evaluation_periods: 1,
            notification_channels: vec!["email".to_string()],
            tags: HashMap::new(),
            auto_resolve_timeout: Some(1800),
        };

        manager.add_rule(rule).await.unwrap();
        let rules = manager.get_rules().await.unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "test_rule");
    }
}
