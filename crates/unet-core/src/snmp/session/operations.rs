//! SNMP session operations (GET, GETNEXT, etc.)

use super::super::values::SnmpValue;
use super::super::{SnmpError, SnmpResult};
use super::core::SnmpSession;
use super::utils::{convert_object_value_to_snmp_value, parse_oids};
use csnmp::ObjectIdentifier;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

impl SnmpSession {
    /// Perform SNMP GET operation
    ///
    /// # Errors
    ///
    /// Returns `SnmpError` if the SNMP request fails or times out
    pub async fn get(&mut self, oids: &[&str]) -> SnmpResult<HashMap<String, SnmpValue>> {
        self.log_operation_start("GET", oids.len());
        self.increment_connection_attempts().await;

        let oid_objects = parse_oids(oids)?;
        let session_id = self.session_id;
        let target_address = self.config.address;

        let client = self.get_client().await?;
        let result = execute_get_requests(client, &oid_objects, oids, session_id).await;

        self.update_success_timestamp(!result.is_empty()).await;
        log_operation_completion(session_id, target_address, "GET", &result);

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
        self.log_operation_start("GETNEXT", 1);
        self.increment_connection_attempts().await;

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

        self.update_success_timestamp(!result.is_empty()).await;
        log_operation_completion(self.session_id, self.config.address, "GETNEXT", &result);

        Ok(result)
    }

    /// Log the start of an SNMP operation
    fn log_operation_start(&self, operation: &str, oid_count: usize) {
        debug!(
            session_id = %self.session_id,
            target = %self.config.address,
            operation = operation,
            oid_count = oid_count,
            "Starting SNMP operation"
        );
    }
}

/// Execute GET requests for all OIDs
async fn execute_get_requests(
    client: &csnmp::Snmp2cClient,
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

/// Log the completion of an SNMP operation
fn log_operation_completion(
    session_id: Uuid,
    target_address: std::net::SocketAddr,
    operation: &str,
    result: &HashMap<String, SnmpValue>,
) {
    info!(
        session_id = %session_id,
        target = %target_address,
        operation = operation,
        result_count = result.len(),
        "SNMP operation completed"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snmp::config::{SessionConfig, SnmpCredentials};
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
            max_vars_per_request: 50,
        }
    }

    #[tokio::test]
    async fn test_get_operation_invalid_oid() {
        let config = create_test_config();
        let mut session = SnmpSession::new(config);

        // Test with invalid OID format
        let result = session.get(&["invalid.oid"]).await;
        assert!(result.is_err());

        if let Err(SnmpError::InvalidOid { oid }) = result {
            assert_eq!(oid, "invalid.oid");
        } else {
            panic!("Expected InvalidOid error");
        }
    }

    #[tokio::test]
    async fn test_get_next_operation_invalid_oid() {
        let config = create_test_config();
        let mut session = SnmpSession::new(config);

        // Test with invalid OID format
        let result = session.get_next("invalid.oid").await;
        assert!(result.is_err());

        if let Err(SnmpError::InvalidOid { oid }) = result {
            assert_eq!(oid, "invalid.oid");
        } else {
            panic!("Expected InvalidOid error");
        }
    }

    #[test]
    fn test_log_operation_completion() {
        let session_id = uuid::Uuid::new_v4();
        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161);
        let result = HashMap::new();

        // This should not panic
        log_operation_completion(session_id, target, "GET", &result);
    }
}
