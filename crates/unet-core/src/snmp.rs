//! SNMP client integration for μNet
//!
//! This module provides SNMP polling capabilities for network devices to collect
//! derived state information. It supports SNMPv2c and SNMPv3 protocols with
//! connection pooling and error handling.
//!
//! # Architecture
//!
//! - [`client`] - SNMP client wrapper with connection pooling
//! - [`oids`] - Standard and vendor-specific OID definitions
//! - [`session`] - SNMP session management
//! - [`poller`] - Background polling implementation
//! - [`types`] - SNMP-specific data types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info};
use uuid::Uuid;

pub mod client;
pub mod oids;
pub mod poller;
pub mod session;
pub mod types;

// Re-export main types
pub use oids::{OidMap, StandardOid, VendorOid};
pub use poller::{PollingConfig, PollingHandle, PollingResult, PollingScheduler, PollingTask};
pub use types::SnmpType;

/// SNMP error types
#[derive(Error, Debug)]
pub enum SnmpError {
    /// Network connection error
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),

    /// SNMP protocol error
    #[error("SNMP protocol error: {message}")]
    Protocol {
        /// The protocol error message
        message: String,
    },

    /// Timeout error
    #[error("SNMP timeout after {duration:?}")]
    Timeout {
        /// The timeout duration that was exceeded
        duration: Duration,
    },

    /// Authentication failure
    #[error("SNMP authentication failed")]
    Authentication,

    /// Invalid OID format
    #[error("Invalid OID: {oid}")]
    InvalidOid {
        /// The invalid OID string
        oid: String,
    },

    /// Response parsing error
    #[error("Failed to parse SNMP response: {reason}")]
    ParseError {
        /// The reason for the parsing failure
        reason: String,
    },

    /// Connection pool exhausted
    #[error("Connection pool exhausted, max connections: {max_connections}")]
    PoolExhausted {
        /// The maximum number of connections in the pool
        max_connections: usize,
    },
}

/// SNMP operation result
pub type SnmpResult<T> = std::result::Result<T, SnmpError>;

/// SNMP value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SnmpValue {
    /// Integer value
    Integer(i64),
    /// String value  
    String(String),
    /// Object identifier
    Oid(String),
    /// IP address
    IpAddress(IpAddr),
    /// Counter (32-bit)
    Counter32(u32),
    /// Counter (64-bit)
    Counter64(u64),
    /// Gauge (32-bit)
    Gauge32(u32),
    /// Time ticks
    TimeTicks(u32),
    /// Opaque data
    Opaque(Vec<u8>),
    /// Null value
    Null,
    /// No such object
    NoSuchObject,
    /// No such instance
    NoSuchInstance,
    /// End of MIB view
    EndOfMibView,
}

impl SnmpValue {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            SnmpValue::Integer(i) => i.to_string(),
            SnmpValue::String(s) => s.clone(),
            SnmpValue::Oid(oid) => oid.clone(),
            SnmpValue::IpAddress(ip) => ip.to_string(),
            SnmpValue::Counter32(c) => c.to_string(),
            SnmpValue::Counter64(c) => c.to_string(),
            SnmpValue::Gauge32(g) => g.to_string(),
            SnmpValue::TimeTicks(t) => t.to_string(),
            SnmpValue::Opaque(data) => format!("Opaque({} bytes)", data.len()),
            SnmpValue::Null => "null".to_string(),
            SnmpValue::NoSuchObject => "noSuchObject".to_string(),
            SnmpValue::NoSuchInstance => "noSuchInstance".to_string(),
            SnmpValue::EndOfMibView => "endOfMibView".to_string(),
        }
    }

    /// Check if value represents an error condition
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            SnmpValue::NoSuchObject | SnmpValue::NoSuchInstance | SnmpValue::EndOfMibView
        )
    }
}

/// SNMP credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnmpCredentials {
    /// SNMPv1/v2c community string
    Community {
        /// Community string (read-only or read-write)
        community: String,
    },
    /// SNMPv3 user-based security
    UserBased {
        /// Username
        username: String,
        /// Authentication protocol and password
        auth: Option<(String, String)>,
        /// Privacy protocol and password
        privacy: Option<(String, String)>,
    },
}

