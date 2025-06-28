//! Git integration for μNet
//!
//! This module provides Git repository management functionality for μNet,
//! including repository operations, credential handling, and state tracking.

pub mod branch_management;
pub mod canary;
pub mod change_tracker;
pub mod client;
pub mod config_management;
pub mod conflict_integration;
pub mod conflict_resolution;
pub mod conflict_tools;
pub mod credentials;
pub mod emergency;
pub mod environment;
pub mod notification;
pub mod repository;
pub mod state;
pub mod types;
pub mod validation;

// Re-exports for convenience
pub use branch_management::{
    BranchManager, BranchOperation, BranchProtectionRules, BranchSwitchContext, BranchSwitchResult,
    EnhancedBranchInfo, MergeStrategy,
};
pub use canary::{
    CanaryAutoRollback, CanaryConfig, CanaryCustomCriterion, CanaryDeployment,
    CanaryDeploymentMetrics, CanaryEventType, CanaryHealthCheckConfig, CanaryHealthCheckResult,
    CanaryHealthStatus, CanaryManager, CanaryMonitoringConfig, CanaryNotificationSettings,
    CanaryPerformanceThresholds, CanaryResourceUtilization, CanaryRollbackInfo,
    CanaryRollbackTrigger, CanaryStatus, CanarySuccessCriteria, CanaryTestStatus, CanaryTestType,
    CanaryValidationConfig, CanaryValidationResult,
};
pub use change_tracker::{
    ChangeNotificationEvent, ChangeTrackerConfig, FileChangeTracker, FileType, TrackedFileChange,
};
pub use client::{GitClient, GitClientConfig};
pub use config_management::{
    ConfigChangeType, ConfigDiff, ConfigDiffEntry, ConfigErrorType, ConfigFormat, ConfigLayer,
    ConfigPriority, ConfigValidationError, EnvironmentConfigManager,
};
pub use conflict_integration::{
    ConflictResolutionService, ConflictResolutionSummary, ConflictResolutionWorkflow,
    ConflictServiceConfig, ConflictSessionStatus, ConflictSuggestion, DiffType, WorkflowProgress,
    WorkflowStatus,
};
pub use conflict_resolution::{
    ConflictAnalysis, ConflictInfo, ConflictResolutionConfig, ConflictResolutionResult,
    ConflictResolver, ConflictType, EnvironmentResolutionRecommendation, ManualResolutionInterface,
    ResolutionOption, ResolutionStrategy,
};
pub use conflict_tools::{
    ComplexityLevel, ConflictComplexity, ConflictDiffViewer, ConflictResolutionAssistant,
    ConflictResolutionSession, ConflictSessionStatistics, DiffColorScheme, DiffViewerConfig,
    MergeToolConfig, MergeToolManager, MergeToolResult, ResolutionSuggestion,
};
pub use credentials::{GitCredentialProvider, GitCredentials, MemoryCredentialProvider};
pub use emergency::{
    EmergencyAction, EmergencyAuditEntry, EmergencyCategory, EmergencyConfigChange,
    EmergencyContact, EmergencyError, EmergencyNotificationMethod, EmergencyNotificationSettings,
    EmergencyOverrideConfig, EmergencyOverrideManager, EmergencyResult, EmergencyRollback,
    EmergencyRollbackStatus, EmergencyRollbackStrategy, EmergencySeverity, EmergencySnapshot,
    EmergencyStatus, EmergencyValidation,
};
pub use environment::{
    EnvironmentConfig, EnvironmentManager, EnvironmentProtection, EnvironmentType,
    PromotionRequest, PromotionResult, PromotionStatus, PromotionWorkflowStatus,
};
pub use notification::{
    ChangeNotificationSystem, DeliveryStatus, LogLevel, LoggingHandler, NotificationConfig,
    NotificationDelivery, NotificationFilter, NotificationHandler, NotificationMethod, SmtpConfig,
    WebhookHandler,
};
pub use repository::{GitRepository, RepositoryStatus};
pub use state::{GitState, GitStateTracker, StateChangeEvent};
pub use types::{
    BranchInfo, CommitInfo, FileChange, FileStatus, GitError, GitResult, RepositoryInfo, TagInfo,
};
pub use validation::{
    BreakingChange, BreakingChangeType, ChangeImpactAssessment, ChangeValidator,
    ChangeValidatorTrait, CompatibilityValidator, DeploymentRecommendation, DeploymentStrategy,
    HealthStatus, ImpactLevel, MaintenanceWindow, PerformanceImpact, PerformanceMetrics,
    PerformanceValidator, RecommendedAction, ResourceConstraints, ResourceUtilization, RiskLevel,
    RollbackAction, RollbackComplexity, RollbackPlan, RollbackTrigger, SafetyCheckConfig,
    SecurityValidator, SemanticValidator, SyntaxValidator, SystemStatus, TestRequirement, TestType,
    ValidationContext, ValidationError, ValidationFinding, ValidationReport, ValidationResult,
    ValidationSeverity, ValidationStatus,
};
