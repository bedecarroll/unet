//! SNMP client with connection pooling

use super::config::{SessionConfig, SnmpClientConfig};
use super::session::SnmpSession;
use super::values::SnmpValue;
use super::{SnmpError, SnmpResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tracing::info;

/// SNMP client with connection pooling and session management
#[derive(Debug)]
pub struct SnmpClient {
    /// Maximum number of concurrent connections
    max_connections: usize,
    /// Connection semaphore for limiting concurrent operations
    connection_semaphore: Semaphore,
    /// Active sessions keyed by target address
    sessions: RwLock<HashMap<SocketAddr, SnmpSession>>,
    /// Default session configuration
    default_config: SessionConfig,
}

impl SnmpClient {
    /// Create new SNMP client with configuration
    #[must_use]
    pub fn new(config: SnmpClientConfig) -> Self {
        Self {
            max_connections: config.max_connections,
            connection_semaphore: Semaphore::new(config.max_connections),
            sessions: RwLock::new(HashMap::new()),
            default_config: config.default_session,
        }
    }

    /// Get or create session for target address
    async fn get_session(
        &self,
        address: SocketAddr,
        config: Option<SessionConfig>,
    ) -> SnmpResult<SnmpSession> {
        // Check if session already exists
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(&address) {
                return Ok(session.clone());
            }
        }

        // Create new session
        let mut sessions = self.sessions.write().await;

        // Double-check after acquiring write lock
        if let Some(session) = sessions.get(&address) {
            return Ok(session.clone());
        }

        // Check connection pool limit
        if sessions.len() >= self.max_connections {
            return Err(SnmpError::PoolExhausted {
                max_connections: self.max_connections,
            });
        }

        let session_config = config.unwrap_or_else(|| {
            let mut config = self.default_config.clone();
            config.address = address;
            config
        });

        let session = SnmpSession::new(session_config);
        sessions.insert(address, session.clone());

        info!(
            target = %address,
            session_count = sessions.len(),
            "Created new SNMP session"
        );

        Ok(session)
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

        let session = self.get_session(address, config).await?;
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

        let session = self.get_session(address, config).await?;
        session.get_next(start_oid)
    }

    /// Get statistics about the client
    pub async fn stats(&self) -> SnmpClientStats {
        let sessions = self.sessions.read().await;
        SnmpClientStats {
            active_sessions: sessions.len(),
            max_connections: self.max_connections,
            available_permits: self.connection_semaphore.available_permits(),
        }
    }

    /// Clean up inactive sessions
    pub async fn cleanup_sessions(&self, max_age: Duration) {
        let mut sessions_to_remove = Vec::new();

        {
            let sessions = self.sessions.read().await;
            for (address, session) in sessions.iter() {
                if !session.is_healthy(max_age).await {
                    sessions_to_remove.push(*address);
                }
            }
        }

        if !sessions_to_remove.is_empty() {
            {
                let mut sessions = self.sessions.write().await;
                for address in &sessions_to_remove {
                    sessions.remove(address);
                }
            }
            for address in &sessions_to_remove {
                info!(target = %address, "Removed inactive SNMP session");
            }
        }
    }
}

/// SNMP client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpClientStats {
    /// Number of active sessions
    pub active_sessions: usize,
    /// Maximum allowed connections
    pub max_connections: usize,
    /// Available connection permits
    pub available_permits: usize,
}