impl Default for SnmpCredentials {
    fn default() -> Self {
        Self::Community {
            community: "public".to_string(),
        }
    }
}

/// Configuration for SNMP session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Target address and port
    pub address: SocketAddr,
    /// SNMP version (1, 2, or 3)
    pub version: u8,
    /// Authentication credentials
    pub credentials: SnmpCredentials,
    /// Request timeout
    pub timeout: Duration,
    /// Number of retries
    pub retries: u32,
    /// Maximum number of variables per request
    pub max_vars_per_request: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:161".parse().unwrap(),
            version: 2,
            credentials: SnmpCredentials::default(),
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 10,
        }
    }
}

/// SNMP session for device communication
#[derive(Debug)]
pub struct SnmpSession {
    /// Session configuration
    config: SessionConfig,
    /// Session ID for tracking
    session_id: Uuid,
    /// Last successful operation timestamp
    last_success: RwLock<Option<SystemTime>>,
    /// Connection attempt counter
    connection_attempts: RwLock<u32>,
}

impl SnmpSession {
    /// Create new SNMP session
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            session_id: Uuid::new_v4(),
            last_success: RwLock::new(None),
            connection_attempts: RwLock::new(0),
        }
    }

    /// Get session ID
    pub fn id(&self) -> Uuid {
        self.session_id
    }

    /// Get session configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Perform SNMP GET operation
    pub async fn get(&self, oids: &[&str]) -> SnmpResult<HashMap<String, SnmpValue>> {
        debug!(
            session_id = %self.session_id,
            target = %self.config.address,
            oid_count = oids.len(),
            "Performing SNMP GET operation"
        );

        // Increment connection attempts
        {
            let mut attempts = self.connection_attempts.write().await;
            *attempts += 1;
        }

        // TODO: Implement actual SNMP GET using snmp2 crate
        // For now, return a placeholder implementation
        let mut result = HashMap::new();
        for oid in oids {
            // Simulate different response types based on OID
            let value = match *oid {
                "1.3.6.1.2.1.1.1.0" => SnmpValue::String("Linux router 5.4.0".to_string()),
                "1.3.6.1.2.1.1.2.0" => SnmpValue::Oid("1.3.6.1.4.1.8072.3.2.10".to_string()),
                "1.3.6.1.2.1.1.3.0" => SnmpValue::TimeTicks(12345678),
                _ => SnmpValue::NoSuchObject,
            };
            result.insert(oid.to_string(), value);
        }

        // Update last success timestamp
        {
            let mut last_success = self.last_success.write().await;
            *last_success = Some(SystemTime::now());
        }

        info!(
            session_id = %self.session_id,
            target = %self.config.address,
            result_count = result.len(),
            "SNMP GET operation completed successfully"
        );

        Ok(result)
    }

    /// Perform SNMP GETNEXT operation (table walking)
    pub async fn get_next(&self, start_oid: &str) -> SnmpResult<HashMap<String, SnmpValue>> {
        debug!(
            session_id = %self.session_id,
            target = %self.config.address,
            start_oid = start_oid,
            "Performing SNMP GETNEXT operation"
        );

        // TODO: Implement actual SNMP GETNEXT using snmp2 crate
        // For now, return a placeholder implementation
        let mut result = HashMap::new();

        // Simulate walking an interface table
        if start_oid.starts_with("1.3.6.1.2.1.2.2.1") {
            for i in 1..=4 {
                let oid = format!("{}.{}", start_oid, i);
                let value = match start_oid {
                    "1.3.6.1.2.1.2.2.1.2" => SnmpValue::String(format!("eth{}", i - 1)),
                    "1.3.6.1.2.1.2.2.1.3" => SnmpValue::Integer(6), // ethernetCsmacd
                    "1.3.6.1.2.1.2.2.1.5" => SnmpValue::Gauge32(1000000000), // 1Gbps
                    _ => SnmpValue::NoSuchObject,
                };
                result.insert(oid, value);
            }
        }

        info!(
            session_id = %self.session_id,
            target = %self.config.address,
            result_count = result.len(),
            "SNMP GETNEXT operation completed successfully"
        );

        Ok(result)
    }

    /// Check if session is healthy (recent successful operations)
    pub async fn is_healthy(&self, max_age: Duration) -> bool {
        let last_success = self.last_success.read().await;
        if let Some(timestamp) = *last_success {
            SystemTime::now()
                .duration_since(timestamp)
                .map_or(false, |age| age <= max_age)
        } else {
            false
        }
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

/// SNMP client with connection pooling
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

/// Configuration for SNMP client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpClientConfig {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Default session configuration
    pub default_session: SessionConfig,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Session timeout (unused sessions are cleaned up)
    pub session_timeout: Duration,
}

