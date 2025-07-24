use super::super::{PollingConfig, PollingTask, SessionConfig};
use super::PollingScheduler;
use crate::snmp::SnmpClientConfig;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
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

#[tokio::test]
async fn test_scheduler_creation() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new(config.clone(), snmp_config);

    assert_eq!(scheduler.task_count().await, 0);
    assert_eq!(scheduler.config.default_interval, config.default_interval);
    assert_eq!(
        scheduler.config.max_concurrent_polls,
        config.max_concurrent_polls
    );
    assert_eq!(scheduler.config.poll_timeout, config.poll_timeout);
}

#[tokio::test]
async fn test_scheduler_shutdown_flag() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new(config, snmp_config);

    // Initially shutdown should be false
    assert!(!*scheduler.shutdown.read().await);

    // Can set shutdown flag
    {
        let mut shutdown = scheduler.shutdown.write().await;
        *shutdown = true;
    }
    assert!(*scheduler.shutdown.read().await);
}

#[tokio::test]
async fn test_scheduler_configuration_values() {
    let config = PollingConfig {
        default_interval: Duration::from_secs(10),
        max_concurrent_polls: 5,
        poll_timeout: Duration::from_secs(30),
        max_retries: 3,
        retry_backoff_multiplier: 1.5,
        health_check_interval: Duration::from_secs(60),
    };
    let snmp_config = SnmpClientConfig {
        max_connections: 50,
        ..Default::default()
    };

    let (scheduler, _handle) = PollingScheduler::new(config, snmp_config);

    // Verify config values are stored correctly
    assert_eq!(scheduler.config.default_interval, Duration::from_secs(10));
    assert_eq!(scheduler.config.max_concurrent_polls, 5);
    assert_eq!(scheduler.config.poll_timeout, Duration::from_secs(30));
    assert_eq!(scheduler.config.max_retries, 3);
    assert!((scheduler.config.retry_backoff_multiplier - 1.5).abs() < f64::EPSILON);
    assert_eq!(
        scheduler.config.health_check_interval,
        Duration::from_secs(60)
    );
}

#[tokio::test]
async fn test_scheduler_components_initialization() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, handle) = PollingScheduler::new(config, snmp_config);

    // Verify scheduler components are properly initialized
    assert!(scheduler.tasks.read().await.is_empty());
    assert!(!*scheduler.shutdown.read().await);

    // Handle should be properly connected
    drop(handle); // Should not panic

    // SNMP client should be initialized
    assert_eq!(scheduler.task_count().await, 0);
}

#[test]
fn test_polling_config_creation() {
    let config = PollingConfig {
        default_interval: Duration::from_millis(500),
        max_concurrent_polls: 10,
        poll_timeout: Duration::from_millis(1000),
        max_retries: 5,
        retry_backoff_multiplier: 2.5,
        health_check_interval: Duration::from_millis(2000),
    };

    assert_eq!(config.default_interval, Duration::from_millis(500));
    assert_eq!(config.max_concurrent_polls, 10);
    assert_eq!(config.poll_timeout, Duration::from_millis(1000));
    assert_eq!(config.max_retries, 5);
    assert!((config.retry_backoff_multiplier - 2.5).abs() < f64::EPSILON);
    assert_eq!(config.health_check_interval, Duration::from_millis(2000));
}

#[tokio::test]
async fn test_scheduler_with_custom_snmp_config() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig {
        max_connections: 25,
        health_check_interval: Duration::from_secs(30),
        session_timeout: Duration::from_secs(120),
        ..Default::default()
    };

    let (scheduler, _handle) = PollingScheduler::new(config, snmp_config);

    // Scheduler should be created successfully with custom SNMP config
    assert_eq!(scheduler.task_count().await, 0);
}

#[tokio::test]
async fn test_scheduler_test_mode_creation() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    assert!(scheduler.test_mode);
    assert_eq!(scheduler.task_count().await, 0);
}

#[tokio::test]
async fn test_scheduler_single_iteration_no_messages() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (mut scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Should return false (continue running) when no messages or tasks
    let should_shutdown = scheduler.run_single_iteration().await;
    assert!(!should_shutdown);
}

#[tokio::test]
async fn test_scheduler_single_iteration_with_shutdown_message() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (mut scheduler, handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Send shutdown message
    handle.shutdown().expect("Should send shutdown message");

    // Should return true (shutdown requested)
    let should_shutdown = scheduler.run_single_iteration().await;
    assert!(should_shutdown);
}

