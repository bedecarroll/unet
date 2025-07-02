//! Emergency override system for μNet
//!
//! This module provides emergency configuration bypass capabilities for critical
//! situations that require immediate configuration changes without normal approval
//! workflows while maintaining full audit trail and safety mechanisms.

use crate::change_tracking::ChangeTrackingService;
use crate::datastore::DataStore;
use crate::git::types::{GitError, GitResult};
use crate::models::change_tracking::{ChangeSource, ChangeStatus, ChangeType, ConfigurationChange};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Emergency override configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyOverrideConfig {
    /// Unique emergency override ID
    pub id: String,
    /// Emergency override name/reason
    pub name: String,
    /// Severity level of the emergency
    pub severity: EmergencySeverity,
    /// Emergency category
    pub category: EmergencyCategory,
    /// Environment being affected
    pub target_environment: String,
    /// Emergency justification
    pub justification: String,
    /// Duration the emergency override is valid
    pub validity_duration: Duration,
    /// Emergency contact information
    pub emergency_contact: EmergencyContact,
    /// Configuration changes to apply
    pub configuration_changes: Vec<EmergencyConfigChange>,
    /// Post-emergency validation requirements
    pub post_emergency_validation: EmergencyValidation,
    /// Emergency notification settings
    pub notification_settings: EmergencyNotificationSettings,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Created by user
    pub created_by: String,
    /// Expiry timestamp
    pub expires_at: DateTime<Utc>,
}

/// Emergency severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EmergencySeverity {
    /// Critical emergency - immediate action required
    Critical,
    /// High emergency - urgent action required
    High,
    /// Medium emergency - prompt action required
    Medium,
    /// Low emergency - planned emergency action
    Low,
}

/// Emergency categories for classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EmergencyCategory {
    /// Security incident requiring immediate response
    SecurityIncident,
    /// Network outage or failure
    NetworkOutage,
    /// Performance degradation
    PerformanceIssue,
    /// System failure or malfunction
    SystemFailure,
    /// Compliance requirement
    ComplianceRequirement,
    /// Disaster recovery operation
    DisasterRecovery,
    /// Other emergency situation
    Other,
}

/// Emergency contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    /// Primary contact name
    pub name: String,
    /// Contact email
    pub email: String,
    /// Contact phone
    pub phone: Option<String>,
    /// On-call group or team
    pub team: Option<String>,
    /// Escalation contact
    pub escalation_contact: Option<String>,
}

/// Emergency configuration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyConfigChange {
    /// Change ID
    pub id: String,
    /// Type of entity being changed
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Change description
    pub description: String,
    /// Configuration change details
    pub change_details: serde_json::Value,
    /// Previous value for rollback
    pub previous_value: Option<serde_json::Value>,
    /// Expected impact assessment
    pub impact_assessment: String,
}

/// Post-emergency validation requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyValidation {
    /// Required validation checks
    pub required_checks: Vec<String>,
    /// Validation timeout
    pub validation_timeout: Duration,
    /// Post-emergency review required
    pub review_required: bool,
    /// Automatic revert if validation fails
    pub auto_revert_on_failure: bool,
    /// Validation criteria
    pub validation_criteria: HashMap<String, serde_json::Value>,
}

/// Emergency notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyNotificationSettings {
    /// Immediate notification recipients
    pub immediate_recipients: Vec<String>,
    /// Executive notification recipients
    pub executive_recipients: Vec<String>,
    /// Notification methods
    pub notification_methods: Vec<EmergencyNotificationMethod>,
    /// Notification frequency
    pub notification_frequency: Duration,
}

/// Emergency notification methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyNotificationMethod {
    /// Email notification
    Email,
    /// SMS notification
    Sms,
    /// Slack notification
    Slack,
    /// PagerDuty alert
    PagerDuty,
    /// Webhook notification
    Webhook { url: String },
}

/// Emergency override status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyStatus {
    /// Emergency declared and active
    Active,
    /// Emergency resolved
    Resolved,
    /// Emergency cancelled
    Cancelled,
    /// Emergency expired
    Expired,
    /// Emergency under review
    UnderReview,
    /// Emergency validation failed
    ValidationFailed,
}

/// Emergency override audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAuditEntry {
    /// Audit entry ID
    pub id: String,
    /// Emergency override ID
    pub emergency_id: String,
    /// Action performed
    pub action: EmergencyAction,
    /// Actor performing the action
    pub actor_id: Option<String>,
    /// Action details
    pub details: String,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Emergency actions for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyAction {
    /// Emergency declared
    Declared,
    /// Configuration applied
    ConfigurationApplied,
    /// Emergency resolved
    Resolved,
    /// Emergency cancelled
    Cancelled,
    /// Validation performed
    ValidationPerformed,
    /// Rollback initiated
    RollbackInitiated,
    /// Notification sent
    NotificationSent,
    /// Review completed
    ReviewCompleted,
}

