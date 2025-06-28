//! Alert management API handlers
//!
//! This module provides HTTP API endpoints for managing alerts, alert rules,
//! escalation policies, and notification channels.

use axum::{
    Json as ExtractJson,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};
use unet_core::{
    Result,
    alerting::{
        Alert, AlertCondition, AlertRule, AlertSeverity, AlertStatus, AlertType, AlertingConfig,
        AlertingManager, EscalationPolicy, NotificationChannel,
    },
    escalation::{EscalationConfig, EscalationEngine, EscalationStats, EscalationStatus},
    metrics::MetricsSnapshot,
    notifications::{MessagePriority, NotificationManager, NotificationMessage},
};

use super::{ServerError, ServerResult};
use crate::{
    api::{ApiError, ApiResponse},
    server::AppState,
};
use uuid::Uuid;

/// Request to create a new alert rule
#[derive(Debug, Deserialize)]
pub struct CreateAlertRuleRequest {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Alert type
    pub alert_type: String,
    /// Alert severity
    pub severity: String,
    /// Whether rule is enabled
    pub enabled: Option<bool>,
    /// Metric name for metric-based alerts
    pub metric_name: Option<String>,
    /// Alert condition configuration
    pub condition: Option<AlertConditionRequest>,
    /// Log pattern for log-based alerts
    pub log_pattern: Option<String>,
    /// Evaluation window in seconds
    pub window_seconds: Option<u64>,
    /// Number of evaluation periods before triggering
    pub evaluation_periods: Option<u32>,
    /// Notification channels for this rule
    pub notification_channels: Option<Vec<String>>,
    /// Tags for categorization
    pub tags: Option<HashMap<String, String>>,
    /// Auto-resolution timeout in seconds
    pub auto_resolve_timeout: Option<u64>,
}

/// Alert condition request format
#[derive(Debug, Deserialize)]
pub struct AlertConditionRequest {
    /// Condition type
    pub condition_type: String,
    /// Primary threshold value
    pub value: f64,
    /// Secondary value for range conditions
    pub value2: Option<f64>,
}

/// Request to create a notification channel
#[derive(Debug, Deserialize)]
pub struct CreateNotificationChannelRequest {
    /// Channel name
    pub name: String,
    /// Channel type (email, slack, webhook, pagerduty)
    pub channel_type: String,
    /// Channel configuration
    pub config: HashMap<String, String>,
    /// Whether channel is enabled
    pub enabled: Option<bool>,
    /// Rate limit configuration
    pub rate_limit: Option<RateLimitRequest>,
}

/// Rate limit configuration request
#[derive(Debug, Deserialize)]
pub struct RateLimitRequest {
    /// Maximum notifications per window
    pub max_notifications: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

/// Request to acknowledge an alert
#[derive(Debug, Deserialize)]
pub struct AcknowledgeAlertRequest {
    /// User acknowledging the alert
    pub user: String,
    /// Optional acknowledgment note
    pub note: Option<String>,
}

/// Request to test notification channels
#[derive(Debug, Deserialize)]
pub struct TestNotificationRequest {
    /// Channel IDs to test
    pub channel_ids: Vec<String>,
    /// Test message
    pub message: Option<String>,
}

/// Query parameters for listing alerts
#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    /// Filter by alert status
    pub status: Option<String>,
    /// Filter by severity
    pub severity: Option<String>,
    /// Filter by rule ID
    pub rule_id: Option<String>,
    /// Page number (0-based)
    pub page: Option<u32>,
    /// Page size
    pub limit: Option<u32>,
}

/// Query parameters for listing alert rules
#[derive(Debug, Deserialize)]
pub struct AlertRulesQuery {
    /// Filter by enabled status
    pub enabled: Option<bool>,
    /// Filter by alert type
    pub alert_type: Option<String>,
    /// Filter by severity
    pub severity: Option<String>,
    /// Page number (0-based)
    pub page: Option<u32>,
    /// Page size
    pub limit: Option<u32>,
}

