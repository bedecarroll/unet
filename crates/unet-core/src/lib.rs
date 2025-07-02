//! μNet Core Library
//!
//! This library provides the core functionality for μNet network configuration management,
//! including data models, storage abstractions, policy engine, template engine, and SNMP integration.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(missing_docs)]
//!
//! # Quick Start
//!
//! ```rust
//! use unet_core::{models::*, datastore::*};
//!
//! // Create a new node using the builder pattern
//! let node = NodeBuilder::new()
//!     .name("router-01".to_string())
//!     .domain("example.com".to_string())
//!     .vendor(Vendor::Cisco)
//!     .model("ISR4431".to_string())
//!     .role(DeviceRole::Router)
//!     .lifecycle(Lifecycle::Live)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Architecture
//!
//! The library is organized into several modules:
//!
//! - [`models`] - Core data models (Node, Link, Location)
//! - [`datastore`] - Storage abstraction layer with multiple backends
//! - [`error`] - Unified error types and handling
//! - [`config`] - Configuration management (Milestone 1.3.3)
//! - [`policy`] - Policy engine (Milestone 3)
//! - [`snmp`] - SNMP integration (Milestone 2)
//! - [`template`] - Template rendering (Milestone 4)

// Public modules
pub mod alerting;
pub mod auth;
pub mod change_tracking;
pub mod cluster;
pub mod config;
pub mod config_encryption;
pub mod database;
pub mod datastore;
pub mod distributed_locking;
pub mod entities;
pub mod error;
pub mod escalation;
pub mod git;
pub mod live_config;
pub mod load_balancer;
pub mod logging;
pub mod metrics;
pub mod models;
pub mod notifications;
pub mod performance;
pub mod policy;
pub mod policy_integration;
pub mod resource_management;
pub mod secrets;
pub mod secrets_integration;
pub mod shared_state;
pub mod snmp;
pub mod stateless;
pub mod template;
pub mod template_integration;
pub mod validation;

// Re-exports for convenience
pub use error::{Error, Result};

/// Prelude module for commonly used types
///
/// Common imports for μNet Core
///
/// This module provides convenient re-exports of the most commonly used types.
pub mod prelude {

    // Core error types
    pub use crate::error::{Error, Result};

    // Data models
    pub use crate::models::{
        DeviceRole, Lifecycle, Link, LinkBuilder, Location, LocationBuilder, Node, NodeBuilder,
        Vendor,
    };

    // Derived state models
    pub use crate::models::derived::{
        EnvironmentalMetrics, InterfaceAdminStatus, InterfaceOperStatus, InterfaceStats,
        InterfaceStatus, NodeStatus, PerformanceMetrics, SystemInfo,
    };

    // Template models
    pub use crate::models::template::{
        Template, TemplateAssignment, TemplateRenderRequest, TemplateRenderResult, TemplateUsage,
        TemplateVersion,
    };

    // DataStore trait and common types
    pub use crate::datastore::{
        BatchOperation,
        BatchResult,
        DataStore,
        DataStoreError,
        DataStoreResult,
        Filter,
        FilterOperation,
        FilterValue,
        PagedResult,
        Pagination,
        QueryOptions,
        Sort,
        SortDirection,
        // Helper functions
        filter_contains,
        filter_equals_string,
        filter_equals_uuid,
        sort_asc,
        sort_desc,
    };

    // Configuration and logging
    pub use crate::config::Config;
    pub use crate::logging::{init_default_tracing, init_tracing};

    // SNMP types
    pub use crate::snmp::{
        OidMap, PollingConfig, PollingHandle, PollingResult, PollingScheduler, PollingTask,
        SessionConfig, SnmpClient, SnmpClientConfig, SnmpCredentials, SnmpError, SnmpResult,
        SnmpValue, StandardOid, VendorOid,
    };

    // Git integration types
    pub use crate::git::{
        BranchInfo, CommitInfo, FileChange, FileStatus, GitClient, GitClientConfig,
        GitCredentialProvider, GitCredentials, GitError, GitRepository, GitResult, GitState,
        GitStateTracker, MemoryCredentialProvider, RepositoryInfo, RepositoryStatus,
        StateChangeEvent,
    };

    // Policy integration types
    pub use crate::policy_integration::{
        DefaultPolicyEvaluationEngine, PolicyEvaluationEngine, PolicyService,
    };

    // Template testing framework
    pub use crate::template::{
        IntegrationTestResult, PerformanceTestResult, RegressionTestResult,
        TemplateIntegrationTest, TemplatePerformanceTest, TemplateRegressionTest,
        TemplateTestFramework, TemplateUnitTest, TestFrameworkConfig, TestRegistry,
        TestSuiteResult, TestSummary, UnitTestResult,
    };

