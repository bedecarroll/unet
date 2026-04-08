use super::super::super::session_management::SessionManager;
use super::super::ClientOperations;
use crate::snmp::config::SnmpClientConfig;

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
