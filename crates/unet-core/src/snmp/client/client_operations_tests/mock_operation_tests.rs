use super::fixtures::{
    MockClientOperations, MockSessionManager, create_test_address, create_test_session_config,
};
use crate::snmp::config::{SessionConfig, SnmpCredentials};
use std::time::Duration;

#[tokio::test]
async fn test_get_method_semaphore_acquisition_success_flow() {
    // Create a mock configuration that fails fast without network calls
    let config = create_test_session_config();
    let session_manager = MockSessionManager::new(config.clone(), 10, true);
    let operations = MockClientOperations::new(session_manager, 10);

    // Verify we have available permits before the test
    assert_eq!(operations.available_permits(), 10);

    let address = create_test_address();
    let oids = &["1.3.6.1.2.1.1.1.0"];

    // This should acquire semaphore successfully but fail quickly at mock session manager level
    let result = tokio::time::timeout(
        Duration::from_millis(50),
        operations.get(address, oids, None),
    )
    .await;

    // Should complete quickly with an error from mock
    assert!(result.is_ok()); // No timeout
    let operation_result = result.unwrap();
    assert!(operation_result.is_err()); // Mock returns error

    // Verify permits are released after operation (this is the critical test)
    assert_eq!(operations.available_permits(), 10);
}

#[tokio::test]
async fn test_walk_method_semaphore_acquisition_success_flow() {
    // Create a mock configuration that fails fast without network calls
    let config = create_test_session_config();
    let session_manager = MockSessionManager::new(config.clone(), 10, true);
    let operations = MockClientOperations::new(session_manager, 10);

    // Verify we have available permits before the test
    assert_eq!(operations.available_permits(), 10);

    let address = create_test_address();
    let start_oid = "1.3.6.1.2.1.1";

    // This should acquire semaphore successfully but fail quickly at mock session manager level
    let result = tokio::time::timeout(
        Duration::from_millis(50),
        operations.walk(address, start_oid, None),
    )
    .await;

    // Should complete quickly with an error from mock
    assert!(result.is_ok()); // No timeout
    let operation_result = result.unwrap();
    assert!(operation_result.is_err()); // Mock returns error

    // Verify permits are released after operation (this is the critical test)
    assert_eq!(operations.available_permits(), 10);
}

#[tokio::test]
async fn test_get_method_with_custom_session_config() {
    let config = create_test_session_config();
    let session_manager = MockSessionManager::new(config.clone(), 10, true);
    let operations = MockClientOperations::new(session_manager, 10);

    let address = create_test_address();
    let oids = &["1.3.6.1.2.1.1.1.0"];

    // Create custom session config
    let custom_config = SessionConfig {
        address,
        version: 2,
        credentials: SnmpCredentials::Community {
            community: "test-community".to_string(),
        },
        timeout: Duration::from_secs(10),
        retries: 5,
        max_vars_per_request: 20,
    };

    // This should pass custom config through to mock session manager and fail quickly
    let result = tokio::time::timeout(
        Duration::from_millis(50),
        operations.get(address, oids, Some(custom_config)),
    )
    .await;

    // Should complete quickly with an error from mock
    assert!(result.is_ok()); // No timeout
    let operation_result = result.unwrap();
    assert!(operation_result.is_err()); // Mock returns error
}

#[tokio::test]
async fn test_walk_method_with_custom_session_config() {
    let config = create_test_session_config();
    let session_manager = MockSessionManager::new(config.clone(), 10, true);
    let operations = MockClientOperations::new(session_manager, 10);

    let address = create_test_address();
    let start_oid = "1.3.6.1.2.1.1";

    // Create custom session config
    let custom_config = SessionConfig {
        address,
        version: 2,
        credentials: SnmpCredentials::Community {
            community: "test-community".to_string(),
        },
        timeout: Duration::from_secs(10),
        retries: 5,
        max_vars_per_request: 20,
    };

    // This should pass custom config through to mock session manager and fail quickly
    let result = tokio::time::timeout(
        Duration::from_millis(50),
        operations.walk(address, start_oid, Some(custom_config)),
    )
    .await;

    // Should complete quickly with an error from mock
    assert!(result.is_ok()); // No timeout
    let operation_result = result.unwrap();
    assert!(operation_result.is_err()); // Mock returns error
}
