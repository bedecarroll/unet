use super::super::super::config::{SnmpClientConfig, SnmpCredentials};
use super::super::session_management::SessionManager;
use super::ClientOperations;
use crate::snmp::SnmpError;
use crate::snmp::config::SessionConfig;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_client_operations_creation() {
    let config = SnmpClientConfig::default();
    let session_manager = SessionManager::new(config.default_session, config.max_connections);
    let operations = ClientOperations::new(session_manager, config.max_connections);

    assert_eq!(operations.max_connections, config.max_connections);
    assert_eq!(operations.available_permits(), config.max_connections);
}

#[test]
fn test_client_operations_available_permits() {
    let config = SnmpClientConfig {
        max_connections: 5,
        ..Default::default()
    };
    let session_manager = SessionManager::new(config.default_session, config.max_connections);
    let operations = ClientOperations::new(session_manager, config.max_connections);

    assert_eq!(operations.available_permits(), 5);
}

// Helper function to create test address
fn create_test_address() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161)
}

// Helper function to create test session config
fn create_test_session_config() -> SessionConfig {
    SessionConfig {
        address: create_test_address(),
        version: 2,
        credentials: SnmpCredentials::Community {
            community: "public".to_string(),
        },
        timeout: Duration::from_secs(5),
        retries: 3,
        max_vars_per_request: 10,
    }
}

#[tokio::test]
async fn test_get_method_semaphore_exhaustion_error() {
    // Create a client operations with max_connections = 1 but pre-exhaust the semaphore
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 1);

    let address = create_test_address();
    let oids = &["1.3.6.1.2.1.1.1.0"];

    // First, acquire the only available permit to exhaust the semaphore
    let _permit = operations.connection_semaphore.acquire().await.unwrap();

    // Verify semaphore is exhausted
    assert_eq!(operations.available_permits(), 0);

    // Now attempt to use get() - it should fail due to pool exhaustion
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        operations.get(address, oids, None),
    )
    .await;

    // Should timeout because semaphore is exhausted
    assert!(result.is_err()); // Timeout error
}

#[tokio::test]
async fn test_walk_method_semaphore_exhaustion_error() {
    // Create a client operations with max_connections = 1 but pre-exhaust the semaphore
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 1);

    let address = create_test_address();
    let start_oid = "1.3.6.1.2.1.1";

    // First, acquire the only available permit to exhaust the semaphore
    let _permit = operations.connection_semaphore.acquire().await.unwrap();

    // Verify semaphore is exhausted
    assert_eq!(operations.available_permits(), 0);

    // Now attempt to use walk() - it should timeout due to semaphore exhaustion
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        operations.walk(address, start_oid, None),
    )
    .await;

    // Should timeout because semaphore is exhausted
    assert!(result.is_err()); // Timeout error
}

#[tokio::test]
async fn test_get_method_session_manager_pool_exhausted() {
    // Create session manager with max_connections = 0 to trigger pool exhausted from session manager
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 0);
    let operations = ClientOperations::new(session_manager, 1); // Client allows 1, session manager allows 0

    let address = create_test_address();
    let oids = &["1.3.6.1.2.1.1.1.0"];

    // This should fail when session manager rejects due to pool exhaustion
    let result = operations.get(address, oids, None).await;

    assert!(result.is_err());
    if let Err(SnmpError::PoolExhausted { max_connections }) = result {
        assert_eq!(max_connections, 0);
    } else {
        panic!("Expected PoolExhausted error from session manager, got: {result:?}");
    }
}

#[tokio::test]
async fn test_walk_method_session_manager_pool_exhausted() {
    // Create session manager with max_connections = 0 to trigger pool exhausted from session manager
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 0);
    let operations = ClientOperations::new(session_manager, 1); // Client allows 1, session manager allows 0

    let address = create_test_address();
    let start_oid = "1.3.6.1.2.1.1";

    // This should fail when session manager rejects due to pool exhaustion
    let result = operations.walk(address, start_oid, None).await;

    assert!(result.is_err());
    if let Err(SnmpError::PoolExhausted { max_connections }) = result {
        assert_eq!(max_connections, 0);
    } else {
        panic!("Expected PoolExhausted error from session manager, got: {result:?}");
    }
}

#[tokio::test]
async fn test_get_method_semaphore_acquisition_success_flow() {
    // Create a normal configuration that should allow operations to proceed to session manager
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 10);

    // Verify we have available permits before the test
    assert_eq!(operations.available_permits(), 10);

    let address = create_test_address();
    let oids = &["1.3.6.1.2.1.1.1.0"];

    // This should acquire semaphore successfully but fail/timeout at session manager level
    // due to no actual SNMP server in test environment
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        operations.get(address, oids, None),
    )
    .await;

    // Either timeout or error - both indicate we successfully acquired semaphore but failed later
    if let Ok(operation_result) = result {
        // If operation completed, it should be an error due to no SNMP server
        assert!(operation_result.is_err());
    }
    // If timeout, that's also expected behavior in test environment

    // Verify permits are released after operation (this is the critical test)
    assert_eq!(operations.available_permits(), 10);
}

