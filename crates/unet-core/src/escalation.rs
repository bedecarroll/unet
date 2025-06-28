//! Alert escalation system for μNet Core
//!
//! This module provides escalation procedures for unacknowledged alerts,
//! including automatic escalation workflows, notification routing, and
//! escalation management.

use crate::alerting::{Alert, AlertSeverity, AlertStatus, EscalationPolicy, EscalationStep};
use crate::error::{Error, Result};
use crate::notifications::{
    DeliveryResult, MessagePriority, NotificationManager, NotificationMessage,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Escalation engine result type
pub type EscalationResult<T> = std::result::Result<T, EscalationError>;

/// Escalation system errors
#[derive(Debug, thiserror::Error)]
pub enum EscalationError {
    /// Escalation policy not found
    #[error("Escalation policy not found: {policy_id}")]
    PolicyNotFound { policy_id: String },

    /// Invalid escalation configuration
    #[error("Invalid escalation configuration: {message}")]
    InvalidConfiguration { message: String },

    /// Notification delivery failed
    #[error("Notification delivery failed: {message}")]
    NotificationFailed { message: String },

    /// Escalation timeout
    #[error("Escalation timeout for alert: {alert_id}")]
    EscalationTimeout { alert_id: String },

    /// Maximum escalation level reached
    #[error("Maximum escalation level reached for alert: {alert_id}")]
    MaxLevelReached { alert_id: String },
}

/// Alert escalation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEscalation {
    /// Alert ID
    pub alert_id: String,
    /// Current escalation level
    pub level: u32,
    /// Escalation policy ID
    pub policy_id: String,
    /// Next escalation timestamp
    pub next_escalation_at: u64,
    /// Escalation history
    pub history: Vec<EscalationEvent>,
    /// Last notification results
    pub last_notification_results: Vec<DeliveryResult>,
    /// Escalation status
    pub status: EscalationStatus,
    /// Created timestamp
    pub created_at: u64,
    /// Updated timestamp
    pub updated_at: u64,
}

/// Escalation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationStatus {
    /// Escalation is active
    Active,
    /// Escalation is paused
    Paused,
    /// Escalation is completed (alert acknowledged/resolved)
    Completed,
    /// Escalation failed due to errors
    Failed,
    /// Escalation reached maximum level
    MaxLevelReached,
}

/// Escalation event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationEvent {
    /// Event timestamp
    pub timestamp: u64,
    /// Escalation level at time of event
    pub level: u32,
    /// Event type
    pub event_type: EscalationEventType,
    /// Event description
    pub description: String,
    /// Notification channels used
    pub channels: Vec<String>,
    /// Delivery results
    pub delivery_results: Vec<DeliveryResult>,
}

/// Escalation event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationEventType {
    /// Initial alert notification
    InitialNotification,
    /// Escalation to next level
    Escalated,
    /// Alert acknowledged
    Acknowledged,
    /// Alert resolved
    Resolved,
    /// Escalation failed
    Failed,
    /// Manual escalation
    ManualEscalation,
    /// Escalation paused
    Paused,
    /// Escalation resumed
    Resumed,
}

/// Escalation engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Whether escalation is enabled
    pub enabled: bool,
    /// Default escalation policy ID
    pub default_policy_id: String,
    /// Escalation evaluation interval in seconds
    pub evaluation_interval: u64,
    /// Maximum escalation levels
    pub max_escalation_levels: u32,
    /// Grace period before first escalation (seconds)
    pub initial_grace_period: u64,
    /// Enable escalation notifications
    pub enable_notifications: bool,
    /// Escalation message templates
    pub message_templates: HashMap<String, String>,
}