    // Template quality analysis
    pub use crate::template::{
        DirectoryQualityReport, DirectoryQualitySummary, QualityAnalysisConfig,
        TemplateDocumentation, TemplateLintingResults, TemplatePerformanceAnalysis,
        TemplateQualityAnalyzer, TemplateQualityReport, TemplateSecurityScan, TemplateVariable,
        UsageExample,
    };

    // Alerting types
    pub use crate::alerting::{
        Alert, AlertCondition, AlertRule, AlertSeverity, AlertStatus, AlertType, AlertingConfig,
        AlertingManager, EscalationPolicy, EscalationStep, NotificationChannel, RateLimitConfig,
    };

    // Notification types
    pub use crate::notifications::{
        DeliveryResult, EmailProvider, MessagePriority, NotificationError, NotificationManager,
        NotificationMessage, NotificationProvider, NotificationResult, PagerDutyProvider,
        SlackProvider, WebhookProvider,
    };

    // Escalation types
    pub use crate::escalation::{
        AlertEscalation, EscalationConfig, EscalationEngine, EscalationError, EscalationEvent,
        EscalationEventType, EscalationResult, EscalationStats, EscalationStatus,
    };

    // Performance optimization types
    pub use crate::performance::{
        AsyncProcessingOptimizer, BenchmarkConfig, BenchmarkResult, Cache, CacheStats,
        ConnectionPool, ConnectionPoolStats, OperationTimer, PerformanceBenchmark,
        PerformanceManager, PerformanceProfiler, PerformanceReport, PooledConnection,
    };

    // Load balancer compatibility types
    pub use crate::load_balancer::{
        ComponentHealth, CustomHealthCheck, HealthCheckConfig, HealthCheckResult,
        HealthCriticality, HealthEndpoint, HealthStatus, LoadBalancerConfig,
        LoadBalancerHealthManager, LoadBalancerInfo, LoadBalancerType, RuntimeInfo,
        ServiceMetadata,
    };

    // Shared state types
    pub use crate::shared_state::{
        InMemoryStateProvider, RedisStateProvider, SharedStateConfig, SharedStateManager,
        SharedStateProvider,
    };

    // Stateless operation types
    pub use crate::stateless::{
        BackgroundTaskConfig, DistributedLockGuard, DistributedLockingConfig, JwtSessionConfig,
        LeaderElectionConfig, LockRetryConfig, RedisPoolConfig, SessionManagementConfig,
        SlidingWindowConfig, StatelessConfig, StatelessHealthStatus, StatelessManager,
        StatelessRateLimitingConfig, StatelessSharedState,
    };

    // Distributed locking types
    pub use crate::distributed_locking::{
        DeadlockDetector, DeadlockInfo, DistributedLock, DistributedLockConfig,
        DistributedLockManager, DistributedLockProvider, DistributedMutex, LeaderElection,
        LockInfo, LockMonitorReport, LockStats, LockType, PostgresLockProvider, RedisLockProvider,
    };

    // Cluster coordination types
    pub use crate::cluster::{
        ClusterConfig, ClusterManager, ClusterMembership, ClusterNode, ClusterStats,
        CustomHealthCheck as ClusterCustomHealthCheck,
        HealthCheckResult as ClusterHealthCheckResult, HealthStatus as ClusterHealthStatus,
        NodeCapacity, NodeConfig, NodeHealth, NodeMetrics, ResponseTimeMetrics,
        ServiceDiscoveryConfig,
    };

    // Resource management types
    pub use crate::resource_management::{
        AlertManager, AlertSeverity as ResourceAlertSeverity, CacheConfig, CacheManager,
        CapacityPlanner, CapacityPlanningConfig, CapacityRecommendation, CircuitBreaker,
        CircuitBreakerConfig, CircuitBreakerState, CpuLimitsConfig, CpuMonitor, DegradationManager,
        FallbackConfig, FallbackManager, GracefulDegradationConfig, LimitsManager, MemoryConfig,
        MemoryLimitsConfig, MemoryManager, MemoryMonitor, MemoryPoolConfig, MemoryPoolManager,
        MonitoringManager, QuotaManager, RequestThrottler, ResourceAlert, ResourceConfig,
        ResourceLimitsConfig, ResourceManager, ResourceMonitoringConfig, ResourceQuota,
        ResourceQuotasConfig, ResourceStatus, SystemMetrics, ThrottlingConfig,
    };
}
