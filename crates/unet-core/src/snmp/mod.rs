//! SNMP client integration for μNet
//!
//! This module provides SNMP polling capabilities for network devices to collect
//! derived state information. It supports `SNMPv2c` and `SNMPv3` protocols with
//! connection pooling and error handling.
//!
//! # Architecture
//!
//! - [`client`] - SNMP client wrapper with connection pooling
//! - [`oids`] - Standard and vendor-specific OID definitions
//! - [`session`] - SNMP session management
//! - [`poller`] - Background polling implementation
//! - [`types`] - SNMP-specific data types

use std::time::Duration;
use thiserror::Error;

pub mod client;
pub mod config;
pub mod oids;
pub mod poller;
pub mod session;
pub mod types;
pub mod values;

// Re-export main types for backward compatibility
pub use client::{SnmpClient, SnmpClientStats};
pub use config::{SessionConfig, SnmpClientConfig, SnmpCredentials};
pub use oids::{OidMap, StandardOid, VendorOid};
pub use poller::{PollingConfig, PollingHandle, PollingResult, PollingScheduler, PollingTask};
pub use session::SnmpSession;
pub use types::SnmpType;
pub use values::SnmpValue;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{defaults, network};
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

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
            SnmpCredentials::UserBased { .. } => panic!("Expected community credentials"),
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

        let address = network::parse_socket_addr(defaults::network::LOCALHOST_SNMP)
            .expect("Test should use valid SNMP address");
        let oids = vec!["1.3.6.1.2.1.1.1.0", "1.3.6.1.2.1.1.5.0"];

        let result = client.get(address, &oids, None).await;
        assert!(result.is_ok());

        let values = result.unwrap();
        assert_eq!(values.len(), 2);
        assert!(values.contains_key("1.3.6.1.2.1.1.1.0"));
        assert!(values.contains_key("1.3.6.1.2.1.1.5.0"));
    }
}
