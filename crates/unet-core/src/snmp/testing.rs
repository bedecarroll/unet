//! Testing utilities for SNMP components
//!
//! This module provides mock implementations and testing utilities for SNMP-related
//! functionality to enable fast, reliable unit tests without actual network calls.

use super::{SessionConfig, SnmpResult, SnmpValue};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

/// Trait for SNMP operations - allows for easy mocking in tests
#[async_trait]
pub trait SnmpOperations: Send + Sync {
    /// Perform SNMP GET operation on target
    async fn get(
        &self,
        address: SocketAddr,
        oids: &[&str],
        config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>>;

    /// Perform SNMP table walk on target
    async fn walk(
        &self,
        address: SocketAddr,
        start_oid: &str,
        config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>>;
}

/// Mock SNMP client for testing that returns predefined responses instantly
#[derive(Debug, Clone)]
pub struct MockSnmpClient {
    /// Predefined responses for GET operations
    pub get_responses: HashMap<String, SnmpResult<HashMap<String, SnmpValue>>>,
    /// Predefined responses for WALK operations
    pub walk_responses: HashMap<String, SnmpResult<HashMap<String, SnmpValue>>>,
    /// Whether operations should succeed by default
    pub default_success: bool,
}

impl MockSnmpClient {
    /// Create a new mock SNMP client
    #[must_use]
    pub fn new() -> Self {
        Self {
            get_responses: HashMap::new(),
            walk_responses: HashMap::new(),
            default_success: true,
        }
    }

    /// Create a mock client that always succeeds with default values
    #[must_use]
    pub fn success() -> Self {
        Self {
            get_responses: HashMap::new(),
            walk_responses: HashMap::new(),
            default_success: true,
        }
    }

    /// Create a mock client that always fails
    #[must_use]
    pub fn failure() -> Self {
        Self {
            get_responses: HashMap::new(),
            walk_responses: HashMap::new(),
            default_success: false,
        }
    }

    /// Add a predefined response for a specific target and OIDs
    #[must_use]
    pub fn with_get_response(
        mut self,
        target: SocketAddr,
        oids: &[&str],
        response: SnmpResult<HashMap<String, SnmpValue>>,
    ) -> Self {
        let key = format!("{}:{}", target, oids.join(","));
        self.get_responses.insert(key, response);
        self
    }

    /// Add a predefined response for a specific walk operation
    #[must_use]
    pub fn with_walk_response(
        mut self,
        target: SocketAddr,
        start_oid: &str,
        response: SnmpResult<HashMap<String, SnmpValue>>,
    ) -> Self {
        let key = format!("{target}:{start_oid}");
        self.walk_responses.insert(key, response);
        self
    }

    /// Create default successful response for testing
    fn create_default_success_response(oids: &[&str]) -> HashMap<String, SnmpValue> {
        oids.iter()
            .enumerate()
            .map(|(i, oid)| {
                let value = match *oid {
                    "1.3.6.1.2.1.1.1.0" => SnmpValue::String("Test Device".to_string()), // sysDescr
                    "1.3.6.1.2.1.1.3.0" => SnmpValue::Counter32(12345), // sysUpTime
                    "1.3.6.1.2.1.1.5.0" => SnmpValue::String("test-device".to_string()), // sysName
                    _ => SnmpValue::String(format!("test-value-{i}")),
                };
                ((*oid).to_string(), value)
            })
            .collect()
    }

    /// Create default failure response
    fn create_default_failure() -> SnmpResult<HashMap<String, SnmpValue>> {
        Err(super::SnmpError::Network {
            message: "Mock connection refused".to_string(),
        })
    }
}

impl Default for MockSnmpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock SNMP client that introduces delays for timeout testing
#[derive(Debug, Clone)]
pub struct DelayedMockSnmpClient {
    /// The delay to introduce before responding
    pub delay: Duration,
    /// Whether to succeed after delay
    pub success: bool,
}

impl DelayedMockSnmpClient {
    /// Create a new delayed mock client
    #[must_use]
    pub fn new(delay: Duration, success: bool) -> Self {
        Self { delay, success }
    }
}

#[async_trait]
impl SnmpOperations for DelayedMockSnmpClient {
    async fn get(
        &self,
        _address: SocketAddr,
        oids: &[&str],
        _config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        tokio::time::sleep(self.delay).await;

        if self.success {
            Ok(MockSnmpClient::create_default_success_response(oids))
        } else {
            MockSnmpClient::create_default_failure()
        }
    }

    async fn walk(
        &self,
        _address: SocketAddr,
        start_oid: &str,
        _config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        tokio::time::sleep(self.delay).await;

        if self.success {
            let mut result = HashMap::new();
            result.insert(
                format!("{start_oid}.1"),
                SnmpValue::String("test-walk-value".to_string()),
            );
            Ok(result)
        } else {
            MockSnmpClient::create_default_failure()
        }
    }
}

#[async_trait]
impl SnmpOperations for MockSnmpClient {
    async fn get(
        &self,
        address: SocketAddr,
        oids: &[&str],
        _config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        let key = format!("{}:{}", address, oids.join(","));

        self.get_responses.get(&key).map_or_else(
            || {
                if self.default_success {
                    Ok(Self::create_default_success_response(oids))
                } else {
                    Self::create_default_failure()
                }
            },
            |response| match response {
                Ok(values) => Ok(values.clone()),
                Err(e) => Err(super::SnmpError::Network {
                    message: format!("Mock error: {e}"),
                }),
            },
        )
    }

    async fn walk(
        &self,
        address: SocketAddr,
        start_oid: &str,
        _config: Option<SessionConfig>,
    ) -> SnmpResult<HashMap<String, SnmpValue>> {
        let key = format!("{address}:{start_oid}");

        self.walk_responses.get(&key).map_or_else(
            || {
                if self.default_success {
                    let mut result = HashMap::new();
                    result.insert(
                        format!("{start_oid}.1"),
                        SnmpValue::String("test-walk-value".to_string()),
                    );
                    Ok(result)
                } else {
                    Self::create_default_failure()
                }
            },
            |response| match response {
                Ok(values) => Ok(values.clone()),
                Err(e) => Err(super::SnmpError::Network {
                    message: format!("Mock error: {e}"),
                }),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_mock_snmp_client_success() {
        let client = MockSnmpClient::success();
        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
        let oids = vec!["1.3.6.1.2.1.1.1.0", "1.3.6.1.2.1.1.3.0"];

        let result = client.get(target, &oids, None).await;
        assert!(result.is_ok());

        let values = result.unwrap();
        assert_eq!(values.len(), 2);
        assert!(values.contains_key("1.3.6.1.2.1.1.1.0"));
        assert!(values.contains_key("1.3.6.1.2.1.1.3.0"));
    }

    #[tokio::test]
    async fn test_mock_snmp_client_failure() {
        let client = MockSnmpClient::failure();
        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
        let oids = vec!["1.3.6.1.2.1.1.1.0"];

        let result = client.get(target, &oids, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_snmp_client_predefined_response() {
        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
        let oids = vec!["1.3.6.1.2.1.1.1.0"];

        let mut expected_values = HashMap::new();
        expected_values.insert(
            "1.3.6.1.2.1.1.1.0".to_string(),
            SnmpValue::String("Custom Response".to_string()),
        );

        let client =
            MockSnmpClient::new().with_get_response(target, &oids, Ok(expected_values.clone()));

        let result = client.get(target, &oids, None).await;
        assert!(result.is_ok());

        let values = result.unwrap();
        assert_eq!(
            values.get("1.3.6.1.2.1.1.1.0"),
            expected_values.get("1.3.6.1.2.1.1.1.0")
        );
    }

    #[tokio::test]
    async fn test_mock_snmp_client_walk() {
        let client = MockSnmpClient::success();
        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 161);
        let start_oid = "1.3.6.1.2.1.1";

        let result = client.walk(target, start_oid, None).await;
        assert!(result.is_ok());

        let values = result.unwrap();
        assert!(!values.is_empty());
    }
}
