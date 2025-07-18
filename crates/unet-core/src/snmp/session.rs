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
    #[allow(clippy::cognitive_complexity)]
    pub async fn get(&mut self, oids: &[&str]) -> SnmpResult<HashMap<String, SnmpValue>> {
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

        // Convert string OIDs to ObjectIdentifier objects
        let mut oid_objects = Vec::new();
        for oid_str in oids {
            let oid = oid_str
                .parse::<ObjectIdentifier>()
                .map_err(|_| SnmpError::InvalidOid {
                    oid: (*oid_str).to_string(),
                })?;
            oid_objects.push(oid);
        }

        // Cache values before mutable borrow
        let session_id = self.session_id;
        let target_address = self.config.address;

        // Get SNMP client
        let client = self.get_client().await?;

        // Execute SNMP GET request for each OID
        let mut result = HashMap::new();

        for (i, oid) in oid_objects.iter().enumerate() {
            match client.get(*oid).await {
                Ok(value) => {
                    let oid_str = oid.to_string();
                    let snmp_value = convert_object_value_to_snmp_value(&value);
                    result.insert(oid_str, snmp_value);
                }
                Err(e) => {
                    // For individual OID failures, insert error value
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

        // Update last success timestamp if we got any results
        if !result.is_empty() {
            let mut last_success = self.last_success.write().await;
            *last_success = Some(SystemTime::now());
        }

        info!(
            session_id = %session_id,
            target = %target_address,
            result_count = result.len(),
            "SNMP GET operation completed"
        );

        Ok(result)
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
