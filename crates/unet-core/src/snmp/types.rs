//! SNMP data types

pub use super::{SnmpError, SnmpResult, SnmpValue};

/// SNMP data type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnmpType {
    /// Integer
    Integer,
    /// Octet string
    OctetString,
    /// Object identifier
    ObjectIdentifier,
    /// IP address
    IpAddress,
    /// Counter (32-bit)
    Counter32,
    /// Counter (64-bit)
    Counter64,
    /// Gauge (32-bit)
    Gauge32,
    /// Time ticks
    TimeTicks,
    /// Opaque
    Opaque,
    /// Null
    Null,
}

impl SnmpType {
    /// Get string representation of the SNMP type
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Integer => "INTEGER",
            Self::OctetString => "OCTET STRING",
            Self::ObjectIdentifier => "OBJECT IDENTIFIER",
            Self::IpAddress => "IpAddress",
            Self::Counter32 => "Counter32",
            Self::Counter64 => "Counter64",
            Self::Gauge32 => "Gauge32",
            Self::TimeTicks => "TimeTicks",
            Self::Opaque => "Opaque",
            Self::Null => "NULL",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snmp_type_as_str() {
        assert_eq!(SnmpType::Integer.as_str(), "INTEGER");
        assert_eq!(SnmpType::OctetString.as_str(), "OCTET STRING");
        assert_eq!(SnmpType::ObjectIdentifier.as_str(), "OBJECT IDENTIFIER");
        assert_eq!(SnmpType::IpAddress.as_str(), "IpAddress");
        assert_eq!(SnmpType::Counter32.as_str(), "Counter32");
        assert_eq!(SnmpType::Counter64.as_str(), "Counter64");
        assert_eq!(SnmpType::Gauge32.as_str(), "Gauge32");
        assert_eq!(SnmpType::TimeTicks.as_str(), "TimeTicks");
        assert_eq!(SnmpType::Opaque.as_str(), "Opaque");
        assert_eq!(SnmpType::Null.as_str(), "NULL");
    }

    #[test]
    fn test_snmp_type_clone() {
        let snmp_type = SnmpType::Integer;
        let cloned = snmp_type;
        assert_eq!(snmp_type, cloned);
    }

    #[test]
    fn test_snmp_type_copy() {
        let snmp_type = SnmpType::Counter32;
        let copied = snmp_type;
        assert_eq!(snmp_type, copied);
    }

    #[test]
    fn test_snmp_type_eq() {
        assert_eq!(SnmpType::Integer, SnmpType::Integer);
        assert_ne!(SnmpType::Integer, SnmpType::OctetString);
    }

    #[test]
    fn test_snmp_type_debug() {
        let snmp_type = SnmpType::Gauge32;
        let debug_str = format!("{snmp_type:?}");
        assert!(debug_str.contains("Gauge32"));
    }

    #[test]
    fn test_all_snmp_types() {
        let types = vec![
            SnmpType::Integer,
            SnmpType::OctetString,
            SnmpType::ObjectIdentifier,
            SnmpType::IpAddress,
            SnmpType::Counter32,
            SnmpType::Counter64,
            SnmpType::Gauge32,
            SnmpType::TimeTicks,
            SnmpType::Opaque,
            SnmpType::Null,
        ];

        for snmp_type in types {
            // Test that each type has a string representation
            assert!(!snmp_type.as_str().is_empty());

            // Test that each type is equal to itself
            assert_eq!(snmp_type, snmp_type);
        }
    }
}
