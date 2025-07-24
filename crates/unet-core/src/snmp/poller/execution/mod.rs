//! Task execution and polling logic for SNMP scheduler

use super::core::PollingScheduler;
use super::{PollingResult, PollingTask};
use crate::snmp::{SnmpClient, SnmpValue};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, error, warn};

/// Check tasks and poll those that are due
pub async fn check_and_poll_tasks(scheduler: &PollingScheduler) {
    let now = Instant::now();
    let mut tasks_to_poll = Vec::new();

    // Collect tasks that need polling
    {
        let tasks = scheduler.tasks.read().await;
        for task in tasks.values() {
            if task.enabled && now >= task.next_poll_time() {
                tasks_to_poll.push(task.clone());
            }
        }
    }

    // Sort by priority (higher first)
    tasks_to_poll.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Limit concurrent polls
    let max_concurrent = scheduler.config.max_concurrent_polls;
    for task_batch in tasks_to_poll.chunks(max_concurrent) {
        let mut poll_handles = Vec::new();

        for task in task_batch {
            let task = task.clone();
            let snmp_client = Arc::clone(&scheduler.snmp_client);
            let result_tx = scheduler.result_tx.clone();
            let poll_timeout = scheduler.config.poll_timeout;

            let handle = tokio::spawn(async move {
                poll_task(task, snmp_client, result_tx, poll_timeout).await;
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

    let poll_result = execute_snmp_poll(&task, &snmp_client, timeout).await;
    let duration = start_time.elapsed();
    let (success, values, error) = process_poll_result(poll_result, &mut task, timeout);
    let result = create_polling_result(&task, poll_start, success, values, error, duration);

    send_result(result, &result_tx);
    log_poll_completion(&task, success, duration);
}

pub async fn execute_snmp_poll(
    task: &PollingTask,
    snmp_client: &SnmpClient,
    timeout: Duration,
) -> Result<Result<HashMap<String, SnmpValue>, crate::snmp::SnmpError>, tokio::time::error::Elapsed>
{
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

pub fn process_poll_result(
    poll_result: Result<
        Result<HashMap<String, SnmpValue>, crate::snmp::SnmpError>,
        tokio::time::error::Elapsed,
    >,
    task: &mut PollingTask,
    timeout: Duration,
) -> (bool, HashMap<String, SnmpValue>, Option<String>) {
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

pub const fn create_polling_result(
    task: &PollingTask,
    poll_start: SystemTime,
    success: bool,
    values: HashMap<String, SnmpValue>,
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

pub fn send_result(result: PollingResult, result_tx: &mpsc::UnboundedSender<PollingResult>) {
    if let Err(e) = result_tx.send(result) {
        error!(error = %e, "Failed to send polling result");
    }
}

pub fn log_poll_completion(task: &PollingTask, success: bool, duration: Duration) {
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
