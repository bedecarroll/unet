//! Background SNMP polling implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use tokio::time::Instant;
use uuid::Uuid;

use super::{SessionConfig, SnmpValue};

// Re-export all public types
pub use self::handle::PollingHandle;
pub use self::scheduler::PollingScheduler;

mod handle;
mod scheduler;

/// Configuration for polling scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingConfig {
    /// Default polling interval
    pub default_interval: Duration,
    /// Maximum concurrent polling tasks
    pub max_concurrent_polls: usize,
    /// Timeout for individual polling operations
    pub poll_timeout: Duration,
    /// Maximum number of retries for failed polls
    pub max_retries: u32,
    /// Backoff multiplier for retries
    pub retry_backoff_multiplier: f64,
    /// Health check interval for cleaning up failed tasks
    pub health_check_interval: Duration,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            default_interval: Duration::from_secs(300), // 5 minutes
            max_concurrent_polls: 50,
            poll_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_backoff_multiplier: 2.0,
            health_check_interval: Duration::from_secs(60),
        }
    }
}

/// A single polling task for a specific device and OID set
#[derive(Debug, Clone)]
pub struct PollingTask {
    /// Unique task ID
    pub id: Uuid,
    /// Target device address
    pub target: SocketAddr,
    /// Node ID this task is polling
    pub node_id: Uuid,
    /// OIDs to poll
    pub oids: Vec<String>,
    /// Polling interval
    pub interval: Duration,
    /// SNMP session configuration
    pub session_config: SessionConfig,
    /// Task priority (higher priority tasks run first)
    pub priority: u8,
    /// Whether this task is enabled
    pub enabled: bool,
    /// Task creation timestamp
    pub created_at: SystemTime,
    /// Last successful poll timestamp
    pub last_success: Option<SystemTime>,
    /// Last error encountered
    pub last_error: Option<String>,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
}

impl PollingTask {
    /// Create a new polling task
    #[must_use]
    pub fn new(
        target: SocketAddr,
        node_id: Uuid,
        oids: Vec<String>,
        interval: Duration,
        session_config: SessionConfig,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            target,
            node_id,
            oids,
            interval,
            session_config,
            priority: 128, // Default priority
            enabled: true,
            created_at: SystemTime::now(),
            last_success: None,
            last_error: None,
            consecutive_failures: 0,
        }
    }

    /// Check if task is healthy (recent successful polls)
    #[must_use]
    pub fn is_healthy(&self, max_failure_age: Duration) -> bool {
        self.last_success
            .map_or(self.consecutive_failures == 0, |last_success| {
                SystemTime::now()
                    .duration_since(last_success)
                    .is_ok_and(|age| age <= max_failure_age)
            })
    }

    /// Calculate next poll time based on interval and failures
    #[must_use]
    pub fn next_poll_time(&self) -> Instant {
        let base_interval = self.interval;

        // Apply exponential backoff for failed tasks
        let actual_interval = if self.consecutive_failures > 0 {
            let backoff_factor =
                2_f64.powi(i32::try_from(self.consecutive_failures.min(5)).unwrap_or(5));
            Duration::from_secs_f64(base_interval.as_secs_f64() * backoff_factor)
        } else {
            base_interval
        };

        Instant::now() + actual_interval
    }
}

/// Result of a polling operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingResult {
    /// Task ID that generated this result
    pub task_id: Uuid,
    /// Node ID this result is for
    pub node_id: Uuid,
    /// Target address that was polled
    pub target: SocketAddr,
    /// Timestamp when poll was performed
    pub timestamp: SystemTime,
    /// Whether the poll was successful
    pub success: bool,
    /// SNMP values retrieved (if successful)
    pub values: HashMap<String, SnmpValue>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration of the polling operation
    pub duration: Duration,
}

/// Message types for the polling scheduler
#[derive(Debug)]
pub enum PollingMessage {
    /// Add a new polling task
    AddTask(PollingTask),
    /// Remove a polling task by ID
    RemoveTask(Uuid),
    /// Update an existing polling task
    UpdateTask(PollingTask),
    /// Enable/disable a task
    EnableTask(Uuid, bool),
    /// Get current task status
    GetTaskStatus(Uuid, tokio::sync::oneshot::Sender<Option<PollingTask>>),
    /// List all tasks
    ListTasks(tokio::sync::oneshot::Sender<Vec<PollingTask>>),
    /// Shutdown the scheduler
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::defaults;
    use crate::snmp::SnmpClientConfig;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_polling_task_creation() {
        let target = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            defaults::network::SNMP_DEFAULT_PORT,
        );
        let node_id = Uuid::new_v4();
        let oids = vec!["1.3.6.1.2.1.1.1.0".to_string()];
        let interval = Duration::from_secs(300);
        let session_config = SessionConfig::default();

        let task = PollingTask::new(target, node_id, oids.clone(), interval, session_config);

        assert_eq!(task.target, target);
        assert_eq!(task.node_id, node_id);
        assert_eq!(task.oids, oids);
        assert_eq!(task.interval, interval);
        assert!(task.enabled);
        assert_eq!(task.consecutive_failures, 0);
    }

    #[test]
    fn test_polling_task_health() {
        let target = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            defaults::network::SNMP_DEFAULT_PORT,
        );
        let node_id = Uuid::new_v4();
        let oids = vec!["1.3.6.1.2.1.1.1.0".to_string()];
        let interval = Duration::from_secs(300);
        let session_config = SessionConfig::default();

        let mut task = PollingTask::new(target, node_id, oids, interval, session_config);

        // New task is healthy
        assert!(task.is_healthy(Duration::from_secs(600)));

        // Task with recent success is healthy
        task.last_success = Some(SystemTime::now());
        assert!(task.is_healthy(Duration::from_secs(600)));

        // Task with failures is still healthy if recent success
        task.consecutive_failures = 2;
        assert!(task.is_healthy(Duration::from_secs(600)));
    }

    #[test]
    fn test_polling_config_default() {
        let config = PollingConfig::default();

        assert_eq!(config.default_interval, Duration::from_secs(300));
        assert_eq!(config.max_concurrent_polls, 50);
        assert_eq!(config.max_retries, 3);
        assert!(config.retry_backoff_multiplier > 1.0);
    }

    #[tokio::test]
    async fn test_polling_scheduler_creation() {
        let polling_config = PollingConfig::default();
        let snmp_config = SnmpClientConfig::default();

        let (scheduler, _handle) = PollingScheduler::new(polling_config, snmp_config);

        // Verify scheduler was created with empty task list
        assert_eq!(scheduler.task_count().await, 0);
    }
}
