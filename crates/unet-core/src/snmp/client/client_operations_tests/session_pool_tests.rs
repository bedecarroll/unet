use super::fixtures::{create_test_address, create_test_session_config};
use super::super::super::session_management::SessionManager;
use super::super::ClientOperations;
use crate::snmp::SnmpError;

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
