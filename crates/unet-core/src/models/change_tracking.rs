//! Change Tracking Models
//!
//! This module provides data models for tracking configuration changes,
//! audit trails, approval workflows, and rollback capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a configuration change in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigurationChange {
    /// Unique identifier for the change
    pub id: String,
    /// Type of change (create, update, delete, bulk_update)
    pub change_type: ChangeType,
    /// Type of entity being changed (node, template, policy, etc.)
    pub entity_type: String,
    /// ID of the specific entity being changed
    pub entity_id: String,
    /// User who initiated the change
    pub user_id: Option<String>,
    /// Source of the change (api, cli, git_sync, etc.)
    pub source: ChangeSource,
    /// Human-readable description of the change
    pub description: Option<String>,
    /// Previous value (JSON serialized)
    pub old_value: Option<String>,
    /// New value (JSON serialized)
    pub new_value: Option<String>,
    /// Unified diff content showing the changes
    pub diff_content: Option<String>,
    /// Git commit hash if change came from Git
    pub git_commit: Option<String>,
    /// Git branch if change came from Git
    pub git_branch: Option<String>,
    /// Current status of the change
    pub status: ChangeStatus,
    /// Whether this change requires approval
    pub approval_required: bool,
    /// User who approved the change
    pub approved_by: Option<String>,
    /// When the change was approved
    pub approved_at: Option<DateTime<Utc>>,
    /// When the change was applied
    pub applied_at: Option<DateTime<Utc>>,
    /// When the change was rolled back
    pub rolled_back_at: Option<DateTime<Utc>>,
    /// Additional custom data (JSON)
    pub custom_data: Option<String>,
    /// When the change record was created
    pub created_at: DateTime<Utc>,
    /// When the change record was last updated
    pub updated_at: DateTime<Utc>,
}

/// Types of configuration changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    /// Creating a new entity
    Create,
    /// Updating an existing entity
    Update,
    /// Deleting an entity
    Delete,
    /// Bulk update operation
    BulkUpdate,
    /// Template application
    TemplateApply,
    /// Policy application
    PolicyApply,
    /// Configuration sync from Git
    GitSync,
}

/// Source of the change
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ChangeSource {
    /// Change made through REST API
    Api,
    /// Change made through CLI
    Cli,
    /// Change from Git synchronization
    GitSync,
    /// Change from automated process
    Automation,
    /// Change from SNMP polling update
    SnmpSync,
    /// Change from external system
    External,
}

/// Status of a configuration change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeStatus {
    /// Change is pending approval or application
    Pending,
    /// Change is approved and ready to apply
    Approved,
    /// Change has been applied successfully
    Applied,
    /// Change was rejected
    Rejected,
    /// Change failed to apply
    Failed,
    /// Change was rolled back
    RolledBack,
    /// Change is in progress
    InProgress,
}

/// Audit log entry for a configuration change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeAuditLog {
    /// Unique identifier for the audit log entry
    pub id: String,
    /// ID of the associated configuration change
    pub change_id: String,
    /// Action that was performed
    pub action: AuditAction,
    /// ID of the actor who performed the action
    pub actor_id: Option<String>,
    /// Type of actor (user, system, api_key, etc.)
    pub actor_type: ActorType,
    /// Detailed information about the action
    pub details: Option<String>,
    /// Additional metadata (JSON)
    pub metadata: Option<String>,
    /// IP address of the actor
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// When the action occurred
    pub timestamp: DateTime<Utc>,
}

/// Actions that can be audited
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditAction {
    /// Change was created
    Created,
    /// Change was submitted for approval
    Submitted,
    /// Change was approved
    Approved,
    /// Change was rejected
    Rejected,
    /// Change was applied
    Applied,
    /// Change was rolled back
    RolledBack,
    /// Change was cancelled
    Cancelled,
    /// Change was modified
    Modified,
    /// Change was viewed
    Viewed,
}

/// Types of actors in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActorType {
    /// Human user
    User,
    /// System process
    System,
    /// API key authentication
    ApiKey,
    /// Service account
    ServiceAccount,
    /// External system
    External,
}

