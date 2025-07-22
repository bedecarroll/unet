//! Utility functions for SNMP session operations

use super::super::values::SnmpValue;
use super::super::{SnmpError, SnmpResult};
use csnmp::{ObjectIdentifier, ObjectValue};

/// Parse OID strings into `ObjectIdentifier` objects
///
/// # Errors
///
/// Returns an error if any OID string cannot be parsed as a valid `ObjectIdentifier`.
pub fn parse_oids(oids: &[&str]) -> SnmpResult<Vec<ObjectIdentifier>> {
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

/// Convert csnmp `ObjectValue` to our `SnmpValue` type
#[must_use]
pub fn convert_object_value_to_snmp_value(value: &ObjectValue) -> SnmpValue {
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
        ObjectValue::Counter64(c) => SnmpValue::Counter64(*c),
        ObjectValue::TimeTicks(t) => SnmpValue::TimeTicks(*t),
        ObjectValue::Opaque(bytes) => SnmpValue::Opaque(bytes.clone()),
        // Handle any other variants as generic values
        ObjectValue::Unsigned32(_) => SnmpValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_parse_oids_valid() {
        let oids = ["1.3.6.1.2.1.1.1.0", "1.3.6.1.2.1.1.2.0"];
        let result = parse_oids(&oids);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_oids_invalid() {
        let oids = ["invalid.oid"];
        let result = parse_oids(&oids);
        assert!(result.is_err());

        if let Err(SnmpError::InvalidOid { oid }) = result {
            assert_eq!(oid, "invalid.oid");
        } else {
            panic!("Expected InvalidOid error");
        }
    }

    #[test]
    fn test_convert_object_value_integer() {
        let value = ObjectValue::Integer(42);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::Integer(42));
    }

    #[test]
    fn test_convert_object_value_string_utf8() {
        let value = ObjectValue::String(b"test".to_vec());
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::String("test".to_string()));
    }

    #[test]
    fn test_convert_object_value_string_non_utf8() {
        let value = ObjectValue::String(vec![0xff, 0xfe]);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::String("0xfffe".to_string()));
    }

    #[test]
    fn test_convert_object_value_ip_address() {
        let ip = Ipv4Addr::new(192, 168, 1, 1);
        let value = ObjectValue::IpAddress(ip);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::IpAddress(std::net::IpAddr::V4(ip)));
    }

    #[test]
    fn test_convert_object_value_counter32() {
        let value = ObjectValue::Counter32(12345);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::Counter32(12345));
    }

    #[test]
    fn test_convert_object_value_counter64() {
        let value = ObjectValue::Counter64(123_456_789);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::Counter64(123_456_789));
    }

    #[test]
    fn test_convert_object_value_opaque() {
        let data = vec![1, 2, 3, 4];
        let value = ObjectValue::Opaque(data.clone());
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::Opaque(data));
    }

    #[test]
    fn test_convert_object_value_time_ticks() {
        let value = ObjectValue::TimeTicks(987_654_321);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::TimeTicks(987_654_321));
    }

    #[test]
    fn test_convert_object_value_object_id() {
        use csnmp::ObjectIdentifier;
        use std::str::FromStr;

        let oid = ObjectIdentifier::from_str("1.3.6.1.2.1.1.1.0").unwrap();
        let value = ObjectValue::ObjectId(oid);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::Oid("1.3.6.1.2.1.1.1.0".to_string()));
    }

    #[test]
    fn test_convert_object_value_unsigned32() {
        let value = ObjectValue::Unsigned32(42);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        // Unsigned32 maps to Null as per the implementation
        assert_eq!(snmp_value, SnmpValue::Null);
    }

    #[test]
    fn test_parse_oids_empty() {
        let oids = [];
        let result = parse_oids(&oids);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_oids_mixed_valid_invalid() {
        let oids = ["1.3.6.1.2.1.1.1.0", "invalid.oid", "1.3.6.1.2.1.1.2.0"];
        let result = parse_oids(&oids);
        assert!(result.is_err());

        // Should fail on first invalid OID
        if let Err(SnmpError::InvalidOid { oid }) = result {
            assert_eq!(oid, "invalid.oid");
        } else {
            panic!("Expected InvalidOid error");
        }
    }

    #[test]
    fn test_convert_object_value_string_empty() {
        let value = ObjectValue::String(Vec::new());
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::String(String::new()));
    }

    #[test]
    fn test_convert_object_value_string_partial_utf8() {
        // Mix valid UTF-8 with invalid bytes
        let mut bytes = b"hello".to_vec();
        bytes.extend_from_slice(&[0xff, 0xfe]); // Invalid UTF-8 bytes

        let value = ObjectValue::String(bytes);
        let snmp_value = convert_object_value_to_snmp_value(&value);
        // Should convert to hex since it's not valid UTF-8
        assert_eq!(
            snmp_value,
            SnmpValue::String("0x68656c6c6ffffe".to_string())
        );
    }

    #[test]
    fn test_convert_object_value_opaque_empty() {
        let value = ObjectValue::Opaque(Vec::new());
        let snmp_value = convert_object_value_to_snmp_value(&value);
        assert_eq!(snmp_value, SnmpValue::Opaque(Vec::new()));
    }

    #[test]
    fn test_parse_oids_long_list() {
        let oids = [
            "1.3.6.1.2.1.1.1.0",
            "1.3.6.1.2.1.1.2.0",
            "1.3.6.1.2.1.1.3.0",
            "1.3.6.1.2.1.1.4.0",
            "1.3.6.1.2.1.1.5.0",
            "1.3.6.1.2.1.2.1.0",
            "1.3.6.1.2.1.2.2.1.1.1",
            "1.3.6.1.2.1.2.2.1.2.1",
        ];
        let result = parse_oids(&oids);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 8);
    }
}