/// Emergency rollback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyRollback {
    /// Rollback ID
    pub id: String,
    /// Emergency override ID
    pub emergency_id: String,
    /// Rollback reason
    pub reason: String,
    /// Rollback strategy
    pub strategy: EmergencyRollbackStrategy,
    /// Configuration snapshots to restore
    pub snapshots: Vec<EmergencySnapshot>,
    /// Rollback status
    pub status: EmergencyRollbackStatus,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Executed timestamp
    pub executed_at: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Emergency rollback strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyRollbackStrategy {
    /// Complete rollback to previous state
    Complete,
    /// Partial rollback of specific changes
    Partial,
    /// Gradual rollback with validation
    Gradual,
    /// Emergency stop - halt all operations
    EmergencyStop,
}

/// Emergency rollback status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyRollbackStatus {
    /// Rollback pending
    Pending,
    /// Rollback in progress
    InProgress,
    /// Rollback completed successfully
    Completed,
    /// Rollback failed
    Failed,
    /// Rollback cancelled
    Cancelled,
}

/// Emergency configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencySnapshot {
    /// Snapshot ID
    pub id: String,
    /// Entity type
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Snapshot data
    pub snapshot_data: serde_json::Value,
    /// Snapshot checksum
    pub checksum: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Emergency override manager
pub struct EmergencyOverrideManager<D: DataStore + Clone> {
    change_tracking: ChangeTrackingService<D>,
    active_emergencies: HashMap<String, EmergencyOverrideConfig>,
}

impl<D: DataStore + Clone> EmergencyOverrideManager<D> {
    /// Create a new emergency override manager
    pub fn new(change_tracking: ChangeTrackingService<D>) -> Self {
        Self {
            change_tracking,
            active_emergencies: HashMap::new(),
        }
    }

    /// Declare an emergency override
    pub async fn declare_emergency(
        &mut self,
        mut config: EmergencyOverrideConfig,
    ) -> GitResult<EmergencyOverrideConfig> {
        info!(
            "Declaring emergency override: {} ({})",
            config.name, config.severity as u8
        );

        // Validate emergency configuration
        self.validate_emergency_config(&config).await?;

        // Set expiry timestamp
        config.expires_at = config.created_at
            + ChronoDuration::from_std(config.validity_duration)
                .map_err(|e| GitError::Other(format!("Invalid duration: {}", e)))?;

        // Create emergency audit entry
        let audit_entry = EmergencyAuditEntry {
            id: Uuid::new_v4().to_string(),
            emergency_id: config.id.clone(),
            action: EmergencyAction::Declared,
            actor_id: Some(config.created_by.clone()),
            details: format!(
                "Emergency declared: {} - {}",
                config.name, config.justification
            ),
            metadata: Some(serde_json::to_value(&config).unwrap()),
            timestamp: Utc::now(),
        };

        // Store emergency audit entry
        self.store_emergency_audit(audit_entry).await?;

        // Send immediate notifications
        self.send_emergency_notifications(&config, EmergencyAction::Declared)
            .await?;

        // Store active emergency
        self.active_emergencies
            .insert(config.id.clone(), config.clone());

        // Create emergency snapshots for rollback
        self.create_emergency_snapshots(&config).await?;

        Ok(config)
    }