/// Approval workflow for a configuration change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeApprovalWorkflow {
    /// Unique identifier for the workflow
    pub id: String,
    /// ID of the associated configuration change
    pub change_id: String,
    /// Type of approval workflow
    pub workflow_type: WorkflowType,
    /// Current status of the workflow
    pub status: WorkflowStatus,
    /// List of required approvers (JSON array)
    pub required_approvers: Option<String>,
    /// List of current approvers (JSON array)
    pub current_approvers: Option<String>,
    /// Approval rules configuration (JSON)
    pub rules: Option<String>,
    /// Comments from approvers
    pub comments: Option<String>,
    /// When the workflow expires
    pub expires_at: Option<DateTime<Utc>>,
    /// When the workflow was created
    pub created_at: DateTime<Utc>,
    /// When the workflow was last updated
    pub updated_at: DateTime<Utc>,
}

/// Types of approval workflows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowType {
    /// Single approver required
    SingleApprover,
    /// Multiple approvers required
    MultipleApprovers,
    /// Consensus required
    Consensus,
    /// Emergency bypass
    Emergency,
    /// Automatic approval
    Automatic,
}

/// Status of an approval workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    /// Workflow is pending approvals
    Pending,
    /// Workflow has been approved
    Approved,
    /// Workflow has been rejected
    Rejected,
    /// Workflow has expired
    Expired,
    /// Workflow has been cancelled
    Cancelled,
}

/// Snapshot for rollback capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeRollbackSnapshot {
    /// Unique identifier for the snapshot
    pub id: String,
    /// ID of the associated configuration change
    pub change_id: String,
    /// Type of entity in the snapshot
    pub entity_type: String,
    /// ID of the entity in the snapshot
    pub entity_id: String,
    /// Type of snapshot
    pub snapshot_type: SnapshotType,
    /// Complete state snapshot (JSON)
    pub state_snapshot: String,
    /// Checksum for integrity verification
    pub checksum: String,
    /// Dependencies that need to be considered for rollback (JSON array)
    pub dependencies: Option<String>,
    /// Additional metadata (JSON)
    pub metadata: Option<String>,
    /// When the snapshot was created
    pub created_at: DateTime<Utc>,
}

/// Types of rollback snapshots
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapshotType {
    /// Snapshot taken before the change
    PreChange,
    /// Snapshot taken after the change
    PostChange,
    /// Full system state snapshot
    FullState,
    /// Incremental change snapshot
    Incremental,
}

/// Builder for ConfigurationChange
#[derive(Debug)]
pub struct ConfigurationChangeBuilder {
    change: ConfigurationChange,
}

impl ConfigurationChangeBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            change: ConfigurationChange {
                id: Uuid::new_v4().to_string(),
                change_type: ChangeType::Update,
                entity_type: String::new(),
                entity_id: String::new(),
                user_id: None,
                source: ChangeSource::Api,
                description: None,
                old_value: None,
                new_value: None,
                diff_content: None,
                git_commit: None,
                git_branch: None,
                status: ChangeStatus::Pending,
                approval_required: false,
                approved_by: None,
                approved_at: None,
                applied_at: None,
                rolled_back_at: None,
                custom_data: None,
                created_at: now,
                updated_at: now,
            },
        }
    }

    /// Set the change type
    pub fn change_type(mut self, change_type: ChangeType) -> Self {
        self.change.change_type = change_type;
        self
    }

    /// Set the entity type and ID
    pub fn entity(mut self, entity_type: String, entity_id: String) -> Self {
        self.change.entity_type = entity_type;
        self.change.entity_id = entity_id;
        self
    }

    /// Set the user ID
    pub fn user_id(mut self, user_id: String) -> Self {
        self.change.user_id = Some(user_id);
        self
    }

    /// Set the change source
    pub fn source(mut self, source: ChangeSource) -> Self {
        self.change.source = source;
        self
    }

    /// Set the description
    pub fn description(mut self, description: String) -> Self {
        self.change.description = Some(description);
        self
    }

    /// Set the old and new values
    pub fn values(mut self, old_value: Option<String>, new_value: Option<String>) -> Self {
        self.change.old_value = old_value;
        self.change.new_value = new_value;
        self
    }

    /// Set the diff content
    pub fn diff_content(mut self, diff: String) -> Self {
        self.change.diff_content = Some(diff);
        self
    }

    /// Set Git information
    pub fn git_info(mut self, commit: Option<String>, branch: Option<String>) -> Self {
        self.change.git_commit = commit;
        self.change.git_branch = branch;
        self
    }

    /// Set approval requirement
    pub fn approval_required(mut self, required: bool) -> Self {
        self.change.approval_required = required;
        self
    }

    /// Build the ConfigurationChange
    pub fn build(self) -> ConfigurationChange {
        self.change
    }
}

/// Result type for change tracking operations
pub type ChangeTrackingResult<T> = Result<T, ChangeTrackingError>;

