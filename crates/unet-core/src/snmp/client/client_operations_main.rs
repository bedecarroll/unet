//! Core SNMP client operations (get, walk)

use super::super::config::SessionConfig;
use super::super::values::SnmpValue;
use super::super::{SnmpError, SnmpResult};
use super::session_management::SessionManager;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::Semaphore;

/// SNMP client operations handler
#[derive(Debug)]
pub struct ClientOperations {
    /// Connection semaphore for limiting concurrent operations
    connection_semaphore: Semaphore,
    /// Session manager
    session_manager: SessionManager,
    /// Maximum number of concurrent connections
    max_connections: usize,
}

impl ClientOperations {
    /// Create new client operations handler
    pub fn new(session_manager: SessionManager, max_connections: usize) -> Self {
        Self {
            connection_semaphore: Semaphore::new(max_connections),
            session_manager,
            max_connections,
        }
    }

    /// Perform SNMP GET operation on target
    ///
    /// # Errors
    ///
    /// Returns `SnmpError` if:
    /// - Connection to target fails
    /// - SNMP request times out
    /// - Invalid OIDs are provided
    /// - Authentication fails
    pub async fn get(
        &self,
        address: SocketAddr,
        oids: &[&str],
        config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        // Acquire connection permit - must be held for duration of function to maintain semaphore limit
        let _permit =
            self.connection_semaphore
                .acquire()
                .await
                .map_err(|_| SnmpError::PoolExhausted {
                    max_connections: self.max_connections,
                })?;

        let mut session = self
            .session_manager
            .get_session_mut(address, config)
            .await?;
        session.get(oids).await
    }

    /// Perform SNMP table walk on target
    ///
    /// # Errors
    ///
    /// Returns `SnmpError` if:
    /// - Connection to target fails
    /// - SNMP request times out
    /// - Invalid start OID is provided
    /// - Authentication fails
    pub async fn walk(
        &self,
        address: SocketAddr,
        start_oid: &str,
        config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        // Acquire connection permit - must be held for duration of function to maintain semaphore limit
        let _permit =
            self.connection_semaphore
                .acquire()
                .await
                .map_err(|_| SnmpError::PoolExhausted {
                    max_connections: self.max_connections,
                })?;

        let mut session = self
            .session_manager
            .get_session_mut(address, config)
            .await?;
        session.get_next(start_oid).await
    }

    /// Get available permits from the connection semaphore
    pub fn available_permits(&self) -> usize {
        self.connection_semaphore.available_permits()
    }
}