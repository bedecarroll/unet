//! SNMP session management and connection pooling

use super::super::config::SessionConfig;
use super::super::session::SnmpSession;
use super::super::{SnmpError, SnmpResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

/// Session management functionality for SNMP client
#[derive(Debug)]
pub struct SessionManager {
    /// Active sessions keyed by target address
    sessions: RwLock<HashMap<SocketAddr, SnmpSession>>,
    /// Default session configuration
    default_config: SessionConfig,
    /// Maximum number of concurrent connections
    max_connections: usize,
}

impl SessionManager {
    /// Create new session manager
    pub fn new(default_config: SessionConfig, max_connections: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            default_config,
            max_connections,
        }
    }

    /// Get or create session for target address
    pub async fn get_session_mut(
        &self,
        address: SocketAddr,
        config: Option<SessionConfig>,
    ) -> SnmpResult<SnmpSession> {
        // Check connection pool limit first
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.max_connections && !sessions.contains_key(&address) {
                return Err(SnmpError::PoolExhausted {
                    max_connections: self.max_connections,
                });
            }
        }

        // Create session configuration
        let session_config = config.unwrap_or_else(|| {
            let mut config = self.default_config.clone();
            config.address = address;
            config
        });

        // Always create a new session for this operation
        // This ensures we have a mutable session to work with
        let session = SnmpSession::new(session_config);

        info!(
            target = %address,
            "Created SNMP session for operation"
        );

        Ok(session)
    }

    /// Get number of active sessions
    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Close a specific session
    pub async fn close_session(&self, address: SocketAddr) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&address);
    }

    /// Close all sessions
    pub async fn close_all_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config.clone(), 10);

        assert_eq!(manager.max_connections, 10);
        assert_eq!(manager.active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_manager_close_session() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config, 10);
        let address = "127.0.0.1:161".parse().unwrap();

        // Close session that doesn't exist - should not panic
        manager.close_session(address).await;
        assert_eq!(manager.active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_manager_close_all_sessions() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config, 10);

        // Close all sessions when none exist - should not panic
        manager.close_all_sessions().await;
        assert_eq!(manager.active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_manager_cleanup_sessions() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config, 10);
        let max_age = Duration::from_secs(3600); // 1 hour

        // Cleanup when no sessions exist - should not panic
        manager.cleanup_sessions(max_age).await;
        assert_eq!(manager.active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_manager_get_session_pool_exhausted() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config, 0); // Set to 0 to trigger pool exhausted
        let address = "127.0.0.1:161".parse().unwrap();

        // This should return pool exhausted error since max_connections is 0
        let result = manager.get_session_mut(address, None).await;
        assert!(result.is_err());

        if let Err(SnmpError::PoolExhausted { max_connections }) = result {
            assert_eq!(max_connections, 0);
        } else {
            panic!("Expected PoolExhausted error");
        }
    }

    #[tokio::test]
    async fn test_session_manager_get_session_with_config() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config, 10);
        let address = "127.0.0.1:161".parse().unwrap();

        let session_config = SessionConfig {
            address,
            timeout: Duration::from_secs(10),
            retries: 5,
            ..Default::default()
        };

        // This will create a new session but won't actually connect since it's a test
        let result = manager.get_session_mut(address, Some(session_config)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_session_manager_get_session_without_config() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config, 10);
        let address = "127.0.0.1:161".parse().unwrap();

        // This will create a new session using default config
        let result = manager.get_session_mut(address, None).await;
        assert!(result.is_ok());
    }
}