impl Default for EscalationConfig {
    fn default() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            "initial".to_string(),
            "Alert: {{alert.title}} - {{alert.description}}".to_string(),
        );
        templates.insert(
            "escalation".to_string(),
            "ESCALATED (Level {{escalation.level}}): {{alert.title}} - {{alert.description}}"
                .to_string(),
        );
        templates.insert(
            "acknowledged".to_string(),
            "Alert acknowledged: {{alert.title}} - {{alert.description}}".to_string(),
        );
        templates.insert(
            "resolved".to_string(),
            "Alert resolved: {{alert.title}} - {{alert.description}}".to_string(),
        );

        Self {
            enabled: true,
            default_policy_id: "default".to_string(),
            evaluation_interval: 60,
            max_escalation_levels: 5,
            initial_grace_period: 300, // 5 minutes
            enable_notifications: true,
            message_templates: templates,
        }
    }
}

/// Escalation engine state
#[derive(Debug, Default)]
struct EscalationState {
    /// Active escalations by alert ID
    escalations: HashMap<String, AlertEscalation>,
    /// Escalation policies by ID
    policies: HashMap<String, EscalationPolicy>,
    /// Policy assignments by alert rule ID
    policy_assignments: HashMap<String, String>,
}

/// Alert escalation engine
pub struct EscalationEngine {
    /// Engine configuration
    config: EscalationConfig,
    /// Internal state
    state: Arc<RwLock<EscalationState>>,
    /// Notification manager
    notification_manager: Option<Arc<NotificationManager>>,
}