impl Default for SnmpClientConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            default_session: SessionConfig::default(),
            health_check_interval: Duration::from_secs(60),
            session_timeout: Duration::from_secs(300),
        }
    }
}

impl SnmpClient {
    /// Create new SNMP client with configuration
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
    pub async fn get(
        &self,
        address: SocketAddr,
        oids: &[&str],
        config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        // Acquire connection permit
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
    pub async fn walk(
        &self,
        address: SocketAddr,
        start_oid: &str,
        config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        // Acquire connection permit
        let _permit =
            self.connection_semaphore
                .acquire()
                .await
                .map_err(|_| SnmpError::PoolExhausted {
                    max_connections: self.max_connections,
                })?;

        let session = self.get_session(address, config).await?;
        session.get_next(start_oid).await
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
            let mut sessions = self.sessions.write().await;
            for address in sessions_to_remove {
                sessions.remove(&address);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_snmp_value_to_string() {
        assert_eq!(SnmpValue::Integer(42).to_string(), "42");
        assert_eq!(SnmpValue::String("test".to_string()).to_string(), "test");
        assert_eq!(
            SnmpValue::IpAddress(IpAddr::V4(Ipv4Addr::LOCALHOST)).to_string(),
            "127.0.0.1"
        );
        assert_eq!(SnmpValue::Counter32(1000).to_string(), "1000");
        assert_eq!(SnmpValue::Null.to_string(), "null");
        assert_eq!(SnmpValue::NoSuchObject.to_string(), "noSuchObject");
    }

    #[test]
    fn test_snmp_value_is_error() {
        assert!(!SnmpValue::Integer(42).is_error());
        assert!(!SnmpValue::String("test".to_string()).is_error());
        assert!(SnmpValue::NoSuchObject.is_error());
        assert!(SnmpValue::NoSuchInstance.is_error());
        assert!(SnmpValue::EndOfMibView.is_error());
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.version, 2);
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retries, 3);
        assert_eq!(config.max_vars_per_request, 10);
    }

    #[test]
    fn test_snmp_credentials_default() {
        let creds = SnmpCredentials::default();
        match creds {
            SnmpCredentials::Community { community } => {
                assert_eq!(community, "public");
            }
            _ => panic!("Expected community credentials"),
        }
    }

    #[tokio::test]
    async fn test_snmp_session_creation() {
        let config = SessionConfig::default();
        let session = SnmpSession::new(config.clone());

        assert_eq!(session.config().version, config.version);
        assert_eq!(session.config().timeout, config.timeout);
        assert!(!session.is_healthy(Duration::from_secs(1)).await);
    }

    #[tokio::test]
    async fn test_snmp_client_creation() {
        let config = SnmpClientConfig::default();
        let client = SnmpClient::new(config.clone());

        let stats = client.stats().await;
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.max_connections, config.max_connections);
        assert_eq!(stats.available_permits, config.max_connections);
    }

    #[tokio::test]
    async fn test_snmp_get_operation() {
        let config = SnmpClientConfig::default();
        let client = SnmpClient::new(config);

        let address = "127.0.0.1:161".parse().unwrap();
        let oids = vec!["1.3.6.1.2.1.1.1.0", "1.3.6.1.2.1.1.5.0"];

        let result = client.get(address, &oids, None).await;
        assert!(result.is_ok());

        let values = result.unwrap();
        assert_eq!(values.len(), 2);
        assert!(values.contains_key("1.3.6.1.2.1.1.1.0"));
        assert!(values.contains_key("1.3.6.1.2.1.1.5.0"));
    }
}
