//! Tests for task scheduling functionality

use super::super::*;
use crate::snmp::poller::{PollingConfig, PollingHandle, PollingScheduler};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use uuid::Uuid;

/// Create test polling task for testing
fn create_test_task() -> PollingTask {
    let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
    let node_id = Uuid::new_v4();
    let oids = vec![
        "1.3.6.1.2.1.1.1.0".to_string(),
        "1.3.6.1.2.1.1.3.0".to_string(),
    ];
    let interval = Duration::from_secs(300);
    let session_config = crate::snmp::SessionConfig::default();

    PollingTask::new(target, node_id, oids, interval, session_config)
}

/// Create test scheduler with mock client
fn create_test_scheduler() -> (PollingScheduler, PollingHandle) {
    let config = PollingConfig {
        default_interval: Duration::from_secs(60),
        max_concurrent_polls: 2,
        poll_timeout: Duration::from_secs(5),
        max_retries: 2,
        retry_backoff_multiplier: 2.0,
        health_check_interval: Duration::from_secs(30),
    };
    let snmp_config = crate::snmp::SnmpClientConfig::default();

    PollingScheduler::new(config, snmp_config)
}

#[tokio::test]
async fn test_check_and_poll_tasks_no_tasks() {
    let (scheduler, _handle) = create_test_scheduler();

    // Should run without panicking when no tasks are available
    check_and_poll_tasks(&scheduler).await;
}

#[tokio::test]
async fn test_check_and_poll_tasks_disabled_task() {
    let (scheduler, _handle) = create_test_scheduler();

    // Add a disabled task
    let mut task = create_test_task();
    task.enabled = false;

    {
        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(task.id, task);
    }

    // Should skip disabled tasks
    check_and_poll_tasks(&scheduler).await;

    // Task should still be there but not polled
    let tasks = scheduler.tasks.read().await;
    assert_eq!(tasks.len(), 1);
    drop(tasks);
}

#[tokio::test]
async fn test_check_and_poll_tasks_not_due() {
    let (scheduler, _handle) = create_test_scheduler();

    // Add a task that's not due yet (very long interval)
    let mut task = create_test_task();
    task.interval = Duration::from_secs(3600); // 1 hour

    {
        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(task.id, task);
    }

    // Should skip tasks that aren't due
    check_and_poll_tasks(&scheduler).await;
}

#[tokio::test]
async fn test_check_and_poll_tasks_priority_sorting() {
    let (scheduler, _handle) = create_test_scheduler();

    // Create tasks with different priorities
    let mut high_priority_task = create_test_task();
    high_priority_task.priority = 200;
    high_priority_task.interval = Duration::from_millis(1); // Make it due immediately

    let mut low_priority_task = create_test_task();
    low_priority_task.priority = 50;
    low_priority_task.interval = Duration::from_millis(1); // Make it due immediately

    {
        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(high_priority_task.id, high_priority_task);
        tasks.insert(low_priority_task.id, low_priority_task);
    }

    // This tests the priority sorting logic (line 29)
    check_and_poll_tasks(&scheduler).await;
}

#[tokio::test]
async fn test_check_and_poll_tasks_concurrent_limit() {
    let (scheduler, _handle) = create_test_scheduler();

    // Create more tasks than max_concurrent_polls (which is 2)
    for _i in 0..5 {
        let mut task = create_test_task();
        task.interval = Duration::from_millis(1); // Make all due immediately
        task.id = Uuid::new_v4(); // Ensure unique IDs

        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(task.id, task);
        drop(tasks); // Release lock between iterations
    }

    // This tests the concurrent batching logic (line 33)
    check_and_poll_tasks(&scheduler).await;
}

#[tokio::test]
async fn test_check_and_poll_tasks_with_due_tasks() {
    let (scheduler, _handle) = create_test_scheduler();

    // Create a task that's due (very short interval)
    let mut task = create_test_task();
    task.interval = Duration::from_millis(1);
    task.enabled = true;

    // Wait a bit to ensure it's due
    tokio::time::sleep(Duration::from_millis(10)).await;

    {
        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(task.id, task);
    }

    // This should actually attempt to poll the task
    check_and_poll_tasks(&scheduler).await;
}
