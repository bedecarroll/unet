//! SNMP session management

use super::config::{SessionConfig, SnmpCredentials};
use super::values::SnmpValue;
use super::{SnmpError, SnmpResult};
use snmp2::{AsyncSession, Oid, Value};
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
    /// Underlying SNMP session
    session: Option<AsyncSession>,
}

impl SnmpSession {
    /// Create new SNMP session
    #[must_use]
    pub fn new(config: SessionConfig) -> Self {
        // We'll create the actual session lazily when needed
        // This avoids blocking async code in constructor
        Self {
            config,
            session_id: Uuid::new_v4(),
            last_success: RwLock::new(None),
            connection_attempts: RwLock::new(0),
            session: None,
        }
    }

    /// Get or create the underlying SNMP session
    async fn get_session(&mut self) -> SnmpResult<&mut AsyncSession> {
        if self.session.is_none() {
            let address = format!(
                "{}:{}",
                self.config.address.ip(),
                self.config.address.port()
            );

            let session = match &self.config.credentials {
                SnmpCredentials::Community { community } => match self.config.version {
                    1 => AsyncSession::new_v1(&address, community.as_bytes(), 0).await,
                    2 => AsyncSession::new_v2c(&address, community.as_bytes(), 0).await,
                    v => {
                        return Err(SnmpError::Protocol {
                            message: format!(
                                "Community credentials not supported for SNMP version {v}"
                            ),
                        });
                    }
                },
                SnmpCredentials::UserBased { .. } => {
                    return Err(SnmpError::Protocol {
                        message: "SNMPv3 user-based security not yet implemented".to_string(),
                    });
                }
            }
            .map_err(|e| SnmpError::Protocol {
                message: format!("Failed to create SNMP session: {e}"),
            })?;

            self.session = Some(session);
        }

        Ok(self.session.as_mut().unwrap())
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

        // Convert string OIDs to Oid objects
        let mut oid_objects = Vec::new();
        for oid_str in oids {
            // Parse OID string into array of integers
            let oid_parts: Result<Vec<u64>, _> =
                oid_str.split('.').map(|part| part.parse::<u64>()).collect();

            let oid_array = oid_parts.map_err(|_| SnmpError::InvalidOid {
                oid: (*oid_str).to_string(),
            })?;

            let oid = Oid::from(&oid_array).map_err(|_| SnmpError::InvalidOid {
                oid: (*oid_str).to_string(),
            })?;

            oid_objects.push(oid);
        }

        // Cache values before mutable borrow
        let session_id = self.session_id;
        let target_address = self.config.address;

        // Get SNMP session
        let session = self.get_session().await?;

        // Execute SNMP GET request for each OID
        let mut result = HashMap::new();

        for (i, oid) in oid_objects.iter().enumerate() {
            match session.get(oid).await {
                Ok(response) => {
                    for (resp_oid, value) in response.varbinds {
                        let oid_str = format_oid(&resp_oid);
                        let snmp_value = convert_snmp2_value_to_snmp_value(&value);
                        result.insert(oid_str, snmp_value);
                    }
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
    /// - The SNMP session cannot be established
    /// - The target device is unreachable
    /// - The OID is invalid or not supported
    pub async fn get_next(&mut self, start_oid: &str) -> SnmpResult<HashMap<String, SnmpValue>> {
        debug!(
            session_id = %self.session_id,
            target = %self.config.address,
            start_oid = start_oid,
            "Performing SNMP GETNEXT operation"
        );

        // Parse start OID string into array of integers
        let oid_parts: Result<Vec<u64>, _> = start_oid
            .split('.')
            .map(|part| part.parse::<u64>())
            .collect();

        let oid_array = oid_parts.map_err(|_| SnmpError::InvalidOid {
            oid: start_oid.to_string(),
        })?;

        let start_oid_obj = Oid::from(&oid_array).map_err(|_| SnmpError::InvalidOid {
            oid: start_oid.to_string(),
        })?;

        // Get SNMP session
        let session = self.get_session().await?;

        // Execute SNMP GETNEXT request
        let response = session
            .getnext(&start_oid_obj)
            .await
            .map_err(|e| SnmpError::Protocol {
                message: format!("SNMP GETNEXT failed: {e}"),
            })?;

        // Convert response to our format
        let mut result = HashMap::new();
        for (oid, value) in response.varbinds {
            let oid_str = format_oid(&oid);
            let snmp_value = convert_snmp2_value_to_snmp_value(&value);
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
            .field("session", &"<AsyncSession>")
            .finish()
    }
}

/// Convert snmp2 Value to our SnmpValue type
fn convert_snmp2_value_to_snmp_value(value: &Value) -> SnmpValue {
    match value {
        Value::Integer(i) => SnmpValue::Integer(*i),
        Value::OctetString(bytes) => {
            // Try to convert to UTF-8 string, fall back to hex if not valid UTF-8
            match std::str::from_utf8(bytes) {
                Ok(s) => SnmpValue::String(s.to_string()),
                Err(_) => {
                    // Convert bytes to hex string without external hex crate
                    let hex_string = bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join("");
                    SnmpValue::String(format!("0x{hex_string}"))
                }
            }
        }
        Value::ObjectIdentifier(oid) => SnmpValue::Oid(format_oid(oid)),
        Value::IpAddress(ip) => SnmpValue::IpAddress(std::net::IpAddr::V4(
            std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]),
        )),
        Value::Counter32(c) => SnmpValue::Counter32(*c),
        Value::Unsigned32(u) => SnmpValue::Gauge32(*u), // Map Unsigned32 to Gauge32
        Value::Timeticks(t) => SnmpValue::TimeTicks(*t),
        Value::Counter64(c) => SnmpValue::Counter64(*c),
        Value::Null => SnmpValue::Null,
        // Handle error cases
        _ => SnmpValue::NoSuchObject,
    }
}

/// Format an OID as a string
fn format_oid(oid: &Oid) -> String {
    // snmp2 Oid implements Display, so we can just convert to string
    oid.to_string()
}
