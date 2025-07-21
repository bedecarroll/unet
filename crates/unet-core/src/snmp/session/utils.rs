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
}
