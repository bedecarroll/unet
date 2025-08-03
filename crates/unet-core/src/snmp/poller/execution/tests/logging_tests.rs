//! Tests for logging functionality

use super::super::*;
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
fn test_log_poll_completion_success() {
    let task = create_test_task();
    let duration = Duration::from_millis(150);

    // This should not panic and should log success
    log_poll_completion(&task, true, duration);
}

#[test]
fn test_log_poll_completion_failure() {
    let mut task = create_test_task();
    task.consecutive_failures = 3;
    task.last_error = Some("SNMP timeout".to_string());
    let duration = Duration::from_secs(5);

    // This should not panic and should log failure details
    log_poll_completion(&task, false, duration);
}

#[test]
fn test_log_poll_completion_failure_no_error_message() {
    let mut task = create_test_task();
    task.consecutive_failures = 1;
    task.last_error = None; // No error message
    let duration = Duration::from_millis(200);

    // Should handle case where last_error is None (line 174)
    log_poll_completion(&task, false, duration);
}