#[tokio::test]
async fn test_scheduler_health_check_no_tasks() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Should clean up 0 tasks when no tasks exist
    let cleaned_count = scheduler.run_health_check_once().await;
    assert_eq!(cleaned_count, 0);
}

#[tokio::test]
async fn test_scheduler_health_check_with_healthy_tasks() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Add a healthy task
    let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
    let node_id = Uuid::new_v4();
    let oids = vec!["1.3.6.1.2.1.1.1.0".to_string()];
    let interval = Duration::from_secs(300);
    let session_config = SessionConfig::default();

    let task = PollingTask::new(target, node_id, oids, interval, session_config);
    let task_id = task.id;

    {
        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(task_id, task);
    }

    // Should not clean up healthy tasks
    let cleaned_count = scheduler.run_health_check_once().await;
    assert_eq!(cleaned_count, 0);
    assert_eq!(scheduler.task_count().await, 1);
}

#[tokio::test]
async fn test_scheduler_health_check_with_unhealthy_tasks() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Add an unhealthy disabled task
    let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
    let node_id = Uuid::new_v4();
    let oids = vec!["1.3.6.1.2.1.1.1.0".to_string()];
    let interval = Duration::from_secs(300);
    let session_config = SessionConfig::default();

    let mut task = PollingTask::new(target, node_id, oids, interval, session_config);
    task.enabled = false;
    task.consecutive_failures = 10; // Make it unhealthy
    let task_id = task.id;

    {
        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(task_id, task);
    }

    // Should clean up unhealthy disabled tasks
    let cleaned_count = scheduler.run_health_check_once().await;
    assert_eq!(cleaned_count, 1);
    assert_eq!(scheduler.task_count().await, 0);
}

#[tokio::test]
async fn test_scheduler_health_check_enabled_tasks_kept() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Add an unhealthy but enabled task
    let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
    let node_id = Uuid::new_v4();
    let oids = vec!["1.3.6.1.2.1.1.1.0".to_string()];
    let interval = Duration::from_secs(300);
    let session_config = SessionConfig::default();

    let mut task = PollingTask::new(target, node_id, oids, interval, session_config);
    task.enabled = true; // Keep enabled
    task.consecutive_failures = 10; // Make it unhealthy
    let task_id = task.id;

    {
        let mut tasks = scheduler.tasks.write().await;
        tasks.insert(task_id, task);
    }

    // Should not clean up enabled tasks even if unhealthy
    let cleaned_count = scheduler.run_health_check_once().await;
    assert_eq!(cleaned_count, 0);
    assert_eq!(scheduler.task_count().await, 1);
}

#[tokio::test]
async fn test_scheduler_shutdown_and_cleanup() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (mut scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Initially not shut down
    assert!(!*scheduler.shutdown.read().await);

    // Call shutdown
    scheduler.shutdown_and_cleanup().await;

    // Should be shut down
    assert!(*scheduler.shutdown.read().await);
}

#[tokio::test]
async fn test_scheduler_health_check_task_spawn() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (scheduler, _handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Start health check task
    let health_task = scheduler.start_health_check_task();

    // Task should be running
    assert!(!health_task.is_finished());

    // Signal shutdown
    {
        let mut shutdown = scheduler.shutdown.write().await;
        *shutdown = true;
    }

    // Wait for task to complete
    tokio::time::sleep(Duration::from_millis(150)).await;
    health_task.abort(); // Force cleanup if still running
}

#[tokio::test]
async fn test_scheduler_message_handling_integration() {
    let config = create_test_config();
    let snmp_config = SnmpClientConfig::default();

    let (mut scheduler, handle) = PollingScheduler::new_test_mode(config, snmp_config);

    // Create a task to add
    let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
    let node_id = Uuid::new_v4();
    let oids = vec!["1.3.6.1.2.1.1.1.0".to_string()];
    let interval = Duration::from_secs(300);
    let session_config = SessionConfig::default();

    let task = PollingTask::new(target, node_id, oids, interval, session_config);
    let task_id = task.id;

    // Send add task message
    handle.add_task(task).expect("Should send add task message");

    // Run single iteration to process message
    let should_shutdown = scheduler.run_single_iteration().await;
    assert!(!should_shutdown);

    // Task should be added
    assert_eq!(scheduler.task_count().await, 1);

    // Send remove task message
    handle
        .remove_task(task_id)
        .expect("Should send remove task message");

    // Run single iteration to process message
    let should_shutdown = scheduler.run_single_iteration().await;
    assert!(!should_shutdown);

    // Task should be removed
    assert_eq!(scheduler.task_count().await, 0);
}