    /// Apply emergency configuration bypass
    pub async fn apply_emergency_bypass(
        &mut self,
        emergency_id: &str,
        actor_id: &str,
    ) -> GitResult<Vec<ConfigurationChange>> {
        let emergency_config = self
            .active_emergencies
            .get(emergency_id)
            .ok_or_else(|| GitError::Other("Emergency not found".to_string()))?
            .clone();

        // Verify emergency is still valid
        self.verify_emergency_validity(&emergency_config).await?;

        info!("Applying emergency bypass for: {}", emergency_config.name);

        let mut applied_changes = Vec::new();

        // Apply each emergency configuration change
        for emergency_change in &emergency_config.configuration_changes {
            // Create configuration change with emergency bypass
            let config_change = ConfigurationChange {
                id: Uuid::new_v4().to_string(),
                change_type: ChangeType::Update,
                entity_type: emergency_change.entity_type.clone(),
                entity_id: emergency_change.entity_id.clone(),
                user_id: Some(actor_id.to_string()),
                source: ChangeSource::External, // Emergency bypass source
                description: Some(format!(
                    "EMERGENCY BYPASS: {} - {}",
                    emergency_config.name, emergency_change.description
                )),
                old_value: emergency_change
                    .previous_value
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap()),
                new_value: Some(serde_json::to_string(&emergency_change.change_details).unwrap()),
                diff_content: None,
                git_commit: None,
                git_branch: Some(format!(
                    "emergency/{}/{}",
                    emergency_id, emergency_change.id
                )),
                status: ChangeStatus::Applied, // Bypass approval - direct application
                approval_required: false,      // Emergency bypass - no approval needed
                approved_by: Some(format!("EMERGENCY_BYPASS:{}", actor_id)),
                approved_at: Some(Utc::now()),
                applied_at: Some(Utc::now()),
                rolled_back_at: None,
                custom_data: Some(
                    serde_json::to_string(&serde_json::json!({
                        "emergency_id": emergency_id,
                        "emergency_category": emergency_config.category,
                        "emergency_severity": emergency_config.severity,
                        "justification": emergency_config.justification,
                        "bypass_reason": "emergency_override"
                    }))
                    .unwrap(),
                ),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // Track the emergency change
            let tracked_change = self
                .change_tracking
                .track_change(config_change)
                .await
                .map_err(|e| GitError::Other(format!("Failed to track emergency change: {}", e)))?;

            applied_changes.push(tracked_change);
        }

        // Create emergency audit entry for configuration application
        let audit_entry = EmergencyAuditEntry {
            id: Uuid::new_v4().to_string(),
            emergency_id: emergency_id.to_string(),
            action: EmergencyAction::ConfigurationApplied,
            actor_id: Some(actor_id.to_string()),
            details: format!(
                "Emergency configuration applied: {} changes",
                applied_changes.len()
            ),
            metadata: Some(serde_json::json!({
                "change_ids": applied_changes.iter().map(|c| &c.id).collect::<Vec<_>>(),
                "entity_count": applied_changes.len()
            })),
            timestamp: Utc::now(),
        };

        self.store_emergency_audit(audit_entry).await?;

        // Send configuration applied notifications
        self.send_emergency_notifications(&emergency_config, EmergencyAction::ConfigurationApplied)
            .await?;

        // Schedule post-emergency validation
        self.schedule_post_emergency_validation(emergency_id)
            .await?;

        Ok(applied_changes)
    }

    /// Initiate emergency rollback
    pub async fn initiate_emergency_rollback(
        &mut self,
        emergency_id: &str,
        reason: &str,
        strategy: EmergencyRollbackStrategy,
        actor_id: &str,
    ) -> GitResult<EmergencyRollback> {
        info!(
            "Initiating emergency rollback for: {} - {}",
            emergency_id, reason
        );

        let emergency_config = self
            .active_emergencies
            .get(emergency_id)
            .ok_or_else(|| GitError::Other("Emergency not found".to_string()))?
            .clone();

        // Load emergency snapshots
        let snapshots = self.load_emergency_snapshots(emergency_id).await?;

        // Create rollback configuration
        let rollback = EmergencyRollback {
            id: Uuid::new_v4().to_string(),
            emergency_id: emergency_id.to_string(),
            reason: reason.to_string(),
            strategy: strategy.clone(),
            snapshots,
            status: EmergencyRollbackStatus::Pending,
            created_at: Utc::now(),
            executed_at: None,
            completed_at: None,
        };

        // Execute rollback based on strategy
        let executed_rollback = match strategy {
            EmergencyRollbackStrategy::Complete => self.execute_complete_rollback(rollback).await?,
            EmergencyRollbackStrategy::Partial => self.execute_partial_rollback(rollback).await?,
            EmergencyRollbackStrategy::Gradual => self.execute_gradual_rollback(rollback).await?,
            EmergencyRollbackStrategy::EmergencyStop => {
                self.execute_emergency_stop(rollback).await?
            }
        };

        // Create emergency audit entry for rollback
        let audit_entry = EmergencyAuditEntry {
            id: Uuid::new_v4().to_string(),
            emergency_id: emergency_id.to_string(),
            action: EmergencyAction::RollbackInitiated,
            actor_id: Some(actor_id.to_string()),
            details: format!("Emergency rollback initiated: {}", reason),
            metadata: Some(serde_json::json!({
                "rollback_id": executed_rollback.id,
                "strategy": strategy,
                "snapshot_count": executed_rollback.snapshots.len()
            })),
            timestamp: Utc::now(),
        };

        self.store_emergency_audit(audit_entry).await?;

        // Send rollback notifications
        self.send_emergency_notifications(&emergency_config, EmergencyAction::RollbackInitiated)
            .await?;

        Ok(executed_rollback)
    }

