//! Task management and message handling for polling scheduler

use super::core::PollingScheduler;
use super::{PollingMessage, PollingTask};
use tokio::sync::oneshot;
use tracing::info;
use uuid::Uuid;

/// Handle control messages
pub async fn handle_message(scheduler: &PollingScheduler, message: PollingMessage) -> bool {
    match message {
        PollingMessage::AddTask(task) => {
            handle_add_task(scheduler, task).await;
        }
        PollingMessage::RemoveTask(task_id) => {
            handle_remove_task(scheduler, task_id).await;
        }
        PollingMessage::UpdateTask(task) => {
            handle_update_task(scheduler, task).await;
        }
        PollingMessage::EnableTask(task_id, enabled) => {
            handle_enable_task(scheduler, task_id, enabled).await;
        }
        PollingMessage::GetTaskStatus(task_id, response_tx) => {
            handle_get_task_status(scheduler, task_id, response_tx).await;
        }
        PollingMessage::ListTasks(response_tx) => {
            handle_list_tasks(scheduler, response_tx).await;
        }
        PollingMessage::Shutdown => {
            info!("Shutdown requested");
            return true;
        }
    }
    false
}

async fn handle_add_task(scheduler: &PollingScheduler, task: PollingTask) {
    info!(task_id = %task.id, target = %task.target, "Adding polling task");
    let mut tasks = scheduler.tasks.write().await;
    tasks.insert(task.id, task);
}

async fn handle_remove_task(scheduler: &PollingScheduler, task_id: Uuid) {
    info!(task_id = %task_id, "Removing polling task");
    let mut tasks = scheduler.tasks.write().await;
    tasks.remove(&task_id);
}

async fn handle_update_task(scheduler: &PollingScheduler, task: PollingTask) {
    info!(task_id = %task.id, "Updating polling task");
    let mut tasks = scheduler.tasks.write().await;
    tasks.insert(task.id, task);
}

async fn handle_enable_task(scheduler: &PollingScheduler, task_id: Uuid, enabled: bool) {
    info!(task_id = %task_id, enabled = enabled, "Updating task enabled state");
    let mut tasks = scheduler.tasks.write().await;
    if let Some(task) = tasks.get_mut(&task_id) {
        task.enabled = enabled;
    }
}

async fn handle_get_task_status(
    scheduler: &PollingScheduler,
    task_id: Uuid,
    response_tx: oneshot::Sender<Option<PollingTask>>,
) {
    let task = scheduler.tasks.read().await.get(&task_id).cloned();
    // Intentionally ignore send result - receiver may have dropped
    let _ = response_tx.send(task);
}

async fn handle_list_tasks(
    scheduler: &PollingScheduler,
    response_tx: oneshot::Sender<Vec<PollingTask>>,
) {
    let task_list: Vec<PollingTask> = scheduler.tasks.read().await.values().cloned().collect();
    // Intentionally ignore send result - receiver may have dropped
    let _ = response_tx.send(task_list);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{PollingConfig, PollingScheduler};
    use crate::snmp::{SessionConfig, SnmpClientConfig};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;
    use tokio::sync::oneshot;
    use uuid::Uuid;

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
            SessionConfig::default(),
        )
    }

    #[tokio::test]
    async fn test_add_and_remove_task() {
        let config = create_test_config();
        let snmp_config = SnmpClientConfig::default();
        let (scheduler, _handle) = PollingScheduler::new(config, snmp_config);

        let task = create_test_task();
        let task_id = task.id;

        // Add task
        handle_add_task(&scheduler, task).await;
        assert_eq!(scheduler.task_count().await, 1);

        // Remove task
        handle_remove_task(&scheduler, task_id).await;
        assert_eq!(scheduler.task_count().await, 0);
    }

    #[tokio::test]
    async fn test_enable_disable_task() {
        let config = create_test_config();
        let snmp_config = SnmpClientConfig::default();
        let (scheduler, _handle) = PollingScheduler::new(config, snmp_config);

        let task = create_test_task();
        let task_id = task.id;

        // Add task
        handle_add_task(&scheduler, task).await;

        // Disable task
        handle_enable_task(&scheduler, task_id, false).await;

        // Check task is disabled
        let (tx, rx) = oneshot::channel();
        handle_get_task_status(&scheduler, task_id, tx).await;
        let task_status = rx.await.unwrap();
        assert!(task_status.is_some());
        assert!(!task_status.unwrap().enabled);
    }
}
