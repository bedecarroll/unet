//! Polling scheduler implementation

use super::{PollingConfig, PollingHandle, PollingMessage, PollingResult, PollingTask};
use crate::snmp::{SnmpClient, SnmpClientConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::time::{Instant, interval};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

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
    #[must_use]
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

        let handle = PollingHandle::new(message_tx, result_rx);

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
                drop(tasks);
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

    /// Get number of active tasks (for testing)
    #[cfg(test)]
    pub async fn task_count(&self) -> usize {
        self.tasks.read().await.len()
    }

    /// Handle control messages
    async fn handle_message(&self, message: PollingMessage) -> bool {
        match message {
            PollingMessage::AddTask(task) => {
                self.handle_add_task(task).await;
            }
            PollingMessage::RemoveTask(task_id) => {
                self.handle_remove_task(task_id).await;
            }
            PollingMessage::UpdateTask(task) => {
                self.handle_update_task(task).await;
            }
            PollingMessage::EnableTask(task_id, enabled) => {
                self.handle_enable_task(task_id, enabled).await;
            }
            PollingMessage::GetTaskStatus(task_id, response_tx) => {
                self.handle_get_task_status(task_id, response_tx).await;
            }
            PollingMessage::ListTasks(response_tx) => {
                self.handle_list_tasks(response_tx).await;
            }
            PollingMessage::Shutdown => {
                info!("Shutdown requested");
                return true;
            }
        }
        false
    }

    async fn handle_add_task(&self, task: PollingTask) {
        info!(task_id = %task.id, target = %task.target, "Adding polling task");
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id, task);
    }

    async fn handle_remove_task(&self, task_id: Uuid) {
        info!(task_id = %task_id, "Removing polling task");
        let mut tasks = self.tasks.write().await;
        tasks.remove(&task_id);
    }

    async fn handle_update_task(&self, task: PollingTask) {
        info!(task_id = %task.id, "Updating polling task");
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id, task);
    }

    async fn handle_enable_task(&self, task_id: Uuid, enabled: bool) {
        info!(task_id = %task_id, enabled = enabled, "Updating task enabled state");
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.enabled = enabled;
        }
    }

    async fn handle_get_task_status(
        &self,
        task_id: Uuid,
        response_tx: oneshot::Sender<Option<PollingTask>>,
    ) {
        let task = self.tasks.read().await.get(&task_id).cloned();
        // Intentionally ignore send result - receiver may have dropped
        let _ = response_tx.send(task);
    }

    async fn handle_list_tasks(&self, response_tx: oneshot::Sender<Vec<PollingTask>>) {
        let task_list: Vec<PollingTask> = self.tasks.read().await.values().cloned().collect();
        // Intentionally ignore send result - receiver may have dropped
        let _ = response_tx.send(task_list);
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
                    Self::poll_task(task, snmp_client, result_tx, poll_timeout).await;
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

        let poll_result = Self::execute_snmp_poll(&task, &snmp_client, timeout).await;
        let duration = start_time.elapsed();
        let (success, values, error) = Self::process_poll_result(poll_result, &mut task, timeout);
        let result =
            Self::create_polling_result(&task, poll_start, success, values, error, duration);

        Self::send_result(result, &result_tx);
        Self::log_poll_completion(&task, success, duration);
    }

    async fn execute_snmp_poll(
        task: &PollingTask,
        snmp_client: &SnmpClient,
        timeout: Duration,
    ) -> Result<
        Result<HashMap<String, crate::snmp::SnmpValue>, crate::snmp::SnmpError>,
        tokio::time::error::Elapsed,
    > {
        tokio::time::timeout(
            timeout,
            snmp_client.get(
                task.target,
                &task
                    .oids
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<_>>(),
                Some(task.session_config.clone()),
            ),
        )
        .await
    }

    fn process_poll_result(
        poll_result: Result<
            Result<HashMap<String, crate::snmp::SnmpValue>, crate::snmp::SnmpError>,
            tokio::time::error::Elapsed,
        >,
        task: &mut PollingTask,
        timeout: Duration,
    ) -> (
        bool,
        HashMap<String, crate::snmp::SnmpValue>,
        Option<String>,
    ) {
        match poll_result {
            Ok(Ok(values)) => {
                task.last_success = Some(SystemTime::now());
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
                let error_msg = format!("SNMP poll timeout after {timeout:?}");
                task.consecutive_failures += 1;
                task.last_error = Some(error_msg.clone());
                (false, HashMap::new(), Some(error_msg))
            }
        }
    }

    const fn create_polling_result(
        task: &PollingTask,
        poll_start: SystemTime,
        success: bool,
        values: HashMap<String, crate::snmp::SnmpValue>,
        error: Option<String>,
        duration: Duration,
    ) -> PollingResult {
        PollingResult {
            task_id: task.id,
            node_id: task.node_id,
            target: task.target,
            timestamp: poll_start,
            success,
            values,
            error,
            duration,
        }
    }

    fn send_result(result: PollingResult, result_tx: &mpsc::UnboundedSender<PollingResult>) {
        if let Err(e) = result_tx.send(result) {
            error!(error = %e, "Failed to send polling result");
        }
    }

    fn log_poll_completion(task: &PollingTask, success: bool, duration: Duration) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snmp::{SnmpClientConfig, SnmpValue};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::{Duration, SystemTime};
    use tokio::time::timeout;

    fn create_test_config() -> PollingConfig {
        PollingConfig {
            default_interval: Duration::from_millis(100),
            max_concurrent_polls: 2,
            poll_timeout: Duration::from_millis(50),
            max_retries: 2,
            retry_backoff_multiplier: 2.0,
            health_check_interval: Duration::from_millis(100),
        }
    }

    fn create_test_task() -> PollingTask {
        PollingTask::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
            Uuid::new_v4(),
            vec!["1.3.6.1.2.1.1.1.0".to_string()],
            Duration::from_millis(100),
            crate::snmp::SessionConfig::default(),
        )
    }

    #[tokio::test]
    async fn test_scheduler_basic_operations() {
        let config = create_test_config();
        let snmp_config = SnmpClientConfig::default();
        let (scheduler, handle) = PollingScheduler::new(config, snmp_config);

        // Test initial task count
        assert_eq!(scheduler.task_count().await, 0);

        // Test add task (lines 154-158)
        let task = create_test_task();
        let task_id = task.id;
        assert!(handle.add_task(task).is_ok());

        // Give scheduler time to process message
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Test remove task (lines 160-164)
        assert!(handle.remove_task(task_id).is_ok());

        // Test shutdown (lines 146-148)
        assert!(handle.shutdown().is_ok());
    }

    #[tokio::test]
    async fn test_scheduler_task_management() {
        let config = create_test_config();
        let snmp_config = SnmpClientConfig::default();
        let (mut scheduler, handle) = PollingScheduler::new(config, snmp_config);

        let task = create_test_task();
        let task_id = task.id;

        // Run scheduler in background
        let scheduler_handle = tokio::spawn(async move {
            scheduler.run().await;
        });

        // Test add task
        assert!(handle.add_task(task.clone()).is_ok());
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Test update task (lines 166-170)
        let mut updated_task = task.clone();
        updated_task.priority = 200;
        assert!(handle.update_task(updated_task).is_ok());
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Test enable/disable task (lines 172-178)
        assert!(handle.enable_task(task_id, false).is_ok());
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Test get task status (lines 180-188)
        let status_result =
            timeout(Duration::from_millis(50), handle.get_task_status(task_id)).await;
        assert!(status_result.is_ok());

        // Test list tasks (lines 190-194)
        let list_result = timeout(Duration::from_millis(50), handle.list_tasks()).await;
        assert!(list_result.is_ok());

        // Test shutdown - this will stop the scheduler
        assert!(handle.shutdown().is_ok());

        // Wait for scheduler to shut down
        let _ = timeout(Duration::from_millis(100), scheduler_handle).await;
    }

    #[tokio::test]
    async fn test_scheduler_error_handling() {
        let config = create_test_config();
        let snmp_config = SnmpClientConfig::default();
        let (mut scheduler, handle) = PollingScheduler::new(config, snmp_config);

        // Run scheduler in background
        let scheduler_handle = tokio::spawn(async move {
            scheduler.run().await;
        });

        // Test operations on non-existent task
        let fake_task_id = Uuid::new_v4();

        // Test enable task that doesn't exist (line 175-177 won't execute)
        assert!(handle.enable_task(fake_task_id, true).is_ok());
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Test get status for non-existent task
        let status_result = timeout(
            Duration::from_millis(50),
            handle.get_task_status(fake_task_id),
        )
        .await;
        assert!(status_result.is_ok());
        if let Ok(Ok(status)) = status_result {
            assert!(status.is_none());
        }

        assert!(handle.shutdown().is_ok());

        // Wait for scheduler to shut down
        let _ = timeout(Duration::from_millis(100), scheduler_handle).await;
    }

    #[tokio::test]
    async fn test_poll_result_processing() {
        use crate::snmp::{SnmpError, SnmpValue};
        use std::collections::HashMap;

        let mut task = create_test_task();
        let timeout_duration = Duration::from_millis(50);

        // Test successful poll result (lines 304-309)
        let mut values = HashMap::new();
        values.insert(
            "1.3.6.1.2.1.1.1.0".to_string(),
            SnmpValue::String("test".to_string()),
        );
        let success_result = Ok(Ok(values.clone()));
        let (success, returned_values, error) =
            PollingScheduler::process_poll_result(success_result, &mut task, timeout_duration);
        assert!(success);
        assert_eq!(returned_values, values);
        assert!(error.is_none());
        assert_eq!(task.consecutive_failures, 0);
        assert!(task.last_success.is_some());

        // Test SNMP error result (lines 310-314)
        task.consecutive_failures = 0;
        let snmp_error = SnmpError::Protocol {
            message: "test error".to_string(),
        };
        let error_result = Ok(Err(snmp_error));
        let (success, returned_values, error) =
            PollingScheduler::process_poll_result(error_result, &mut task, timeout_duration);
        assert!(!success);
        assert!(returned_values.is_empty());
        assert!(error.is_some());
        assert_eq!(task.consecutive_failures, 1);

        // Test timeout error result (lines 315-321)
        task.consecutive_failures = 0;
        // Create a timeout error by actually timing out a future
        let timeout_result = tokio::time::timeout(
            Duration::from_millis(1),
            tokio::time::sleep(Duration::from_millis(10)),
        )
        .await;
        let timeout_error = timeout_result.map(|()| Ok(HashMap::new()));
        let (success, returned_values, error) =
            PollingScheduler::process_poll_result(timeout_error, &mut task, timeout_duration);
        assert!(!success);
        assert!(returned_values.is_empty());
        assert!(error.is_some());
        assert!(error.unwrap().contains("timeout"));
        assert_eq!(task.consecutive_failures, 1);
    }

    #[tokio::test]
    async fn test_send_result_error_handling() {
        let (tx, rx) = mpsc::unbounded_channel();

        // Drop receiver to test error case (lines 344-347)
        drop(rx);

        let result = PollingResult {
            task_id: Uuid::new_v4(),
            node_id: Uuid::new_v4(),
            target: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
            timestamp: SystemTime::now(),
            success: true,
            values: HashMap::new(),
            error: None,
            duration: Duration::from_millis(10),
        };

        // This should not panic, just log an error
        PollingScheduler::send_result(result, &tx);
    }

    #[test]
    fn test_create_polling_result() {
        let task = create_test_task();
        let poll_start = SystemTime::now();
        let duration = Duration::from_millis(100);
        let mut values = HashMap::new();
        values.insert("test".to_string(), SnmpValue::Integer(42));

        // Test successful result creation (lines 324-342)
        let result = PollingScheduler::create_polling_result(
            &task,
            poll_start,
            true,
            values.clone(),
            None,
            duration,
        );

        assert_eq!(result.task_id, task.id);
        assert_eq!(result.node_id, task.node_id);
        assert_eq!(result.target, task.target);
        assert_eq!(result.timestamp, poll_start);
        assert!(result.success);
        assert_eq!(result.values, values);
        assert!(result.error.is_none());
        assert_eq!(result.duration, duration);

        // Test error result creation
        let error_result = PollingScheduler::create_polling_result(
            &task,
            poll_start,
            false,
            HashMap::new(),
            Some("test error".to_string()),
            duration,
        );

        assert!(!error_result.success);
        assert!(error_result.values.is_empty());
        assert_eq!(error_result.error, Some("test error".to_string()));
    }

    #[test]
    fn test_log_poll_completion() {
        let task = create_test_task();
        let duration = Duration::from_millis(100);

        // Test successful completion logging (lines 350-358)
        PollingScheduler::log_poll_completion(&task, true, duration);

        // Test failed completion logging (lines 359-367)
        let mut failed_task = task;
        failed_task.consecutive_failures = 3;
        failed_task.last_error = Some("test error".to_string());
        PollingScheduler::log_poll_completion(&failed_task, false, duration);
    }
}