/// Response with pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// Data items
    pub data: Vec<T>,
    /// Total number of items
    pub total: usize,
    /// Current page number
    pub page: u32,
    /// Page size
    pub limit: u32,
    /// Whether there are more pages
    pub has_more: bool,
}

/// Alert rule response
#[derive(Debug, Serialize)]
pub struct AlertRuleResponse {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Alert type
    pub alert_type: String,
    /// Alert severity
    pub severity: String,
    /// Whether rule is enabled
    pub enabled: bool,
    /// Metric name
    pub metric_name: Option<String>,
    /// Alert condition
    pub condition: Option<String>,
    /// Log pattern
    pub log_pattern: Option<String>,
    /// Evaluation window in seconds
    pub window_seconds: u64,
    /// Evaluation periods
    pub evaluation_periods: u32,
    /// Notification channels
    pub notification_channels: Vec<String>,
    /// Tags
    pub tags: HashMap<String, String>,
    /// Auto-resolution timeout
    pub auto_resolve_timeout: Option<u64>,
    /// Creation timestamp
    pub created_at: Option<u64>,
    /// Last modification timestamp
    pub updated_at: Option<u64>,
}

/// Get all active alerts
pub async fn get_alerts(
    State(_state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> ServerResult<Json<ApiResponse<PaginatedResponse<Alert>>>> {
    debug!("Getting alerts with query: {:?}", query);

    // In a real implementation, this would get alerts from the alerting manager
    // For now, return empty results with proper pagination structure
    let alerts = Vec::new();
    let total = 0;
    let page = query.page.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).min(100); // Cap at 100

    let response = PaginatedResponse {
        data: alerts,
        total,
        page,
        limit,
        has_more: (page + 1) * limit < total as u32,
    };

    info!(
        "Retrieved {} alerts (page {}, limit {})",
        total, page, limit
    );
    Ok(Json(ApiResponse::success(response)))
}

/// Get specific alert by ID
pub async fn get_alert(
    State(_state): State<AppState>,
    Path(alert_id): Path<String>,
) -> ServerResult<Json<ApiResponse<Alert>>> {
    debug!("Getting alert: {}", alert_id);

    // In a real implementation, this would get the alert from the alerting manager
    Err(ServerError::NotFound(format!(
        "Alert not found: {}",
        alert_id
    )))
}

/// Acknowledge an alert
pub async fn acknowledge_alert(
    State(_state): State<AppState>,
    Path(alert_id): Path<String>,
    ExtractJson(request): ExtractJson<AcknowledgeAlertRequest>,
) -> ServerResult<Json<ApiResponse<Alert>>> {
    debug!("Acknowledging alert {} by user {}", alert_id, request.user);

    // In a real implementation, this would acknowledge the alert in the alerting manager
    info!("Alert {} acknowledged by {}", alert_id, request.user);
    Err(ServerError::NotFound(format!(
        "Alert not found: {}",
        alert_id
    )))
}

/// Resolve an alert
pub async fn resolve_alert(
    State(_state): State<AppState>,
    Path(alert_id): Path<String>,
) -> ServerResult<Json<ApiResponse<Alert>>> {
    debug!("Resolving alert: {}", alert_id);

    // In a real implementation, this would resolve the alert in the alerting manager
    info!("Alert {} resolved", alert_id);
    Err(ServerError::NotFound(format!(
        "Alert not found: {}",
        alert_id
    )))
}

/// Get all alert rules
pub async fn get_alert_rules(
    State(_state): State<AppState>,
    Query(query): Query<AlertRulesQuery>,
) -> ServerResult<Json<ApiResponse<PaginatedResponse<AlertRuleResponse>>>> {
    debug!("Getting alert rules with query: {:?}", query);

    // In a real implementation, this would get rules from the alerting manager
    let rules = Vec::new();
    let total = 0;
    let page = query.page.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).min(100);

    let response = PaginatedResponse {
        data: rules,
        total,
        page,
        limit,
        has_more: (page + 1) * limit < total as u32,
    };

    info!(
        "Retrieved {} alert rules (page {}, limit {})",
        total, page, limit
    );
    Ok(Json(ApiResponse::success(response)))
}

