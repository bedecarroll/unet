//! Background SNMP polling implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Instant, interval};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{SessionConfig, SnmpClient, SnmpClientConfig, SnmpValue};

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
    pub fn is_healthy(&self, max_failure_age: Duration) -> bool {
        if let Some(last_success) = self.last_success {
            SystemTime::now()
                .duration_since(last_success)
                .map_or(false, |age| age <= max_failure_age)
        } else {
            // New task is considered healthy until it fails
            self.consecutive_failures == 0
        }
    }

    /// Calculate next poll time based on interval and failures
    pub fn next_poll_time(&self) -> Instant {
        let base_interval = self.interval;

        // Apply exponential backoff for failed tasks
        let actual_interval = if self.consecutive_failures > 0 {
            let backoff_factor = 2_f64.powi(self.consecutive_failures.min(5) as i32);
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

/// Background SNMP polling scheduler
pub struct PollingScheduler {
    /// Configuration
    config: PollingConfig,
    /// SNMP client for polling operations
    snmp_client: Arc<SnmpClient>,
    /// Active polling tasks
    tasks: Arc<RwLock<HashMap<Uuid, PollingTask>>>,
    /// Channel for receiving control messages
    message_rx: mpsc::UnboundedReceiver<PollingMessage>,
    /// Channel for sending polling results
    result_tx: mpsc::UnboundedSender<PollingResult>,
    /// Shutdown flag
    shutdown: Arc<RwLock<bool>>,
}

impl PollingScheduler {
    /// Create new polling scheduler
    pub fn new(config: PollingConfig, snmp_config: SnmpClientConfig) -> (Self, PollingHandle) {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (result_tx, result_rx) = mpsc::unbounded_channel();

        let snmp_client = Arc::new(SnmpClient::new(snmp_config));
        let tasks = Arc::new(RwLock::new(HashMap::new()));
        let shutdown = Arc::new(RwLock::new(false));

        let scheduler = Self {
            config,
            snmp_client,
            tasks,
            message_rx,
            result_tx,
            shutdown,
        };

        let handle = PollingHandle {
            message_tx,
            result_rx,
        };

        (scheduler, handle)
    }

    /// Run the polling scheduler (main loop)
    pub async fn run(&mut self) {
        info!("Starting SNMP polling scheduler");

        // Start health check task
        let tasks_for_health = Arc::clone(&self.tasks);
        let shutdown_for_health = Arc::clone(&self.shutdown);
        let health_check_interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval = interval(health_check_interval);

            loop {
                interval.tick().await;

                if *shutdown_for_health.read().await {
                    break;
                }

                // Clean up unhealthy tasks
                let mut tasks = tasks_for_health.write().await;
                let before_count = tasks.len();

                tasks.retain(|_, task| task.is_healthy(health_check_interval * 3) || task.enabled);

                let after_count = tasks.len();
                if before_count != after_count {
                    info!(
                        cleaned_tasks = before_count - after_count,
                        remaining_tasks = after_count,
                        "Cleaned up unhealthy polling tasks"
                    );
                }
            }
        });

        // Main scheduler loop
        let mut poll_interval = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                // Handle control messages
                Some(message) = self.message_rx.recv() => {
                    if self.handle_message(message).await {
                        break; // Shutdown requested
                    }
                }

                // Check for tasks that need polling
                _ = poll_interval.tick() => {
                    self.check_and_poll_tasks().await;
                }
            }
        }

        // Signal shutdown to other tasks
        {
            let mut shutdown = self.shutdown.write().await;
            *shutdown = true;
        }

        info!("SNMP polling scheduler shut down");
    }

    /// Handle control messages
    async fn handle_message(&mut self, message: PollingMessage) -> bool {
        match message {
            PollingMessage::AddTask(task) => {
                info!(task_id = %task.id, target = %task.target, "Adding polling task");
                let mut tasks = self.tasks.write().await;
                tasks.insert(task.id, task);
            }

            PollingMessage::RemoveTask(task_id) => {
                info!(task_id = %task_id, "Removing polling task");
                let mut tasks = self.tasks.write().await;
                tasks.remove(&task_id);
            }

            PollingMessage::UpdateTask(task) => {
                info!(task_id = %task.id, "Updating polling task");
                let mut tasks = self.tasks.write().await;
                tasks.insert(task.id, task);
            }

            PollingMessage::EnableTask(task_id, enabled) => {
                info!(task_id = %task_id, enabled = enabled, "Updating task enabled state");
                let mut tasks = self.tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.enabled = enabled;
                }
            }

            PollingMessage::GetTaskStatus(task_id, response_tx) => {
                let tasks = self.tasks.read().await;
                let task = tasks.get(&task_id).cloned();
                let _ = response_tx.send(task);
            }

            PollingMessage::ListTasks(response_tx) => {
                let tasks = self.tasks.read().await;
                let task_list: Vec<PollingTask> = tasks.values().cloned().collect();
                let _ = response_tx.send(task_list);
            }

            PollingMessage::Shutdown => {
                info!("Shutdown requested");
                return true;
            }
        }

        false
    }

    /// Check tasks and poll those that are due
    async fn check_and_poll_tasks(&self) {
        let now = Instant::now();
        let mut tasks_to_poll = Vec::new();

        // Collect tasks that need polling
        {
            let tasks = self.tasks.read().await;
            for task in tasks.values() {
                if task.enabled && now >= task.next_poll_time() {
                    tasks_to_poll.push(task.clone());
                }
            }
        }

        // Sort by priority (higher first)
        tasks_to_poll.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Limit concurrent polls
        let max_concurrent = self.config.max_concurrent_polls;
        for task_batch in tasks_to_poll.chunks(max_concurrent) {
            let mut poll_handles = Vec::new();

            for task in task_batch {
                let task = task.clone();
                let snmp_client = Arc::clone(&self.snmp_client);
                let result_tx = self.result_tx.clone();
                let poll_timeout = self.config.poll_timeout;

                let handle = tokio::spawn(async move {
                    Self::poll_task(task, snmp_client, result_tx, poll_timeout).await
                });

                poll_handles.push(handle);
            }

            // Wait for this batch to complete
            for handle in poll_handles {
                if let Err(e) = handle.await {
                    error!(error = %e, "Polling task panicked");
                }
            }
        }
    }

    /// Poll a single task
    async fn poll_task(
        mut task: PollingTask,
        snmp_client: Arc<SnmpClient>,
        result_tx: mpsc::UnboundedSender<PollingResult>,
        timeout: Duration,
    ) {
        let start_time = Instant::now();
        let poll_start = SystemTime::now();

        debug!(
            task_id = %task.id,
            target = %task.target,
            oid_count = task.oids.len(),
            "Starting SNMP poll"
        );

        // Perform the SNMP poll with timeout
        let poll_result = tokio::time::timeout(
            timeout,
            snmp_client.get(
                task.target,
                &task.oids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                Some(task.session_config.clone()),
            ),
        )
        .await;

        let duration = start_time.elapsed();

        // Process poll result
        let (success, values, error) = match poll_result {
            Ok(Ok(values)) => {
                task.last_success = Some(poll_start);
                task.consecutive_failures = 0;
                task.last_error = None;
                (true, values, None)
            }
            Ok(Err(e)) => {
                task.consecutive_failures += 1;
                task.last_error = Some(e.to_string());
                (false, HashMap::new(), Some(e.to_string()))
            }
            Err(_) => {
                let error_msg = format!("SNMP poll timeout after {:?}", timeout);
                task.consecutive_failures += 1;
                task.last_error = Some(error_msg.clone());
                (false, HashMap::new(), Some(error_msg))
            }
        };

        // Send result
        let result = PollingResult {
            task_id: task.id,
            node_id: task.node_id,
            target: task.target,
            timestamp: poll_start,
            success,
            values,
            error,
            duration,
        };

        if let Err(e) = result_tx.send(result) {
            error!(error = %e, "Failed to send polling result");
        }

        if success {
            debug!(
                task_id = %task.id,
                target = %task.target,
                duration = ?duration,
                "SNMP poll completed successfully"
            );
        } else {
            warn!(
                task_id = %task.id,
                target = %task.target,
                duration = ?duration,
                consecutive_failures = task.consecutive_failures,
                error = task.last_error.as_deref().unwrap_or("unknown"),
                "SNMP poll failed"
            );
        }
    }
}

