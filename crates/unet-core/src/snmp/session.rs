//! SNMP session management

use super::config::{SessionConfig, SnmpCredentials};
use super::values::SnmpValue;
use super::{SnmpError, SnmpResult};
use csnmp::{ObjectIdentifier, ObjectValue, Snmp2cClient};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// SNMP session for device communication
pub struct SnmpSession {
    /// Session configuration
    config: SessionConfig,
    /// Session ID for tracking
    session_id: Uuid,
    /// Last successful operation timestamp
    last_success: RwLock<Option<SystemTime>>,
    /// Connection attempt counter
    connection_attempts: RwLock<u32>,
    /// Underlying SNMP client
    client: Option<Snmp2cClient>,
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
    async fn create_client(config: &SessionConfig) -> SnmpResult<Snmp2cClient> {
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
    async fn get_client(&mut self) -> SnmpResult<&Snmp2cClient> {
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

    /// Perform SNMP GET operation
    ///
    /// # Errors
    ///
    /// Returns `SnmpError` if the SNMP request fails or times out
    pub async fn get(&mut self, oids: &[&str]) -> SnmpResult<HashMap<String, SnmpValue>> {
        self.log_operation_start(oids.len());
        self.increment_connection_attempts().await;

        let oid_objects = Self::parse_oids(oids)?;
        let session_id = self.session_id;
        let target_address = self.config.address;

        let client = self.get_client().await?;
        let result =
            Self::execute_get_requests_static(client, &oid_objects, oids, session_id).await;

        self.update_success_timestamp(&result).await;
        Self::log_operation_completion(session_id, target_address, &result);

        Ok(result)
    }

    /// Log the start of an SNMP operation
    fn log_operation_start(&self, oid_count: usize) {
        debug!(
            session_id = %self.session_id,
            target = %self.config.address,
            oid_count = oid_count,
            "Performing SNMP GET operation"
        );
    }

    /// Increment the connection attempts counter
    async fn increment_connection_attempts(&self) {
        let mut attempts = self.connection_attempts.write().await;
        *attempts += 1;
    }

    /// Execute GET requests for all OIDs
    async fn execute_get_requests_static(
        client: &Snmp2cClient,
        oid_objects: &[ObjectIdentifier],
        oids: &[&str],
        session_id: Uuid,
    ) -> HashMap<String, SnmpValue> {
        let mut result = HashMap::new();

        for (i, oid) in oid_objects.iter().enumerate() {
            match client.get(*oid).await {
                Ok(value) => {
                    let oid_str = oid.to_string();
                    let snmp_value = convert_object_value_to_snmp_value(&value);
                    result.insert(oid_str, snmp_value);
                }
                Err(e) => {
                    result.insert(oids[i].to_string(), SnmpValue::NoSuchObject);
                    debug!(
                        session_id = %session_id,
                        oid = oids[i],
                        error = %e,
                        "Failed to get OID value"
                    );
                }
            }
        }

        result
    }

    /// Update the last success timestamp if we got results
    async fn update_success_timestamp(&self, result: &HashMap<String, SnmpValue>) {
        if !result.is_empty() {
            let mut last_success = self.last_success.write().await;
            *last_success = Some(SystemTime::now());
        }
    }

    /// Log the completion of an SNMP operation
    fn log_operation_completion(
        session_id: Uuid,
        target_address: std::net::SocketAddr,
        result: &HashMap<String, SnmpValue>,
    ) {
        info!(
            session_id = %session_id,
            target = %target_address,
            result_count = result.len(),
            "SNMP GET operation completed"
        );
    }

    fn parse_oids(oids: &[&str]) -> SnmpResult<Vec<ObjectIdentifier>> {
        oids.iter()
            .map(|oid_str| {
                oid_str
                    .parse::<ObjectIdentifier>()
                    .map_err(|_| SnmpError::InvalidOid {
                        oid: (*oid_str).to_string(),
                    })
            })
            .collect()
    }

    /// Perform SNMP GETNEXT operation (table walking)
    ///
    /// # Errors
    ///
    /// Returns `SnmpError` if:
    /// - The SNMP client cannot be established
    /// - The target device is unreachable
    /// - The OID is invalid or not supported
    pub async fn get_next(&mut self, start_oid: &str) -> SnmpResult<HashMap<String, SnmpValue>> {
        debug!(
            session_id = %self.session_id,
            target = %self.config.address,
            start_oid = start_oid,
            "Performing SNMP GETNEXT operation"
        );

        // Parse start OID
        let start_oid_obj =
            start_oid
                .parse::<ObjectIdentifier>()
                .map_err(|_| SnmpError::InvalidOid {
                    oid: start_oid.to_string(),
                })?;

        // Get SNMP client
        let client = self.get_client().await?;

        // Execute SNMP GETNEXT request using walk_bulk with limit 1
        let response =
            client
                .walk_bulk(start_oid_obj, 1)
                .await
                .map_err(|e| SnmpError::Protocol {
                    message: format!("SNMP GETNEXT failed: {e}"),
                })?;

        // Convert response to our format
        let mut result = HashMap::new();
        for (oid, value) in response {
            let oid_str = oid.to_string();
            let snmp_value = convert_object_value_to_snmp_value(&value);
            result.insert(oid_str, snmp_value);
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

/// Convert csnmp `ObjectValue` to our `SnmpValue` type
fn convert_object_value_to_snmp_value(value: &ObjectValue) -> SnmpValue {
    match value {
        ObjectValue::Integer(i) => SnmpValue::Integer((*i).into()),
        ObjectValue::String(bytes) => {
            // Try to convert to UTF-8 string, fall back to hex if not valid UTF-8
            std::str::from_utf8(bytes).map_or_else(
                |_| {
                    // Convert bytes to hex string without external hex crate
                    let hex_string = bytes.iter().fold(String::new(), |mut acc, b| {
                        use std::fmt::Write;
                        let _ = write!(acc, "{b:02x}");
                        acc
                    });
                    SnmpValue::String(format!("0x{hex_string}"))
                },
                |s| SnmpValue::String(s.to_string()),
            )
        }
        ObjectValue::ObjectId(oid) => SnmpValue::Oid(oid.to_string()),
        ObjectValue::IpAddress(ip) => SnmpValue::IpAddress(std::net::IpAddr::V4(*ip)),
        ObjectValue::Counter32(c) => SnmpValue::Counter32(*c),
        ObjectValue::Unsigned32(u) => SnmpValue::Gauge32(*u), // Map Unsigned32 to Gauge32
        ObjectValue::TimeTicks(t) => SnmpValue::TimeTicks(*t),
        ObjectValue::Counter64(c) => SnmpValue::Counter64(*c),
        ObjectValue::Opaque(bytes) => {
            // Handle opaque data as hex string
            let hex_string = bytes.iter().fold(String::new(), |mut acc, b| {
                use std::fmt::Write;
                let _ = write!(acc, "{b:02x}");
                acc
            });
            SnmpValue::String(format!("0x{hex_string}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snmp::SnmpCredentials;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;

    fn create_test_config() -> SessionConfig {
        SessionConfig {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
            version: 2,
            credentials: SnmpCredentials::Community {
                community: "public".to_string(),
            },
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 10,
        }
    }

    fn create_test_config_with_user_based() -> SessionConfig {
        SessionConfig {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
            version: 3,
            credentials: SnmpCredentials::UserBased {
                username: "testuser".to_string(),
                auth: Some(("SHA".to_string(), "authpass".to_string())),
                privacy: Some(("AES".to_string(), "privpass".to_string())),
            },
            timeout: Duration::from_secs(5),
            retries: 3,
            max_vars_per_request: 10,
        }
    }

    #[test]
    fn test_session_creation() {
        let config = create_test_config();
        let session = SnmpSession::new(config.clone());

        assert_eq!(session.config(), &config);
        assert!(session.client.is_none());
    }

    #[test]
    fn test_session_accessors() {
        let config = create_test_config();
        let session = SnmpSession::new(config.clone());

        // Test id() method
        let session_id = session.id();
        assert_ne!(session_id, Uuid::nil());

        // Test config() method
        assert_eq!(session.config(), &config);
    }

    #[tokio::test]
    async fn test_create_client_with_community() {
        let config = create_test_config();

        // This will likely fail since we don't have a real SNMP server,
        // but we want to test the code path (lines 43-58)
        let result = SnmpSession::create_client(&config).await;

        // We expect either success or a specific error
        match result {
            Ok(_) | Err(SnmpError::Network(_)) => {
                // Success case or network errors are expected when no SNMP server is running
            }
            Err(SnmpError::Protocol { message }) => {
                assert!(message.contains("Failed to create SNMP client"));
            }
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_create_client_with_user_based() {
        let config = create_test_config_with_user_based();

        // Test unsupported SNMPv3 user-based security (lines 60-62)
        let result = SnmpSession::create_client(&config).await;

        match result {
            Err(SnmpError::Protocol { message }) => {
                assert!(message.contains("SNMPv3 user-based security not supported"));
            }
            _ => panic!("Expected Protocol error for SNMPv3"),
        }
    }

    #[tokio::test]
    async fn test_connection_attempts() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        // Initial connection attempts should be 0
        assert_eq!(session.connection_attempts().await, 0);
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        // New session should not be healthy (no successful operations)
        assert!(!session.is_healthy(Duration::from_secs(10)).await);
    }

    #[test]
    fn test_parse_oids_valid() {
        let oids = &["1.3.6.1.2.1.1.1.0", "1.3.6.1.2.1.1.5.0"];

        // Test valid OIDs (lines 154-164)
        let result = SnmpSession::parse_oids(oids);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_oids_invalid() {
        let oids = &["invalid.oid", "1.3.6.1.2.1.1.1.0"];

        // Test invalid OIDs (lines 159-161)
        let result = SnmpSession::parse_oids(oids);
        match result {
            Err(SnmpError::InvalidOid { oid }) => {
                assert_eq!(oid, "invalid.oid");
            }
            _ => panic!("Expected InvalidOid error"),
        }
    }

    #[test]
    fn test_convert_object_value_to_snmp_value() {
        use csnmp::ObjectValue;

        // Test Integer conversion (line 257)
        let int_value = ObjectValue::Integer(42);
        let snmp_value = convert_object_value_to_snmp_value(&int_value);
        assert_eq!(snmp_value, SnmpValue::Integer(42));

        // Test String conversion with valid UTF-8 (lines 258, 270)
        let string_bytes = b"test".to_vec();
        let string_value = ObjectValue::String(string_bytes);
        let snmp_value = convert_object_value_to_snmp_value(&string_value);
        assert_eq!(snmp_value, SnmpValue::String("test".to_string()));

        // Test String conversion with invalid UTF-8 (lines 260-268)
        let invalid_utf8 = vec![0xFF, 0xFE];
        let invalid_string_value = ObjectValue::String(invalid_utf8);
        let snmp_value = convert_object_value_to_snmp_value(&invalid_string_value);
        if let SnmpValue::String(s) = snmp_value {
            assert!(s.starts_with("0x"));
            assert!(s.contains("ff"));
            assert!(s.contains("fe"));
        } else {
            panic!("Expected String value with hex encoding");
        }

        // Test ObjectId conversion (line 273)
        let oid = "1.3.6.1.2.1.1.1.0".parse().unwrap();
        let oid_value = ObjectValue::ObjectId(oid);
        let snmp_value = convert_object_value_to_snmp_value(&oid_value);
        assert_eq!(snmp_value, SnmpValue::Oid("1.3.6.1.2.1.1.1.0".to_string()));

        // Test IpAddress conversion (line 274)
        let ip = Ipv4Addr::new(192, 168, 1, 1);
        let ip_value = ObjectValue::IpAddress(ip);
        let snmp_value = convert_object_value_to_snmp_value(&ip_value);
        assert_eq!(snmp_value, SnmpValue::IpAddress(IpAddr::V4(ip)));

        // Test Counter32 conversion (line 275)
        let counter_value = ObjectValue::Counter32(12345);
        let snmp_value = convert_object_value_to_snmp_value(&counter_value);
        assert_eq!(snmp_value, SnmpValue::Counter32(12345));

        // Test Unsigned32 -> Gauge32 conversion (line 276)
        let unsigned_value = ObjectValue::Unsigned32(54321);
        let snmp_value = convert_object_value_to_snmp_value(&unsigned_value);
        assert_eq!(snmp_value, SnmpValue::Gauge32(54321));

        // Test TimeTicks conversion (line 277)
        let timeticks_value = ObjectValue::TimeTicks(98765);
        let snmp_value = convert_object_value_to_snmp_value(&timeticks_value);
        assert_eq!(snmp_value, SnmpValue::TimeTicks(98765));

        // Test Counter64 conversion (line 278)
        let counter64_value = ObjectValue::Counter64(1_234_567_890_123);
        let snmp_value = convert_object_value_to_snmp_value(&counter64_value);
        assert_eq!(snmp_value, SnmpValue::Counter64(1_234_567_890_123));

        // Test Opaque conversion (lines 279-287)
        let opaque_data = vec![0xAB, 0xCD, 0xEF];
        let opaque_value = ObjectValue::Opaque(opaque_data);
        let snmp_value = convert_object_value_to_snmp_value(&opaque_value);
        if let SnmpValue::String(s) = snmp_value {
            assert!(s.starts_with("0x"));
            assert!(s.contains("ab"));
            assert!(s.contains("cd"));
            assert!(s.contains("ef"));
        } else {
            panic!("Expected String value with hex encoding for opaque data");
        }
    }

    #[test]
    fn test_session_clone() {
        let config = create_test_config();
        let session = SnmpSession::new(config.clone());

        // Test clone implementation (lines 236-239)
        let cloned_session = session.clone();
        assert_eq!(cloned_session.config(), &config);
        assert_ne!(cloned_session.id(), session.id()); // Different session IDs
    }

    #[test]
    fn test_session_debug() {
        let config = create_test_config();
        let session = SnmpSession::new(config);

        // Test Debug implementation (lines 242-251)
        let debug_str = format!("{session:?}");
        assert!(debug_str.contains("SnmpSession"));
        assert!(debug_str.contains("config"));
        assert!(debug_str.contains("session_id"));
    }
}