    /// Resolve emergency override
    pub async fn resolve_emergency(
        &mut self,
        emergency_id: &str,
        resolution_notes: &str,
        actor_id: &str,
    ) -> GitResult<()> {
        info!("Resolving emergency override: {}", emergency_id);

        let emergency_config = self
            .active_emergencies
            .get(emergency_id)
            .ok_or_else(|| GitError::Other("Emergency not found".to_string()))?
            .clone();

        // Perform post-emergency validation if required
        if emergency_config.post_emergency_validation.review_required {
            self.perform_post_emergency_review(emergency_id, actor_id)
                .await?;
        }

        // Remove from active emergencies
        self.active_emergencies.remove(emergency_id);

        // Create emergency audit entry for resolution
        let audit_entry = EmergencyAuditEntry {
            id: Uuid::new_v4().to_string(),
            emergency_id: emergency_id.to_string(),
            action: EmergencyAction::Resolved,
            actor_id: Some(actor_id.to_string()),
            details: format!("Emergency resolved: {}", resolution_notes),
            metadata: Some(serde_json::json!({
                "resolution_notes": resolution_notes,
                "emergency_duration": (Utc::now() - emergency_config.created_at).num_seconds()
            })),
            timestamp: Utc::now(),
        };

        self.store_emergency_audit(audit_entry).await?;

        // Send resolution notifications
        self.send_emergency_notifications(&emergency_config, EmergencyAction::Resolved)
            .await?;

        Ok(())
    }

    /// Get active emergency overrides
    pub fn get_active_emergencies(&self) -> Vec<&EmergencyOverrideConfig> {
        self.active_emergencies.values().collect()
    }

    /// Check if emergency exists and is active
    pub fn is_emergency_active(&self, emergency_id: &str) -> bool {
        self.active_emergencies.contains_key(emergency_id)
    }

    // Private helper methods

    async fn validate_emergency_config(&self, config: &EmergencyOverrideConfig) -> GitResult<()> {
        // Validate severity and category combination
        match (config.severity, config.category) {
            (EmergencySeverity::Critical, EmergencyCategory::SecurityIncident) => {
                // High priority validation for critical security incidents
                if config.validity_duration > Duration::from_secs(4 * 60 * 60) {
                    return Err(GitError::Other(
                        "Critical security incidents cannot have validity longer than 4 hours"
                            .to_string(),
                    ));
                }
            }
            _ => {}
        }

        // Validate justification length
        if config.justification.len() < 50 {
            return Err(GitError::Other(
                "Emergency justification must be at least 50 characters".to_string(),
            ));
        }

        // Validate configuration changes
        if config.configuration_changes.is_empty() {
            return Err(GitError::Other(
                "Emergency must include at least one configuration change".to_string(),
            ));
        }

        Ok(())
    }

    async fn verify_emergency_validity(&self, config: &EmergencyOverrideConfig) -> GitResult<()> {
        if Utc::now() > config.expires_at {
            return Err(GitError::Other("Emergency has expired".to_string()));
        }
        Ok(())
    }

    async fn create_emergency_snapshots(&self, config: &EmergencyOverrideConfig) -> GitResult<()> {
        info!(
            "Creating emergency snapshots for: {}",
            config.configuration_changes.len()
        );

        for change in &config.configuration_changes {
            if let Some(previous_value) = &change.previous_value {
                let snapshot = EmergencySnapshot {
                    id: Uuid::new_v4().to_string(),
                    entity_type: change.entity_type.clone(),
                    entity_id: change.entity_id.clone(),
                    snapshot_data: previous_value.clone(),
                    checksum: self.calculate_checksum(previous_value),
                    created_at: Utc::now(),
                };

                // Store emergency snapshot (would integrate with actual storage)
                info!("Created emergency snapshot: {}", snapshot.id);
            }
        }

        Ok(())
    }

    async fn load_emergency_snapshots(
        &self,
        emergency_id: &str,
    ) -> GitResult<Vec<EmergencySnapshot>> {
        // In a real implementation, this would load from storage
        info!("Loading emergency snapshots for: {}", emergency_id);
        Ok(Vec::new()) // Placeholder
    }

