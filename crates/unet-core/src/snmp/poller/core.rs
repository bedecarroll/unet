//! Core polling scheduler implementation

use super::{PollingConfig, PollingHandle, PollingMessage, PollingResult, PollingTask};
use crate::snmp::{SnmpClient, SnmpClientConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::info;
use uuid::Uuid;

/// Background SNMP polling scheduler
pub struct PollingScheduler {
    /// Configuration
    pub(super) config: PollingConfig,
    /// SNMP client for polling operations
    pub(super) snmp_client: Arc<SnmpClient>,
    /// Active polling tasks
    pub(super) tasks: Arc<RwLock<HashMap<Uuid, PollingTask>>>,
    /// Channel for receiving control messages
    pub(super) message_rx: mpsc::UnboundedReceiver<PollingMessage>,
    /// Channel for sending polling results
    pub(super) result_tx: mpsc::UnboundedSender<PollingResult>,
    /// Shutdown flag
    pub(super) shutdown: Arc<RwLock<bool>>,
    /// Test mode flag - disables background tasks and timers
    #[cfg(test)]
    pub(super) test_mode: bool,
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
            #[cfg(test)]
            test_mode: false,
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
                    if super::management::handle_message(self, message).await {
                        break; // Shutdown requested
                    }
                }

                // Check for tasks that need polling
                _ = poll_interval.tick() => {
                    super::execution::check_and_poll_tasks(self).await;
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

    /// Create scheduler in test mode (for testing main loop)
    #[cfg(test)]
    #[must_use]
    pub fn new_test_mode(
        config: PollingConfig,
        snmp_config: SnmpClientConfig,
    ) -> (Self, PollingHandle) {
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
            test_mode: true,
        };

        let handle = PollingHandle::new(message_tx, result_rx);

        (scheduler, handle)
    }

    /// Run a single iteration of the main loop (for testing)
    #[cfg(test)]
    pub async fn run_single_iteration(&mut self) -> bool {
        // Process any pending messages
        if let Ok(message) = self.message_rx.try_recv() {
            if super::management::handle_message(self, message).await {
                return true; // Shutdown requested
            }
        }

        // Check for tasks that need polling
        super::execution::check_and_poll_tasks(self).await;

        false // Continue running
    }

    /// Start health check task (for testing health check logic)
    #[cfg(test)]
    #[must_use]
    pub fn start_health_check_task(&self) -> tokio::task::JoinHandle<()> {
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
        })
    }

    /// Run health check once (for testing)
    #[cfg(test)]
    pub async fn run_health_check_once(&self) -> usize {
        let mut tasks = self.tasks.write().await;
        let before_count = tasks.len();

        tasks.retain(|_, task| {
            task.is_healthy(self.config.health_check_interval * 3) || task.enabled
        });

        let after_count = tasks.len();
        drop(tasks);
        before_count - after_count
    }

    /// Signal shutdown and wait for cleanup (for testing)
    #[cfg(test)]
    pub async fn shutdown_and_cleanup(&mut self) {
        // Signal shutdown to other tasks
        {
            let mut shutdown = self.shutdown.write().await;
            *shutdown = true;
        }

        info!("SNMP polling scheduler shut down");
    }
}

#[cfg(test)]
#[path = "core_tests.rs"]
mod core_tests;