impl EscalationEngine {
    /// Create a new escalation engine
    pub fn new(config: EscalationConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(EscalationState::default())),
            notification_manager: None,
        }
    }

    /// Set the notification manager
    pub fn set_notification_manager(&mut self, manager: Arc<NotificationManager>) {
        self.notification_manager = Some(manager);
    }

    /// Add an escalation policy
    pub async fn add_policy(&self, policy: EscalationPolicy) -> Result<()> {
        let mut state = self.state.write().await;
        state.policies.insert(policy.id.clone(), policy.clone());
        info!("Added escalation policy: {}", policy.name);
        Ok(())
    }

    /// Remove an escalation policy
    pub async fn remove_policy(&self, policy_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        if state.policies.remove(policy_id).is_some() {
            info!("Removed escalation policy: {}", policy_id);
        }
        Ok(())
    }

    /// Assign escalation policy to an alert rule
    pub async fn assign_policy_to_rule(&self, rule_id: &str, policy_id: &str) -> Result<()> {
        let state = self.state.read().await;
        if !state.policies.contains_key(policy_id) {
            return Err(Error::validation(
                "policy_not_found",
                &format!("Escalation policy not found: {}", policy_id),
            ));
        }
        drop(state);

        let mut state = self.state.write().await;
        state
            .policy_assignments
            .insert(rule_id.to_string(), policy_id.to_string());

        info!(
            "Assigned escalation policy {} to rule {}",
            policy_id, rule_id
        );
        Ok(())
    }

    /// Start escalation for a new alert
    pub async fn start_escalation(&self, alert: &Alert) -> EscalationResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let state = self.state.read().await;
        let policy_id = state
            .policy_assignments
            .get(&alert.rule_id)
            .unwrap_or(&self.config.default_policy_id)
            .clone();

        let policy = state
            .policies
            .get(&policy_id)
            .ok_or_else(|| EscalationError::PolicyNotFound {
                policy_id: policy_id.clone(),
            })?
            .clone();
        drop(state);

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let escalation = AlertEscalation {
            alert_id: alert.id.clone(),
            level: 0,
            policy_id: policy_id.clone(),
            next_escalation_at: current_time + self.config.initial_grace_period,
            history: Vec::new(),
            last_notification_results: Vec::new(),
            status: EscalationStatus::Active,
            created_at: current_time,
            updated_at: current_time,
        };

        let mut state = self.state.write().await;
        state
            .escalations
            .insert(alert.id.clone(), escalation.clone());

        info!(
            "Started escalation for alert {} with policy {}",
            alert.id, policy_id
        );

        // Send initial notification if configured
        if self.config.enable_notifications {
            self.send_initial_notification(alert, &escalation, &policy)
                .await?;
        }

        Ok(())
    }

    /// Stop escalation for an alert (when acknowledged or resolved)
    pub async fn stop_escalation(&self, alert_id: &str, reason: EscalationEventType) -> Result<()> {
        let mut state = self.state.write().await;

        if let Some(escalation) = state.escalations.get_mut(alert_id) {
            escalation.status = EscalationStatus::Completed;
            escalation.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let event = EscalationEvent {
                timestamp: escalation.updated_at,
                level: escalation.level,
                event_type: reason.clone(),
                description: format!("Escalation stopped: {:?}", reason),
                channels: Vec::new(),
                delivery_results: Vec::new(),
            };
            escalation.history.push(event);

            info!("Stopped escalation for alert {}: {:?}", alert_id, reason);
        }

        Ok(())
    }

    /// Process escalations (called periodically)
    pub async fn process_escalations(&self, active_alerts: &[Alert]) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut state = self.state.write().await;
        let alert_map: HashMap<String, &Alert> = active_alerts
            .iter()
            .map(|alert| (alert.id.clone(), alert))
            .collect();

        // Collect alerts that need escalation to avoid borrowing conflicts
        let mut alerts_to_escalate = Vec::new();

        for (alert_id, escalation) in state.escalations.iter_mut() {
            if escalation.status != EscalationStatus::Active {
                continue;
            }

            // Check if alert still exists and is active
            let alert = match alert_map.get(alert_id) {
                Some(alert) if alert.status == AlertStatus::Active => alert,
                _ => {
                    // Alert is no longer active, stop escalation
                    escalation.status = EscalationStatus::Completed;
                    continue;
                }
            };

            // Check if it's time to escalate
            if current_time >= escalation.next_escalation_at {
                alerts_to_escalate.push((alert_id.clone(), (*alert).clone()));
            }
        }

        // Clone policies to avoid borrowing conflicts
        let policies = state.policies.clone();

        // Process escalations
        for (alert_id, alert) in alerts_to_escalate {
            if let Some(escalation) = state.escalations.get_mut(&alert_id) {
                if let Err(e) = self.escalate_alert(&alert, escalation, &policies).await {
                    error!("Failed to escalate alert {}: {}", alert_id, e);
                    escalation.status = EscalationStatus::Failed;
                }
            }
        }

        // Clean up completed escalations older than 24 hours
        let one_day_ago = current_time - 86400;
        state.escalations.retain(|_, escalation| {
            escalation.status == EscalationStatus::Active || escalation.updated_at > one_day_ago
        });

        Ok(())
    }

    /// Escalate an alert to the next level
    async fn escalate_alert(
        &self,
        alert: &Alert,
        escalation: &mut AlertEscalation,
        policies: &HashMap<String, EscalationPolicy>,
    ) -> EscalationResult<()> {
        let policy =
            policies
                .get(&escalation.policy_id)
                .ok_or_else(|| EscalationError::PolicyNotFound {
                    policy_id: escalation.policy_id.clone(),
                })?;

        // Check if we've reached the maximum escalation level
        if escalation.level >= self.config.max_escalation_levels
            || escalation.level >= policy.steps.len() as u32
        {
            escalation.status = EscalationStatus::MaxLevelReached;
            return Err(EscalationError::MaxLevelReached {
                alert_id: alert.id.clone(),
            });
        }

        escalation.level += 1;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get the current escalation step
        let step_index = (escalation.level as usize).saturating_sub(1);
        let step =
            policy
                .steps
                .get(step_index)
                .ok_or_else(|| EscalationError::InvalidConfiguration {
                    message: format!("Escalation step {} not found in policy", escalation.level),
                })?;

        // Send escalation notifications
        let delivery_results = if self.config.enable_notifications {
            self.send_escalation_notification(alert, escalation, step)
                .await?
        } else {
            Vec::new()
        };

        // Calculate next escalation time
        escalation.next_escalation_at = current_time + (step.timeout_minutes as u64 * 60);
        escalation.updated_at = current_time;
        escalation.last_notification_results = delivery_results.clone();

        // Record escalation event
        let event = EscalationEvent {
            timestamp: current_time,
            level: escalation.level,
            event_type: EscalationEventType::Escalated,
            description: format!("Escalated to level {}", escalation.level),
            channels: step.notification_channels.clone(),
            delivery_results,
        };
        escalation.history.push(event);

        info!(
            "Escalated alert {} to level {} using policy {}",
            alert.id, escalation.level, escalation.policy_id
        );

        Ok(())
    }

    /// Send initial notification for a new alert
    async fn send_initial_notification(
        &self,
        alert: &Alert,
        escalation: &AlertEscalation,
        policy: &EscalationPolicy,
    ) -> EscalationResult<()> {
        if let Some(first_step) = policy.steps.first() {
            let delivery_results = self
                .send_escalation_notification(alert, escalation, first_step)
                .await?;

            let mut state = self.state.write().await;
            if let Some(esc) = state.escalations.get_mut(&alert.id) {
                let event = EscalationEvent {
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    level: 0,
                    event_type: EscalationEventType::InitialNotification,
                    description: "Initial alert notification sent".to_string(),
                    channels: first_step.notification_channels.clone(),
                    delivery_results,
                };
                esc.history.push(event);
            }
        }

        Ok(())
    }

    /// Send escalation notification
    async fn send_escalation_notification(
        &self,
        alert: &Alert,
        escalation: &AlertEscalation,
        step: &EscalationStep,
    ) -> EscalationResult<Vec<DeliveryResult>> {
        let notification_manager = self.notification_manager.as_ref().ok_or_else(|| {
            EscalationError::NotificationFailed {
                message: "Notification manager not configured".to_string(),
            }
        })?;

        // Create notification message
        let message = self.create_notification_message(alert, escalation)?;

        // Send to all channels in the escalation step
        let delivery_results = notification_manager
            .send_notification(&message, &step.notification_channels)
            .await;

        // Check for any delivery failures
        let failed_deliveries: Vec<_> = delivery_results
            .iter()
            .filter(|result| !result.success)
            .collect();

        if !failed_deliveries.is_empty() {
            warn!(
                "Some escalation notifications failed for alert {}: {} failures",
                alert.id,
                failed_deliveries.len()
            );
        }

        Ok(delivery_results)
    }

    /// Create notification message for an alert
    fn create_notification_message(
        &self,
        alert: &Alert,
        escalation: &AlertEscalation,
    ) -> EscalationResult<NotificationMessage> {
        let template_key = if escalation.level == 0 {
            "initial"
        } else {
            "escalation"
        };

        let default_template = "Alert: {{alert.title}}".to_string();
        let template = self
            .config
            .message_templates
            .get(template_key)
            .unwrap_or(&default_template);

        // Simple template rendering (in a real implementation, use a proper template engine)
        let title = template
            .replace("{{alert.title}}", &alert.title)
            .replace("{{alert.description}}", &alert.description)
            .replace("{{escalation.level}}", &escalation.level.to_string());

        let body = format!(
            "Alert Details:\n\
             ID: {}\n\
             Severity: {}\n\
             Rule: {}\n\
             Status: {:?}\n\
             Escalation Level: {}\n\
             Description: {}",
            alert.id,
            alert.severity.to_string(),
            alert.rule_id,
            alert.status,
            escalation.level,
            alert.description
        );

        let priority = match alert.severity {
            AlertSeverity::Critical => MessagePriority::Critical,
            AlertSeverity::High => MessagePriority::High,
            AlertSeverity::Medium => MessagePriority::Normal,
            AlertSeverity::Low | AlertSeverity::Info => MessagePriority::Low,
        };

        let mut metadata = HashMap::new();
        metadata.insert("alert_id".to_string(), alert.id.clone());
        metadata.insert("rule_id".to_string(), alert.rule_id.clone());
        metadata.insert("escalation_level".to_string(), escalation.level.to_string());
        metadata.insert("policy_id".to_string(), escalation.policy_id.clone());

        Ok(NotificationMessage {
            title,
            body,
            priority,
            metadata,
            alert: Some(alert.clone()),
        })
    }

    /// Get escalation status for an alert
    pub async fn get_escalation_status(&self, alert_id: &str) -> Option<AlertEscalation> {
        let state = self.state.read().await;
        state.escalations.get(alert_id).cloned()
    }

    /// Get all active escalations
    pub async fn get_active_escalations(&self) -> Vec<AlertEscalation> {
        let state = self.state.read().await;
        state
            .escalations
            .values()
            .filter(|esc| esc.status == EscalationStatus::Active)
            .cloned()
            .collect()
    }

    /// Get escalation statistics
    pub async fn get_escalation_stats(&self) -> EscalationStats {
        let state = self.state.read().await;

        let total_escalations = state.escalations.len();
        let active_escalations = state
            .escalations
            .values()
            .filter(|esc| esc.status == EscalationStatus::Active)
            .count();

        let completed_escalations = state
            .escalations
            .values()
            .filter(|esc| esc.status == EscalationStatus::Completed)
            .count();

        let failed_escalations = state
            .escalations
            .values()
            .filter(|esc| esc.status == EscalationStatus::Failed)
            .count();

        let max_level_reached = state
            .escalations
            .values()
            .filter(|esc| esc.status == EscalationStatus::MaxLevelReached)
            .count();

        let avg_escalation_level = if active_escalations > 0 {
            let total_levels: u32 = state
                .escalations
                .values()
                .filter(|esc| esc.status == EscalationStatus::Active)
                .map(|esc| esc.level)
                .sum();
            total_levels as f64 / active_escalations as f64
        } else {
            0.0
        };

        EscalationStats {
            total_escalations,
            active_escalations,
            completed_escalations,
            failed_escalations,
            max_level_reached,
            avg_escalation_level,
            policies_count: state.policies.len(),
            policy_assignments_count: state.policy_assignments.len(),
        }
    }

    /// Manually escalate an alert
    pub async fn manual_escalate(&self, alert_id: &str) -> EscalationResult<()> {
        let mut state = self.state.write().await;

        if let Some(escalation) = state.escalations.get_mut(alert_id) {
            if escalation.status != EscalationStatus::Active {
                return Err(EscalationError::InvalidConfiguration {
                    message: "Cannot escalate inactive escalation".to_string(),
                });
            }

            // Force immediate escalation
            escalation.next_escalation_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let event = EscalationEvent {
                timestamp: escalation.next_escalation_at,
                level: escalation.level,
                event_type: EscalationEventType::ManualEscalation,
                description: "Manual escalation triggered".to_string(),
                channels: Vec::new(),
                delivery_results: Vec::new(),
            };
            escalation.history.push(event);

            info!("Manual escalation triggered for alert {}", alert_id);
        }

        Ok(())
    }

    /// Start background escalation processing
    pub fn start_background_processing(&self) {
        let engine = Arc::new(self.clone());
        let interval = Duration::from_secs(self.config.evaluation_interval);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            info!(
                "Starting background escalation processing with {}s interval",
                engine.config.evaluation_interval
            );

            loop {
                interval_timer.tick().await;

                // In a real implementation, this would get active alerts from the alerting manager
                let active_alerts = Vec::new(); // Placeholder

                if let Err(e) = engine.process_escalations(&active_alerts).await {
                    error!("Failed to process escalations: {}", e);
                }
            }
        });
    }
}