/// Comprehensive audit trail report for a specific change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailReport {
    /// ID of the change being audited
    pub change_id: String,
    /// Type of entity that was changed
    pub entity_type: String,
    /// ID of the entity that was changed
    pub entity_id: String,
    /// Total number of audit actions
    pub total_actions: usize,
    /// Complete timeline of audit events
    pub timeline: Vec<ChangeAuditLog>,
    /// History of approval/rejection actions
    pub approval_history: Vec<ChangeAuditLog>,
    /// History of status changes
    pub status_changes: Vec<ChangeAuditLog>,
    /// History of rollback actions
    pub rollback_history: Vec<ChangeAuditLog>,
    /// Human-readable summary of the audit trail
    pub summary: String,
}

/// Change history report with trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeHistoryReport {
    /// Type of entity being analyzed
    pub entity_type: String,
    /// ID of the entity being analyzed
    pub entity_id: String,
    /// Number of days in the analysis period
    pub period_days: u32,
    /// Total number of changes in the period
    pub total_changes: usize,
    /// Count of changes by status
    pub changes_by_status: std::collections::HashMap<String, usize>,
    /// Count of changes by type
    pub changes_by_type: std::collections::HashMap<String, usize>,
    /// Count of changes by source
    pub changes_by_source: std::collections::HashMap<String, usize>,
    /// List of recent changes
    pub recent_changes: Vec<ConfigurationChange>,
    /// Average changes per day
    pub change_frequency: f64,
    /// Most active time period (if detected)
    pub most_active_period: Option<String>,
    /// Percentage of changes that were approved
    pub approval_rate: f64,
    /// Percentage of changes that were rolled back
    pub rollback_rate: f64,
}

/// Parameters for searching configuration changes
#[derive(Debug, Clone, Default)]
pub struct ChangeSearchParams {
    /// Filter by entity type
    pub entity_type: Option<String>,
    /// Filter by entity ID
    pub entity_id: Option<String>,
    /// Filter by change status
    pub status: Option<ChangeStatus>,
    /// Filter by user ID
    pub user_id: Option<String>,
    /// Filter by change source
    pub source: Option<ChangeSource>,
    /// Filter by date range (from)
    pub from_date: Option<DateTime<Utc>>,
    /// Filter by date range (to)
    pub to_date: Option<DateTime<Utc>>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Search in description/details
    pub search_text: Option<String>,
}

/// System-wide audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAuditReport {
    /// Reporting period description
    pub report_period: String,
    /// Total number of changes in the period
    pub total_changes: usize,
    /// Number of unique entities affected
    pub total_entities_affected: usize,
    /// Overall compliance score (0.0 - 1.0)
    pub compliance_score: f64,
    /// List of security-related events
    pub security_events: Vec<SecurityEvent>,
    /// Top change sources by volume
    pub top_change_sources: std::collections::HashMap<String, usize>,
    /// Detected unusual patterns
    pub unusual_patterns: Vec<String>,
    /// System recommendations
    pub recommendations: Vec<String>,
}

/// Security event detected in audit analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Type of security event
    pub event_type: SecurityEventType,
    /// Severity level
    pub severity: EventSeverity,
    /// Description of the event
    pub description: String,
    /// Related change ID (if applicable)
    pub related_change_id: Option<String>,
    /// User associated with the event (if applicable)
    pub user_id: Option<String>,
    /// When the event was detected
    pub detected_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: Option<String>,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Unauthorized access attempt
    UnauthorizedAccess,
    /// Suspicious change pattern
    SuspiciousActivity,
    /// Privilege escalation
    PrivilegeEscalation,
    /// Data integrity violation
    IntegrityViolation,
    /// Policy violation
    PolicyViolation,
    /// Unusual rollback pattern
    UnusualRollback,
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Low impact event
    Low,
    /// Medium impact event
    Medium,
    /// High impact event
    High,
    /// Critical security event
    Critical,
}

/// Errors that can occur in change tracking operations
#[derive(Debug, thiserror::Error)]
pub enum ChangeTrackingError {
    /// Change not found
    #[error("Configuration change not found: {id}")]
    ChangeNotFound { id: String },
    /// Invalid change status transition
    #[error("Invalid status transition from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },
    /// Approval required but not provided
    #[error("Approval required for change {id}")]
    ApprovalRequired { id: String },
    /// Snapshot creation failed
    #[error("Failed to create rollback snapshot: {reason}")]
    SnapshotFailed { reason: String },
    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    /// Database error
    #[error("Database error: {message}")]
    DatabaseError { message: String },
}
