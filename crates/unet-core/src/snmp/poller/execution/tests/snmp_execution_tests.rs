//! Tests for SNMP execution functionality

use super::super::*;
use crate::snmp::SnmpClientConfig;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
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

#[tokio::test]
async fn test_execute_snmp_poll_success() {
    use crate::snmp::testing::MockSnmpClient;

    let task = create_test_task();
    let mock_client = MockSnmpClient::success();
    let timeout = Duration::from_millis(100); // Much shorter timeout since we're using a mock

    // Use the mock client function for testing
    let result = execute_snmp_poll_with_mock(&task, &mock_client, timeout).await;

    // Should get a successful result
    assert!(result.is_ok());
    let inner_result = result.unwrap();
    assert!(inner_result.is_ok());
    let values = inner_result.unwrap();

    // Should have values for our test OIDs
    assert!(!values.is_empty());
    assert!(values.contains_key("1.3.6.1.2.1.1.1.0") || values.contains_key("1.3.6.1.2.1.1.3.0"));
}

#[tokio::test]
async fn test_execute_snmp_poll_timeout() {
    use crate::snmp::testing::DelayedMockSnmpClient;

    let task = create_test_task();
    // Create a mock that delays for 100ms with a 10ms timeout
    let mock_client = DelayedMockSnmpClient::new(Duration::from_millis(100), true);
    let timeout = Duration::from_millis(10); // Shorter than the mock delay

    let result = execute_snmp_poll_with_mock(&task, &mock_client, timeout).await;

    // Should get a timeout error because mock takes longer than timeout
    assert!(result.is_err());
}

#[tokio::test]
async fn test_poll_task_with_mock_client() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let task = create_test_task();
    let snmp_config = SnmpClientConfig::default();
    let snmp_client = Arc::new(crate::snmp::SnmpClient::new(snmp_config));
    let timeout = Duration::from_millis(100); // Short timeout to ensure it fails quickly

    // This will test the full poll_task function including error handling
    poll_task(task.clone(), snmp_client, tx, timeout).await;

    // Should receive a polling result
    let result = rx.recv().await;
    assert!(result.is_some());
    let polling_result = result.unwrap();
    assert_eq!(polling_result.task_id, task.id);
    assert_eq!(polling_result.node_id, task.node_id);
    assert_eq!(polling_result.target, task.target);
    // Should be false due to connection failure or timeout
    assert!(!polling_result.success);
    assert!(polling_result.error.is_some());
}