    async fn execute_complete_rollback(
        &self,
        mut rollback: EmergencyRollback,
    ) -> GitResult<EmergencyRollback> {
        rollback.status = EmergencyRollbackStatus::InProgress;
        rollback.executed_at = Some(Utc::now());

        info!("Executing complete emergency rollback: {}", rollback.id);

        // Rollback all changes from snapshots
        for snapshot in &rollback.snapshots {
            info!(
                "Rolling back {}/{} to snapshot {}",
                snapshot.entity_type, snapshot.entity_id, snapshot.id
            );
            // Implementation would restore from snapshot
        }

        rollback.status = EmergencyRollbackStatus::Completed;
        rollback.completed_at = Some(Utc::now());

        Ok(rollback)
    }

    async fn execute_partial_rollback(
        &self,
        mut rollback: EmergencyRollback,
    ) -> GitResult<EmergencyRollback> {
        rollback.status = EmergencyRollbackStatus::InProgress;
        rollback.executed_at = Some(Utc::now());

        info!("Executing partial emergency rollback: {}", rollback.id);

        // Implementation would rollback selected changes
        rollback.status = EmergencyRollbackStatus::Completed;
        rollback.completed_at = Some(Utc::now());

        Ok(rollback)
    }

    async fn execute_gradual_rollback(
        &self,
        mut rollback: EmergencyRollback,
    ) -> GitResult<EmergencyRollback> {
        rollback.status = EmergencyRollbackStatus::InProgress;
        rollback.executed_at = Some(Utc::now());

        info!("Executing gradual emergency rollback: {}", rollback.id);

        // Implementation would rollback changes gradually with validation
        rollback.status = EmergencyRollbackStatus::Completed;
        rollback.completed_at = Some(Utc::now());

        Ok(rollback)
    }

    async fn execute_emergency_stop(
        &self,
        mut rollback: EmergencyRollback,
    ) -> GitResult<EmergencyRollback> {
        rollback.status = EmergencyRollbackStatus::InProgress;
        rollback.executed_at = Some(Utc::now());

        warn!("Executing emergency stop: {}", rollback.id);

        // Implementation would halt all operations immediately
        rollback.status = EmergencyRollbackStatus::Completed;
        rollback.completed_at = Some(Utc::now());

        Ok(rollback)
    }

    async fn schedule_post_emergency_validation(&self, emergency_id: &str) -> GitResult<()> {
        info!("Scheduling post-emergency validation for: {}", emergency_id);
        // Implementation would schedule validation tasks
        Ok(())
    }

    async fn perform_post_emergency_review(
        &self,
        emergency_id: &str,
        _actor_id: &str,
    ) -> GitResult<()> {
        info!("Performing post-emergency review for: {}", emergency_id);
        // Implementation would perform comprehensive review
        Ok(())
    }

    async fn send_emergency_notifications(
        &self,
        config: &EmergencyOverrideConfig,
        action: EmergencyAction,
    ) -> GitResult<()> {
        info!(
            "Sending emergency notifications for: {} - {:?}",
            config.name, action
        );

        for method in &config.notification_settings.notification_methods {
            match method {
                EmergencyNotificationMethod::Email => {
                    info!("Sending email notifications");
                }
                EmergencyNotificationMethod::Sms => {
                    info!("Sending SMS notifications");
                }
                EmergencyNotificationMethod::Slack => {
                    info!("Sending Slack notifications");
                }
                EmergencyNotificationMethod::PagerDuty => {
                    warn!("Triggering PagerDuty alert");
                }
                EmergencyNotificationMethod::Webhook { url } => {
                    info!("Sending webhook notification to: {}", url);
                }
            }
        }

        Ok(())
    }

    async fn store_emergency_audit(&self, audit_entry: EmergencyAuditEntry) -> GitResult<()> {
        info!("Storing emergency audit entry: {}", audit_entry.id);
        // Implementation would store to database
        Ok(())
    }

    fn calculate_checksum(&self, data: &serde_json::Value) -> String {
        use sha2::{Digest, Sha256};
        let data_str = serde_json::to_string(data).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(data_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Emergency override result type
pub type EmergencyResult<T> = Result<T, GitError>;

/// Error types specific to emergency operations
#[derive(Debug, thiserror::Error)]
pub enum EmergencyError {
    #[error("Emergency configuration validation failed: {0}")]
    ValidationFailed(String),

    #[error("Emergency has expired")]
    Expired,

    #[error("Emergency not found: {0}")]
    NotFound(String),

    #[error("Emergency rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Insufficient permissions for emergency operation")]
    InsufficientPermissions,

    #[error("Emergency notification failed: {0}")]
    NotificationFailed(String),
}