impl Clone for EscalationEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
            notification_manager: self.notification_manager.clone(),
        }
    }
}

/// Escalation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStats {
    /// Total number of escalations
    pub total_escalations: usize,
    /// Number of active escalations
    pub active_escalations: usize,
    /// Number of completed escalations
    pub completed_escalations: usize,
    /// Number of failed escalations
    pub failed_escalations: usize,
    /// Number of escalations that reached max level
    pub max_level_reached: usize,
    /// Average escalation level for active escalations
    pub avg_escalation_level: f64,
    /// Number of escalation policies
    pub policies_count: usize,
    /// Number of policy assignments
    pub policy_assignments_count: usize,
}

/// Create default escalation policies
pub fn create_default_escalation_policies() -> Vec<EscalationPolicy> {
    vec![
        EscalationPolicy {
            id: "default".to_string(),
            name: "Default Escalation Policy".to_string(),
            steps: vec![
                EscalationStep {
                    step: 0,
                    timeout_minutes: 15,
                    notification_channels: vec!["email".to_string()],
                    requires_acknowledgment: false,
                },
                EscalationStep {
                    step: 1,
                    timeout_minutes: 30,
                    notification_channels: vec!["email".to_string(), "slack".to_string()],
                    requires_acknowledgment: true,
                },
                EscalationStep {
                    step: 2,
                    timeout_minutes: 60,
                    notification_channels: vec!["pagerduty".to_string()],
                    requires_acknowledgment: true,
                },
            ],
            default_timeout_minutes: 15,
        },
        EscalationPolicy {
            id: "critical".to_string(),
            name: "Critical Alert Escalation".to_string(),
            steps: vec![
                EscalationStep {
                    step: 0,
                    timeout_minutes: 5,
                    notification_channels: vec!["email".to_string(), "slack".to_string()],
                    requires_acknowledgment: false,
                },
                EscalationStep {
                    step: 1,
                    timeout_minutes: 10,
                    notification_channels: vec!["pagerduty".to_string()],
                    requires_acknowledgment: true,
                },
                EscalationStep {
                    step: 2,
                    timeout_minutes: 15,
                    notification_channels: vec!["pagerduty".to_string(), "webhook".to_string()],
                    requires_acknowledgment: true,
                },
            ],
            default_timeout_minutes: 5,
        },
        EscalationPolicy {
            id: "low_priority".to_string(),
            name: "Low Priority Escalation".to_string(),
            steps: vec![
                EscalationStep {
                    step: 0,
                    timeout_minutes: 60,
                    notification_channels: vec!["email".to_string()],
                    requires_acknowledgment: false,
                },
                EscalationStep {
                    step: 1,
                    timeout_minutes: 120,
                    notification_channels: vec!["email".to_string()],
                    requires_acknowledgment: false,
                },
            ],
            default_timeout_minutes: 60,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerting::{AlertRule, AlertType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_escalation_engine_creation() {
        let config = EscalationConfig::default();
        let engine = EscalationEngine::new(config);

        let policies = create_default_escalation_policies();
        for policy in policies {
            engine.add_policy(policy).await.unwrap();
        }

        let stats = engine.get_escalation_stats().await;
        assert_eq!(stats.policies_count, 3);
    }

    #[tokio::test]
    async fn test_alert_escalation_start() {
        let config = EscalationConfig::default();
        let engine = EscalationEngine::new(config);

        let default_policy = create_default_escalation_policies()
            .into_iter()
            .find(|p| p.id == "default")
            .unwrap();
        engine.add_policy(default_policy).await.unwrap();

        let alert = Alert {
            id: "test-alert".to_string(),
            rule_id: "test-rule".to_string(),
            title: "Test Alert".to_string(),
            description: "Test alert description".to_string(),
            severity: AlertSeverity::High,
            status: AlertStatus::Active,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            current_value: Some(85.0),
            threshold_value: Some(80.0),
            context: HashMap::new(),
            escalation_level: 0,
            last_escalation_at: None,
        };

        engine.start_escalation(&alert).await.unwrap();

        let escalation = engine.get_escalation_status(&alert.id).await;
        assert!(escalation.is_some());
        assert_eq!(escalation.unwrap().status, EscalationStatus::Active);
    }

    #[test]
    fn test_default_escalation_policies() {
        let policies = create_default_escalation_policies();
        assert_eq!(policies.len(), 3);

        let default_policy = policies.iter().find(|p| p.id == "default").unwrap();
        assert_eq!(default_policy.steps.len(), 3);

        let critical_policy = policies.iter().find(|p| p.id == "critical").unwrap();
        assert_eq!(critical_policy.steps.len(), 3);
        assert_eq!(critical_policy.steps[0].timeout_minutes, 5);
    }
}