/// Create a new alert rule
pub async fn create_alert_rule(
    State(_state): State<AppState>,
    ExtractJson(request): ExtractJson<CreateAlertRuleRequest>,
) -> ServerResult<Json<ApiResponse<AlertRuleResponse>>> {
    debug!("Creating alert rule: {}", request.name);

    // Validate the request
    if request.name.is_empty() {
        return Err(ServerError::BadRequest(
            "Rule name cannot be empty".to_string(),
        ));
    }

    // Parse alert type
    let alert_type = match request.alert_type.as_str() {
        "metric_threshold" => AlertType::MetricThreshold,
        "log_pattern" => AlertType::LogPattern,
        "system_health" => AlertType::SystemHealth,
        "business" => AlertType::Business,
        "security" => AlertType::Security,
        "custom" => AlertType::Custom,
        _ => return Err(ServerError::BadRequest("Invalid alert type".to_string())),
    };

    // Parse severity
    let severity = AlertSeverity::from_str(&request.severity)
        .map_err(|_| ServerError::BadRequest("Invalid alert severity".to_string()))?;

    // Parse condition if provided
    let condition = if let Some(cond_req) = request.condition {
        Some(match cond_req.condition_type.as_str() {
            "greater_than" => AlertCondition::GreaterThan(cond_req.value),
            "less_than" => AlertCondition::LessThan(cond_req.value),
            "equals" => AlertCondition::Equals(cond_req.value),
            "between" => {
                let value2 = cond_req.value2.ok_or_else(|| {
                    ServerError::BadRequest(
                        "Second value required for 'between' condition".to_string(),
                    )
                })?;
                AlertCondition::Between(cond_req.value, value2)
            }
            "rate_of_change" => AlertCondition::RateOfChange(cond_req.value),
            "percentage_change" => AlertCondition::PercentageChange(cond_req.value),
            _ => {
                return Err(ServerError::BadRequest(
                    "Invalid condition type".to_string(),
                ));
            }
        })
    } else {
        None
    };

    // In a real implementation, this would create the rule in the alerting manager
    let rule_id = Uuid::new_v4().to_string();
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = AlertRuleResponse {
        id: rule_id.clone(),
        name: request.name.clone(),
        description: request.description,
        alert_type: request.alert_type,
        severity: request.severity,
        enabled: request.enabled.unwrap_or(true),
        metric_name: request.metric_name,
        condition: condition.map(|c| format!("{:?}", c)),
        log_pattern: request.log_pattern,
        window_seconds: request.window_seconds.unwrap_or(300),
        evaluation_periods: request.evaluation_periods.unwrap_or(1),
        notification_channels: request.notification_channels.unwrap_or_default(),
        tags: request.tags.unwrap_or_default(),
        auto_resolve_timeout: request.auto_resolve_timeout,
        created_at: Some(current_time),
        updated_at: Some(current_time),
    };

    info!("Created alert rule: {} ({})", request.name, rule_id);
    Ok(Json(ApiResponse::success(response)))
}

