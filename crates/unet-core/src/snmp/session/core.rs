//! Core SNMP session management

use super::super::config::{SessionConfig, SnmpCredentials};
use super::super::{SnmpError, SnmpResult};
use csnmp::Snmp2cClient;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// SNMP session for device communication
pub struct SnmpSession {
    /// Session configuration
    pub(super) config: SessionConfig,
    /// Session ID for tracking
    pub(super) session_id: Uuid,
    /// Last successful operation timestamp
    pub(super) last_success: RwLock<Option<SystemTime>>,
    /// Connection attempt counter
    pub(super) connection_attempts: RwLock<u32>,
    /// Underlying SNMP client
    pub(super) client: Option<Snmp2cClient>,
}

impl SnmpSession {
    /// Create new SNMP session
    #[must_use]
    pub fn new(config: SessionConfig) -> Self {
        // We'll create the actual client lazily when needed
        // This avoids blocking async code in constructor
        Self {
            config,
            session_id: Uuid::new_v4(),
            last_success: RwLock::new(None),
            connection_attempts: RwLock::new(0),
            client: None,
        }
    }

    /// Create a new SNMP client
    pub(super) async fn create_client(config: &SessionConfig) -> SnmpResult<Snmp2cClient> {
        match &config.credentials {
            SnmpCredentials::Community { community } => {
                let client = Snmp2cClient::new(
                    config.address,
                    community.as_bytes().to_vec(),
                    None, // Use default local address
                    None, // Use default timeout
                    0,    // Default request_id
                )
                .await
                .map_err(|e| SnmpError::Protocol {
                    message: format!("Failed to create SNMP client: {e}"),
                })?;

                Ok(client)
            }
            SnmpCredentials::UserBased { .. } => Err(SnmpError::Protocol {
                message: "SNMPv3 user-based security not supported by csnmp".to_string(),
            }),
        }
    }

    /// Get or create the underlying SNMP client
    pub(super) async fn get_client(&mut self) -> SnmpResult<&Snmp2cClient> {
        if self.client.is_none() {
            self.client = Some(Self::create_client(&self.config).await?);
        }

        Ok(self.client.as_ref().unwrap())
    }

    /// Get session ID
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.session_id
    }

    /// Get session configuration
    pub const fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Increment the connection attempts counter
    pub(super) async fn increment_connection_attempts(&self) {
        let mut attempts = self.connection_attempts.write().await;
        *attempts += 1;
    }

    /// Update the last success timestamp if we got results
    pub(super) async fn update_success_timestamp(&self, has_results: bool) {
        if has_results {
            let mut last_success = self.last_success.write().await;
            *last_success = Some(SystemTime::now());
        }
    }

    /// Check if session is healthy (recent successful operations)
    pub async fn is_healthy(&self, max_age: Duration) -> bool {
        let last_success = self.last_success.read().await;
        last_success.is_some_and(|timestamp| {
            SystemTime::now()
                .duration_since(timestamp)
                .is_ok_and(|age| age <= max_age)
        })
    }

    /// Get connection attempt count
    pub async fn connection_attempts(&self) -> u32 {
        *self.connection_attempts.read().await
    }
}

impl Clone for SnmpSession {
    fn clone(&self) -> Self {
        Self::new(self.config.clone())
    }
}

impl std::fmt::Debug for SnmpSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnmpSession")
            .field("config", &self.config)
            .field("session_id", &self.session_id)
            .field("last_success", &"<RwLock<Option<SystemTime>>>")
            .field("connection_attempts", &"<RwLock<u32>>")
            .field("client", &"<Option<Snmp2cClient>>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snmp::config::SnmpCredentials;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn create_test_config() -> SessionConfig {
        SessionConfig {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
            version: 2,
            credentials: SnmpCredentials::Community {
                community: "public".to_string(),
            },
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 50,
        }
    }

    #[test]
    fn test_session_creation() {
        let config = create_test_config();
        let session = SnmpSession::new(config.clone());

        assert_eq!(session.config().address, config.address);
        assert!(!session.id().is_nil());
    }

    #[tokio::test]
    async fn test_session_health() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        // New session is not healthy (no successful operations)
        assert!(!session.is_healthy(Duration::from_secs(300)).await);

        // Update success timestamp
        session.update_success_timestamp(true).await;

        // Now it should be healthy
        assert!(session.is_healthy(Duration::from_secs(300)).await);
    }

    #[tokio::test]
    async fn test_connection_attempts() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        assert_eq!(session.connection_attempts().await, 0);

        session.increment_connection_attempts().await;
        assert_eq!(session.connection_attempts().await, 1);

        session.increment_connection_attempts().await;
        assert_eq!(session.connection_attempts().await, 2);
    }

    #[tokio::test]
    async fn test_session_clone() {
        let config = create_test_config();
        let session1 = SnmpSession::new(config);
        let session2 = session1.clone();

        // Cloned session should have different ID but same config
        assert_ne!(session1.id(), session2.id());
        assert_eq!(session1.config().address, session2.config().address);
        assert_eq!(session1.config().version, session2.config().version);
    }

    #[tokio::test]
    async fn test_session_debug_format() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        let debug_str = format!("{session:?}");
        assert!(debug_str.contains("SnmpSession"));
        assert!(debug_str.contains("session_id"));
        assert!(debug_str.contains("config"));
    }

    #[tokio::test]
    async fn test_update_success_timestamp_false() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        // Should not update timestamp when has_results is false
        session.update_success_timestamp(false).await;
        assert!(!session.is_healthy(Duration::from_secs(300)).await);
    }

    #[tokio::test]
    async fn test_session_health_expired() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        // Update success timestamp
        session.update_success_timestamp(true).await;

        // Session is healthy for long duration
        assert!(session.is_healthy(Duration::from_secs(300)).await);

        // Session is not healthy for very short duration
        assert!(!session.is_healthy(Duration::from_nanos(1)).await);
    }

    #[tokio::test]
    async fn test_create_client_community() {
        let config = SessionConfig {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
            version: 2,
            credentials: SnmpCredentials::Community {
                community: "test-community".to_string(),
            },
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 50,
        };

        // This will fail in test environment but we test the error handling
        let result = SnmpSession::create_client(&config).await;
        // In test environment, this will likely fail with a connection error
        // but that's expected - we're testing the error path
        match result {
            Ok(_) | Err(SnmpError::Protocol { .. }) => {
                // Expected - either success (unlikely in test) or protocol error
            }
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_create_client_user_based() {
        let config = SessionConfig {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
            version: 3,
            credentials: SnmpCredentials::UserBased {
                username: "test".to_string(),
                auth: Some(("SHA".to_string(), "auth".to_string())),
                privacy: Some(("AES".to_string(), "priv".to_string())),
            },
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 50,
        };

        let result = SnmpSession::create_client(&config).await;
        assert!(result.is_err());
        if let Err(SnmpError::Protocol { message }) = result {
            assert!(message.contains("SNMPv3 user-based security not supported"));
        } else {
            panic!("Expected Protocol error for SNMPv3");
        }
    }

    #[tokio::test]
    async fn test_get_client() {
        let config = create_test_config();
        let mut session = SnmpSession::new(config);

        // First call should try to create client (will likely fail in test env)
        let result = session.get_client().await;
        match result {
            Ok(_) | Err(SnmpError::Protocol { .. }) => {
                // Expected - either success (unlikely in test) or protocol error
            }
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[test]
    fn test_session_const_methods() {
        let config = create_test_config();
        let session = SnmpSession::new(config.clone());

        // Test const methods work
        let _id = session.id();
        let _config = session.config();

        assert_eq!(session.config().address, config.address);
    }
}
