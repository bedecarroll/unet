//! Tests for polling result creation and handling

use super::super::*;
use crate::snmp::SnmpValue;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
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

#[test]
fn test_create_polling_result() {
    let task = create_test_task();
    let poll_start = SystemTime::now();
    let success = true;
    let mut values = HashMap::new();
    values.insert("test_oid".to_string(), SnmpValue::Integer(42));
    let error = None;
    let duration = Duration::from_millis(150);

    let result = create_polling_result(&task, poll_start, success, values.clone(), error, duration);

    assert_eq!(result.task_id, task.id);
    assert_eq!(result.node_id, task.node_id);
    assert_eq!(result.target, task.target);
    assert_eq!(result.timestamp, poll_start);
    assert_eq!(result.success, success);
    assert_eq!(result.values, values);
    assert_eq!(result.error, None);
    assert_eq!(result.duration, duration);
}

#[test]
fn test_create_polling_result_with_error() {
    let task = create_test_task();
    let poll_start = SystemTime::now();
    let success = false;
    let values = HashMap::new();
    let error = Some("Connection timeout".to_string());
    let duration = Duration::from_secs(5);

    let result = create_polling_result(&task, poll_start, success, values, error.clone(), duration);

    assert!(!result.success);
    assert!(result.values.is_empty());
    assert_eq!(result.error, error);
}

#[test]
fn test_create_polling_result_with_values() {
    let task = create_test_task();
    let poll_start = SystemTime::now();
    let mut values = HashMap::new();
    values.insert(
        "1.3.6.1.2.1.1.1.0".to_string(),
        SnmpValue::String("Router".to_string()),
    );
    values.insert("1.3.6.1.2.1.1.3.0".to_string(), SnmpValue::Integer(12345));
    values.insert("1.3.6.1.2.1.2.1.0".to_string(), SnmpValue::Integer(42));

    let result = create_polling_result(
        &task,
        poll_start,
        true,
        values.clone(),
        None,
        Duration::from_millis(200),
    );

    assert!(result.success);
    assert_eq!(result.values.len(), 3);
    assert_eq!(
        result.values.get("1.3.6.1.2.1.1.1.0"),
        Some(&SnmpValue::String("Router".to_string()))
    );
    assert_eq!(
        result.values.get("1.3.6.1.2.1.1.3.0"),
        Some(&SnmpValue::Integer(12345))
    );
    assert_eq!(
        result.values.get("1.3.6.1.2.1.2.1.0"),
        Some(&SnmpValue::Integer(42))
    );
}

#[tokio::test]
async fn test_send_result_success() {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let task = create_test_task();
    let result = create_polling_result(
        &task,
        SystemTime::now(),
        true,
        HashMap::new(),
        None,
        Duration::from_millis(100),
    );

    send_result(result.clone(), &tx);

    let received = rx.recv().await.expect("Should receive result");
    assert_eq!(received.task_id, result.task_id);
    assert_eq!(received.success, result.success);
}

#[tokio::test]
async fn test_send_result_channel_closed() {
    let (tx, rx) = mpsc::unbounded_channel();
    drop(rx); // Close the receiver

    let task = create_test_task();
    let result = create_polling_result(
        &task,
        SystemTime::now(),
        true,
        HashMap::new(),
        None,
        Duration::from_millis(100),
    );

    // This should not panic, just log an error
    send_result(result, &tx);
}

#[tokio::test]
async fn test_send_result_multiple_results() {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let task1 = create_test_task();
    let task2 = create_test_task();

    let result1 = create_polling_result(
        &task1,
        SystemTime::now(),
        true,
        HashMap::new(),
        None,
        Duration::from_millis(100),
    );

    let result2 = create_polling_result(
        &task2,
        SystemTime::now(),
        false,
        HashMap::new(),
        Some("Error".to_string()),
        Duration::from_millis(200),
    );

    // Send multiple results
    send_result(result1.clone(), &tx);
    send_result(result2.clone(), &tx);

    // Receive and verify both results
    let received1 = rx.recv().await.expect("Should receive first result");
    assert_eq!(received1.task_id, result1.task_id);
    assert!(received1.success);

    let received2 = rx.recv().await.expect("Should receive second result");
    assert_eq!(received2.task_id, result2.task_id);
    assert!(!received2.success);
    assert_eq!(received2.error, Some("Error".to_string()));
}