/// Handle for controlling the polling scheduler
#[derive(Debug)]
pub struct PollingHandle {
    /// Channel for sending control messages
    message_tx: mpsc::UnboundedSender<PollingMessage>,
    /// Channel for receiving polling results
    pub result_rx: mpsc::UnboundedReceiver<PollingResult>,
}

impl PollingHandle {
    /// Add a new polling task
    pub async fn add_task(&self, task: PollingTask) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::AddTask(task))
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Remove a polling task
    pub async fn remove_task(&self, task_id: Uuid) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::RemoveTask(task_id))
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Update a polling task
    pub async fn update_task(&self, task: PollingTask) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::UpdateTask(task))
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Enable or disable a task
    pub async fn enable_task(&self, task_id: Uuid, enabled: bool) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::EnableTask(task_id, enabled))
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Get status of a specific task
    pub async fn get_task_status(&self, task_id: Uuid) -> Result<Option<PollingTask>, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.message_tx
            .send(PollingMessage::GetTaskStatus(task_id, tx))
            .map_err(|e| format!("Failed to send message: {}", e))?;

        rx.await
            .map_err(|e| format!("Failed to receive response: {}", e))
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Result<Vec<PollingTask>, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.message_tx
            .send(PollingMessage::ListTasks(tx))
            .map_err(|e| format!("Failed to send message: {}", e))?;

        rx.await
            .map_err(|e| format!("Failed to receive response: {}", e))
    }

    /// Shutdown the scheduler
    pub async fn shutdown(&self) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::Shutdown)
            .map_err(|e| format!("Failed to send message: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_polling_task_creation() {
        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161);
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
        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161);
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
        let tasks = scheduler.tasks.read().await;
        assert!(tasks.is_empty());
    }
}
