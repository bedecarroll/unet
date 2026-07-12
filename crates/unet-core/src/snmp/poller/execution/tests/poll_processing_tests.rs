//! Tests for poll result processing

use super::super::*;
use crate::snmp::{SnmpError, SnmpValue};
use std::collections::HashMap;
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

#[test]
fn test_process_poll_result_success() {
    let mut task = create_test_task();
    let timeout = Duration::from_secs(30);

    // Create successful SNMP result
    let mut values = HashMap::new();
    values.insert(
        "1.3.6.1.2.1.1.1.0".to_string(),
        SnmpValue::String("test".to_string()),
    );
    values.insert("1.3.6.1.2.1.1.3.0".to_string(), SnmpValue::Integer(12345));

    let snmp_result = Ok(Ok(values.clone()));
    let (success, returned_values, error) = process_poll_result(snmp_result, &mut task, timeout);

    assert!(success);
    assert!(error.is_none());
    assert_eq!(returned_values.len(), 2);

    // Should reset consecutive failures on success
    assert_eq!(task.consecutive_failures, 0);
}

#[test]
fn test_process_poll_result_snmp_error() {
    let mut task = create_test_task();
    let timeout = Duration::from_secs(30);

    // Create SNMP error
    let snmp_result = Ok(Err(SnmpError::Timeout {
        duration: Duration::from_secs(5),
    }));
    let (success, values, error) = process_poll_result(snmp_result, &mut task, timeout);

    assert!(!success);
    assert!(error.is_some());
    assert!(values.is_empty());

    // Should increment consecutive failures
    assert_eq!(task.consecutive_failures, 1);
}

#[tokio::test]
async fn test_process_poll_result_timeout_error() {
    let mut task = create_test_task();
    let timeout = Duration::from_secs(30);

    // Create timeout error by using tokio::time::timeout that actually times out
    let slow_future = tokio::time::sleep(Duration::from_secs(1));
    let snmp_result = tokio::time::timeout(Duration::from_millis(1), slow_future).await;
    // This creates a real Elapsed error
    let timeout_error_result = snmp_result.map(|()| Ok(HashMap::new()));

    let (success, values, error) = process_poll_result(timeout_error_result, &mut task, timeout);

    assert!(!success);
    assert!(error.is_some());
    assert!(values.is_empty());
    assert_eq!(task.consecutive_failures, 1);
}

#[test]
fn test_process_poll_result_consecutive_failures() {
    let mut task = create_test_task();
    task.consecutive_failures = 2; // Start with some failures
    let timeout = Duration::from_secs(30);

    // Create another error
    let snmp_result = Ok(Err(SnmpError::Timeout {
        duration: Duration::from_secs(5),
    }));
    let (success, _values, _error) = process_poll_result(snmp_result, &mut task, timeout);

    assert!(!success);
    assert_eq!(task.consecutive_failures, 3);
}

#[test]
fn test_process_poll_result_success_resets_failures() {
    let mut task = create_test_task();
    task.consecutive_failures = 5; // Start with some failures

    // Create successful result
    let mut values = HashMap::new();
    values.insert(
        "1.3.6.1.2.1.1.1.0".to_string(),
        SnmpValue::String("success".to_string()),
    );

    let snmp_result = Ok(Ok(values.clone()));
    let (success, returned_values, error) =
        process_poll_result(snmp_result, &mut task, Duration::from_secs(10));

    assert!(success);
    assert_eq!(returned_values, values);
    assert!(error.is_none());
    assert_eq!(task.consecutive_failures, 0); // Should be reset
}