/// Get specific alert rule by ID
pub async fn get_alert_rule(
    State(_state): State<AppState>,
    Path(rule_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<AlertRuleResponse>>, ApiError> {
    debug!("Getting alert rule: {}", rule_id);

    // In a real implementation, this would get the rule from the alerting manager
    Err(ApiError::not_found(&format!(
        "Alert rule not found: {}",
        rule_id
    )))
}

/// Update an alert rule
pub async fn update_alert_rule(
    State(_state): State<AppState>,
    Path(rule_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateAlertRuleRequest>,
) -> std::result::Result<Json<ApiResponse<AlertRuleResponse>>, ApiError> {
    debug!("Updating alert rule: {}", rule_id);

    // In a real implementation, this would update the rule in the alerting manager
    info!("Alert rule {} updated", rule_id);
    Err(ApiError::not_found(&format!(
        "Alert rule not found: {}",
        rule_id
    )))
}

/// Delete an alert rule
pub async fn delete_alert_rule(
    State(_state): State<AppState>,
    Path(rule_id): Path<String>,
) -> std::result::Result<StatusCode, ApiError> {
    debug!("Deleting alert rule: {}", rule_id);

    // In a real implementation, this would delete the rule from the alerting manager
    info!("Alert rule {} deleted", rule_id);
    Err(ApiError::not_found(&format!(
        "Alert rule not found: {}",
        rule_id
    )))
}

/// Get all notification channels
pub async fn get_notification_channels(
    State(_state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<NotificationChannel>>>, ApiError> {
    debug!("Getting notification channels");

    // In a real implementation, this would get channels from the notification manager
    let channels = Vec::new();

    info!("Retrieved {} notification channels", channels.len());
    Ok(Json(ApiResponse::success(channels)))
}

/// Create a new notification channel
pub async fn create_notification_channel(
    State(_state): State<AppState>,
    ExtractJson(request): ExtractJson<CreateNotificationChannelRequest>,
) -> std::result::Result<Json<ApiResponse<NotificationChannel>>, ApiError> {
    debug!("Creating notification channel: {}", request.name);

    // Validate the request
    if request.name.is_empty() {
        return Err(ApiError::bad_request("Channel name cannot be empty"));
    }

    // Validate channel type
    match request.channel_type.as_str() {
        "email" | "slack" | "webhook" | "pagerduty" => {}
        _ => return Err(ApiError::bad_request("Invalid channel type")),
    }

    // In a real implementation, this would create the channel in the notification manager
    let channel_id = Uuid::new_v4().to_string();
    let rate_limit = request
        .rate_limit
        .map(|rl| unet_core::alerting::RateLimitConfig {
            max_notifications: rl.max_notifications,
            window_seconds: rl.window_seconds,
        });

    let channel = NotificationChannel {
        id: channel_id.clone(),
        name: request.name.clone(),
        channel_type: request.channel_type,
        config: request.config,
        enabled: request.enabled.unwrap_or(true),
        rate_limit,
    };

    info!(
        "Created notification channel: {} ({})",
        request.name, channel_id
    );
    Ok(Json(ApiResponse::success(channel)))
}

/// Test notification channels
pub async fn test_notifications(
    State(_state): State<AppState>,
    ExtractJson(request): ExtractJson<TestNotificationRequest>,
) -> std::result::Result<Json<ApiResponse<HashMap<String, bool>>>, ApiError> {
    debug!("Testing notification channels: {:?}", request.channel_ids);

    // In a real implementation, this would send test notifications
    let mut results = HashMap::new();
    for channel_id in &request.channel_ids {
        // Simulate test result
        results.insert(channel_id.clone(), true);
    }

    info!("Tested {} notification channels", request.channel_ids.len());
    Ok(Json(ApiResponse::success(results)))
}

/// Get escalation statistics
pub async fn get_escalation_stats(
    State(_state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<EscalationStats>>, ApiError> {
    debug!("Getting escalation statistics");

    // In a real implementation, this would get stats from the escalation engine
    let stats = EscalationStats {
        total_escalations: 0,
        active_escalations: 0,
        completed_escalations: 0,
        failed_escalations: 0,
        max_level_reached: 0,
        avg_escalation_level: 0.0,
        policies_count: 0,
        policy_assignments_count: 0,
    };

    Ok(Json(ApiResponse::success(stats)))
}

/// Get all escalation policies
pub async fn get_escalation_policies(
    State(_state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<EscalationPolicy>>>, ApiError> {
    debug!("Getting escalation policies");

    // In a real implementation, this would get policies from the escalation engine
    let policies = Vec::new();

    info!("Retrieved {} escalation policies", policies.len());
    Ok(Json(ApiResponse::success(policies)))
}

/// Create a new escalation policy
pub async fn create_escalation_policy(
    State(_state): State<AppState>,
    ExtractJson(policy): ExtractJson<EscalationPolicy>,
) -> std::result::Result<Json<ApiResponse<EscalationPolicy>>, ApiError> {
    debug!("Creating escalation policy: {}", policy.name);

    // Validate the policy
    if policy.name.is_empty() {
        return Err(ApiError::bad_request("Policy name cannot be empty"));
    }

    if policy.steps.is_empty() {
        return Err(ApiError::bad_request("Policy must have at least one step"));
    }

    // In a real implementation, this would create the policy in the escalation engine
    info!("Created escalation policy: {} ({})", policy.name, policy.id);
    Ok(Json(ApiResponse::success(policy)))
}

/// Manually escalate an alert
pub async fn manual_escalate_alert(
    State(_state): State<AppState>,
    Path(alert_id): Path<String>,
) -> std::result::Result<StatusCode, ApiError> {
    debug!("Manually escalating alert: {}", alert_id);

    // In a real implementation, this would trigger manual escalation
    info!("Alert {} manually escalated", alert_id);
    Ok(StatusCode::OK)
}

/// Get alerting system configuration
pub async fn get_alerting_config(
    State(_state): State<AppState>,
) -> ServerResult<Json<ApiResponse<AlertingConfig>>> {
    debug!("Getting alerting configuration");

    // In a real implementation, this would get the current alerting configuration
    let config = AlertingConfig {
        enabled: true,
        default_channels: vec!["email".to_string()],
        evaluation_interval: 60,
        max_active_alerts: 1000,
        auto_resolve_timeout: 3600,
        enable_grouping: true,
        grouping_window: 300,
    };

    Ok(Json(ApiResponse::success(config)))
}

/// Update alerting system configuration
pub async fn update_alerting_config(
    State(_state): State<AppState>,
    ExtractJson(config): ExtractJson<AlertingConfig>,
) -> ServerResult<Json<ApiResponse<AlertingConfig>>> {
    debug!("Updating alerting configuration");

    // Validate configuration
    if config.evaluation_interval < 10 {
        return Err(ServerError::BadRequest(
            "Evaluation interval must be at least 10 seconds".to_string(),
        ));
    }

    if config.max_active_alerts == 0 {
        return Err(ServerError::BadRequest(
            "Max active alerts must be greater than 0".to_string(),
        ));
    }

    // In a real implementation, this would update the alerting configuration
    info!("Alerting configuration updated");
    Ok(Json(ApiResponse::success(config)))
}

/// Get alert history and statistics
pub async fn get_alert_statistics(
    State(_state): State<AppState>,
) -> ServerResult<Json<ApiResponse<AlertStatistics>>> {
    debug!("Getting alert statistics");

    // In a real implementation, this would calculate statistics from the alerting manager
    let stats = AlertStatistics {
        total_alerts: 0,
        active_alerts: 0,
        acknowledged_alerts: 0,
        resolved_alerts: 0,
        critical_alerts: 0,
        high_alerts: 0,
        medium_alerts: 0,
        low_alerts: 0,
        avg_resolution_time_minutes: 0.0,
        alerts_by_rule: HashMap::new(),
        alerts_last_24h: 0,
        false_positive_rate: 0.0,
    };

    Ok(Json(ApiResponse::success(stats)))
}

/// Alert statistics response
#[derive(Debug, Serialize)]
pub struct AlertStatistics {
    /// Total number of alerts
    pub total_alerts: u64,
    /// Number of active alerts
    pub active_alerts: u64,
    /// Number of acknowledged alerts
    pub acknowledged_alerts: u64,
    /// Number of resolved alerts
    pub resolved_alerts: u64,
    /// Number of critical alerts
    pub critical_alerts: u64,
    /// Number of high priority alerts
    pub high_alerts: u64,
    /// Number of medium priority alerts
    pub medium_alerts: u64,
    /// Number of low priority alerts
    pub low_alerts: u64,
    /// Average resolution time in minutes
    pub avg_resolution_time_minutes: f64,
    /// Alert count by rule ID
    pub alerts_by_rule: HashMap<String, u64>,
    /// Alerts in the last 24 hours
    pub alerts_last_24h: u64,
    /// False positive rate (0.0 to 1.0)
    pub false_positive_rate: f64,
}
