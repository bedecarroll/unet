use crate::snmp::SnmpError;
use crate::snmp::config::{SessionConfig, SnmpCredentials};
use crate::snmp::values::SnmpValue;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

// Helper function to create test address
pub(super) fn create_test_address() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161)
}

// Helper function to create test session config
pub(super) fn create_test_session_config() -> SessionConfig {
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

// Mock session manager that returns controlled errors for testing
pub(super) struct MockSessionManager {
    should_fail: bool,
}

impl MockSessionManager {
    pub(super) fn new(_config: SessionConfig, _max_connections: usize, should_fail: bool) -> Self {
        Self { should_fail }
    }

    // Mock get_session_mut that returns a quick error instead of doing network calls
    fn get_session_mut(
        &self,
        _address: SocketAddr,
        _config: Option<SessionConfig>,
    ) -> Result<MockSnmpSession, SnmpError> {
        if self.should_fail {
            Err(SnmpError::Network {
                message: "Mock connection failure".to_string(),
            })
        } else {
            Ok(MockSnmpSession::new())
        }
    }
}

// Mock SNMP session that fails fast for testing
struct MockSnmpSession;

impl MockSnmpSession {
    fn new() -> Self {
        Self
    }

    fn get(_oids: &[&str]) -> Result<HashMap<String, SnmpValue>, SnmpError> {
        Err(SnmpError::Network {
            message: "Mock SNMP operation failure".to_string(),
        })
    }

    fn get_next(_start_oid: &str) -> Result<HashMap<String, SnmpValue>, SnmpError> {
        Err(SnmpError::Network {
            message: "Mock SNMP operation failure".to_string(),
        })
    }
}

// Mock client operations for testing semaphore behavior without network calls
pub(super) struct MockClientOperations {
    connection_semaphore: tokio::sync::Semaphore,
    session_manager: MockSessionManager,
    max_connections: usize,
}

impl MockClientOperations {
    pub(super) fn new(session_manager: MockSessionManager, max_connections: usize) -> Self {
        Self {
            connection_semaphore: tokio::sync::Semaphore::new(max_connections),
            session_manager,
            max_connections,
        }
    }

    pub(super) fn available_permits(&self) -> usize {
        self.connection_semaphore.available_permits()
    }

    pub(super) async fn get(
        &self,
        address: SocketAddr,
        oids: &[&str],
        config: Option<SessionConfig>,
    ) -> Result<HashMap<String, SnmpValue>, SnmpError> {
        // Acquire connection permit - must be held for duration of function to maintain semaphore limit
        let _permit =
            self.connection_semaphore
                .acquire()
                .await
                .map_err(|_| SnmpError::PoolExhausted {
                    max_connections: self.max_connections,
                })?;

        let _session = self.session_manager.get_session_mut(address, config)?;
        MockSnmpSession::get(oids)
    }

    pub(super) async fn walk(
        &self,
        address: SocketAddr,
        start_oid: &str,
        config: Option<SessionConfig>,
    ) -> Result<HashMap<String, SnmpValue>, SnmpError> {
        // Acquire connection permit - must be held for duration of function to maintain semaphore limit
        let _permit =
            self.connection_semaphore
                .acquire()
                .await
                .map_err(|_| SnmpError::PoolExhausted {
                    max_connections: self.max_connections,
                })?;

        let _session = self.session_manager.get_session_mut(address, config)?;
        MockSnmpSession::get_next(start_oid)
    }
}
