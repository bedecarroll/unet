//! SNMP configuration types

use crate::config::{defaults, network};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// SNMP credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnmpCredentials {
    /// SNMPv1/v2c community string
    Community {
        /// Community string (read-only or read-write)
        community: String,
    },
    /// `SNMPv3` user-based security
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
            address: network::parse_socket_addr(defaults::network::LOCALHOST_SNMP)
                .expect("Default LOCALHOST_SNMP constant should always be valid"),
            version: 2,
            credentials: SnmpCredentials::default(),
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 10,
        }
    }
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

#[cfg(test)]
mod tests;
