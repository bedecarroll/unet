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
    async fn test_snmp_session_creation_and_config() {
        // Test session creation without network operations
        let address = network::parse_socket_addr(defaults::network::LOCALHOST_SNMP)
            .expect("Test should use valid SNMP address");

        let config = SessionConfig {
            address,
            version: 2,
            credentials: SnmpCredentials::Community {
                community: "public".to_string(),
            },
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 10,
        };

        let session = SnmpSession::new(config);

        // Verify session properties without making network calls
        assert_eq!(session.config().address, address);
        assert_eq!(session.config().version, 2);
        assert_eq!(session.connection_attempts().await, 0);
        assert!(!session.is_healthy(Duration::from_secs(1)).await);

        // Verify session ID is generated
        let id1 = session.id();
        let session2 = SnmpSession::new(session.config().clone());
        let id2 = session2.id();
        assert_ne!(id1, id2); // Each session should have unique ID
    }

    #[test]
    fn test_snmp_error_display() {
        let network_error = SnmpError::Network(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Connection refused",
        ));
        assert!(network_error.to_string().contains("Network error"));
        assert!(network_error.to_string().contains("Connection refused"));

        let protocol_error = SnmpError::Protocol {
            message: "Invalid PDU".to_string(),
        };
        assert!(protocol_error.to_string().contains("SNMP protocol error"));
        assert!(protocol_error.to_string().contains("Invalid PDU"));

        let timeout_error = SnmpError::Timeout {
            duration: Duration::from_secs(5),
        };
        assert!(timeout_error.to_string().contains("SNMP timeout"));
        assert!(timeout_error.to_string().contains("5s"));

        let auth_error = SnmpError::Authentication;
        assert!(auth_error.to_string().contains("authentication failed"));

        let invalid_oid_error = SnmpError::InvalidOid {
            oid: "1.3.6.1.invalid".to_string(),
        };
        assert!(invalid_oid_error.to_string().contains("Invalid OID"));
        assert!(invalid_oid_error.to_string().contains("1.3.6.1.invalid"));

        let parse_error = SnmpError::ParseError {
            reason: "Malformed ASN.1".to_string(),
        };
        assert!(parse_error.to_string().contains("parse SNMP response"));
        assert!(parse_error.to_string().contains("Malformed ASN.1"));

        let pool_error = SnmpError::PoolExhausted {
            max_connections: 10,
        };
        assert!(pool_error.to_string().contains("pool exhausted"));
        assert!(pool_error.to_string().contains("10"));
    }

    #[test]
    fn test_snmp_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout");
        let snmp_error = SnmpError::from(io_error);

        match snmp_error {
            SnmpError::Network(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::TimedOut);
                assert_eq!(e.to_string(), "Timeout");
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_snmp_error_debug() {
        let error = SnmpError::Protocol {
            message: "Test error".to_string(),
        };
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("Protocol"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_snmp_result_type() {
        let success: SnmpResult<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(42, 42);

        let failure: SnmpResult<i32> = Err(SnmpError::Authentication);
        assert!(failure.is_err());

        match SnmpError::Authentication {
            SnmpError::Authentication => {} // Expected
            _ => panic!("Expected Authentication error"),
        }
    }

    #[test]
    fn test_snmp_error_all_variants() {
        let errors = vec![
            SnmpError::Network(std::io::Error::other("test")),
            SnmpError::Protocol {
                message: "test".to_string(),
            },
            SnmpError::Timeout {
                duration: Duration::from_secs(1),
            },
            SnmpError::Authentication,
            SnmpError::InvalidOid {
                oid: "test".to_string(),
            },
            SnmpError::ParseError {
                reason: "test".to_string(),
            },
            SnmpError::PoolExhausted { max_connections: 1 },
        ];

        for error in errors {
            // Test that each error variant has a string representation
            assert!(!error.to_string().is_empty());

            // Test that each error can be formatted with debug
            let debug_str = format!("{error:?}");
            assert!(!debug_str.is_empty());
        }
    }
}