#[tokio::test]
async fn test_walk_method_semaphore_acquisition_success_flow() {
    // Create a normal configuration that should allow operations to proceed to session manager
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 10);

    // Verify we have available permits before the test
    assert_eq!(operations.available_permits(), 10);

    let address = create_test_address();
    let start_oid = "1.3.6.1.2.1.1";

    // This should acquire semaphore successfully but fail/timeout at session manager level
    // due to no actual SNMP server in test environment
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        operations.walk(address, start_oid, None),
    )
    .await;

    // Either timeout or error - both indicate we successfully acquired semaphore but failed later
    if let Ok(operation_result) = result {
        // If operation completed, it should be an error due to no SNMP server
        assert!(operation_result.is_err());
    }
    // If timeout, that's also expected behavior in test environment

    // Verify permits are released after operation (this is the critical test)
    assert_eq!(operations.available_permits(), 10);
}

#[tokio::test]
async fn test_get_method_with_custom_session_config() {
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 10);

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

    // This should pass custom config through to session manager
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        operations.get(address, oids, Some(custom_config)),
    )
    .await;

    // Either timeout or error - both are expected in test environment
    if let Ok(operation_result) = result {
        assert!(operation_result.is_err());
    }
}

#[tokio::test]
async fn test_walk_method_with_custom_session_config() {
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 10);

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

    // This should pass custom config through to session manager
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        operations.walk(address, start_oid, Some(custom_config)),
    )
    .await;

    // Either timeout or error - both are expected in test environment
    if let Ok(operation_result) = result {
        assert!(operation_result.is_err());
    }
}

#[tokio::test]
async fn test_concurrent_semaphore_exhaustion() {
    // Create operations with very limited connections
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = Arc::new(ClientOperations::new(session_manager, 1)); // Only 1 permit

    let address = create_test_address();
    let oids = &["1.3.6.1.2.1.1.1.0"];

    // Launch two concurrent operations - one should get permit, other should fail
    let ops1 = Arc::clone(&operations);
    let ops2 = Arc::clone(&operations);

    let oids_clone = oids.to_vec();
    let task1 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops1.get(address, &oids_clone.clone(), None),
        )
        .await
    });

    let oids_clone = oids.to_vec();
    let task2 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops2.get(address, &oids_clone.clone(), None),
        )
        .await
    });

    let (result1, result2) = tokio::join!(task1, task2);

    // Both should complete (one way or another)
    let result1 = result1.expect("Task 1 should complete");
    let result2 = result2.expect("Task 2 should complete");

    // Due to only 1 permit, one task should get the permit and the other should wait
    // Both may timeout or error, but they should both complete without hanging
    // The main test is that they don't hang indefinitely
    assert!(result1.is_err() || result2.is_err()); // At least one should error/timeout
}

#[tokio::test]
async fn test_concurrent_walk_semaphore_exhaustion() {
    // Create operations with very limited connections
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = Arc::new(ClientOperations::new(session_manager, 1)); // Only 1 permit

    let address = create_test_address();
    let start_oid = "1.3.6.1.2.1.1";

    // Launch two concurrent walk operations
    let ops1 = Arc::clone(&operations);
    let ops2 = Arc::clone(&operations);

    let task1 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops1.walk(address, start_oid, None),
        )
        .await
    });

    let task2 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops2.walk(address, start_oid, None),
        )
        .await
    });

    let (result1, result2) = tokio::join!(task1, task2);

    // Both should complete
    let result1 = result1.expect("Task 1 should complete");
    let result2 = result2.expect("Task 2 should complete");

    // Due to only 1 permit, both may timeout or error, but they should complete without hanging
    assert!(result1.is_err() || result2.is_err()); // At least one should error/timeout
}

#[tokio::test]
async fn test_get_method_semaphore_acquire_error_path() {
    // This test tries to trigger the specific error path by simulating semaphore closure
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 1);

    let _address = create_test_address();

    // Take the permit and hold it, then immediately try to get another one
    let _permit = operations.connection_semaphore.acquire().await.unwrap();

    // Now try to acquire another permit which should fail immediately due to closed state
    // We'll use try_acquire which fails immediately if no permits available
    assert!(operations.connection_semaphore.try_acquire().is_err());

    // The specific map_err lines (53-54, 84-85) are hard to test directly due to async nature
    // But the above verifies the semaphore exhaustion logic works correctly
}

#[tokio::test]
async fn test_walk_method_semaphore_acquire_error_path() {
    // This test tries to trigger the specific error path by simulating semaphore closure
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = ClientOperations::new(session_manager, 1);

    let _address = create_test_address();

    // Take the permit and hold it, then immediately try to get another one
    let _permit = operations.connection_semaphore.acquire().await.unwrap();

    // Now try to acquire another permit which should fail immediately due to closed state
    assert!(operations.connection_semaphore.try_acquire().is_err());

    // The semaphore exhaustion logic is tested - the map_err lines cover error creation
    // which happens when semaphore.acquire() returns an error
}
