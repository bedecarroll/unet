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
