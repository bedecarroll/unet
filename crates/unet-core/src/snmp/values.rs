//! SNMP value types and conversions

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// SNMP value types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnmpValue {
    /// Integer value
    Integer(i64),
    /// String value  
    String(String),
    /// Object identifier
    Oid(String),
    /// IP address
    IpAddress(IpAddr),
    /// Counter (32-bit)
    Counter32(u32),
    /// Counter (64-bit)
    Counter64(u64),
    /// Gauge (32-bit)
    Gauge32(u32),
    /// Time ticks
    TimeTicks(u32),
    /// Opaque data
    Opaque(Vec<u8>),
    /// Null value
    Null,
    /// No such object
    NoSuchObject,
    /// No such instance
    NoSuchInstance,
    /// End of MIB view
    EndOfMibView,
}

impl SnmpValue {
    /// Check if value represents an error condition
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(
            self,
            Self::NoSuchObject | Self::NoSuchInstance | Self::EndOfMibView
        )
    }
}

impl std::fmt::Display for SnmpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{i}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Oid(oid) => write!(f, "{oid}"),
            Self::IpAddress(ip) => write!(f, "{ip}"),
            Self::Counter32(c) => write!(f, "{c}"),
            Self::Counter64(c) => write!(f, "{c}"),
            Self::Gauge32(g) => write!(f, "{g}"),
            Self::TimeTicks(t) => write!(f, "{t}"),
            Self::Opaque(data) => write!(f, "Opaque({} bytes)", data.len()),
            Self::Null => write!(f, "null"),
            Self::NoSuchObject => write!(f, "noSuchObject"),
            Self::NoSuchInstance => write!(f, "noSuchInstance"),
            Self::EndOfMibView => write!(f, "endOfMibView"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[test]
    fn test_snmp_value_is_error() {
        // Error values
        assert!(SnmpValue::NoSuchObject.is_error());
        assert!(SnmpValue::NoSuchInstance.is_error());
        assert!(SnmpValue::EndOfMibView.is_error());

        // Non-error values
        assert!(!SnmpValue::Integer(42).is_error());
        assert!(!SnmpValue::String("test".to_string()).is_error());
        assert!(!SnmpValue::Null.is_error());
        assert!(!SnmpValue::Counter32(100).is_error());
        assert!(!SnmpValue::Counter64(1000).is_error());
        assert!(!SnmpValue::Gauge32(50).is_error());
        assert!(!SnmpValue::TimeTicks(12345).is_error());
        assert!(!SnmpValue::Opaque(vec![1, 2, 3]).is_error());
        assert!(!SnmpValue::Oid("1.3.6.1.2.1.1.1.0".to_string()).is_error());
        assert!(!SnmpValue::IpAddress("192.168.1.1".parse::<IpAddr>().unwrap()).is_error());
    }

    #[test]
    fn test_snmp_value_display() {
        assert_eq!(SnmpValue::Integer(42).to_string(), "42");
        assert_eq!(SnmpValue::String("test".to_string()).to_string(), "test");
        assert_eq!(
            SnmpValue::Oid("1.3.6.1.2.1.1.1.0".to_string()).to_string(),
            "1.3.6.1.2.1.1.1.0"
        );
        assert_eq!(
            SnmpValue::IpAddress("192.168.1.1".parse().unwrap()).to_string(),
            "192.168.1.1"
        );
        assert_eq!(SnmpValue::Counter32(100).to_string(), "100");
        assert_eq!(SnmpValue::Counter64(1000).to_string(), "1000");
        assert_eq!(SnmpValue::Gauge32(50).to_string(), "50");
        assert_eq!(SnmpValue::TimeTicks(12345).to_string(), "12345");
        assert_eq!(
            SnmpValue::Opaque(vec![1, 2, 3]).to_string(),
            "Opaque(3 bytes)"
        );
        assert_eq!(SnmpValue::Null.to_string(), "null");
        assert_eq!(SnmpValue::NoSuchObject.to_string(), "noSuchObject");
        assert_eq!(SnmpValue::NoSuchInstance.to_string(), "noSuchInstance");
        assert_eq!(SnmpValue::EndOfMibView.to_string(), "endOfMibView");
    }

    #[test]
    fn test_snmp_value_equality() {
        assert_eq!(SnmpValue::Integer(42), SnmpValue::Integer(42));
        assert_ne!(SnmpValue::Integer(42), SnmpValue::Integer(43));
        assert_eq!(
            SnmpValue::String("test".to_string()),
            SnmpValue::String("test".to_string())
        );
        assert_ne!(
            SnmpValue::String("test".to_string()),
            SnmpValue::String("other".to_string())
        );
    }

    #[test]
    fn test_snmp_value_serialization() {
        let value = SnmpValue::Integer(42);
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: SnmpValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_all_snmp_value_variants_serialization() {
        let values = vec![
            SnmpValue::Integer(42),
            SnmpValue::String("test".to_string()),
            SnmpValue::Oid("1.3.6.1.2.1.1.1.0".to_string()),
            SnmpValue::IpAddress("192.168.1.1".parse().unwrap()),
            SnmpValue::Counter32(100),
            SnmpValue::Counter64(1000),
            SnmpValue::Gauge32(50),
            SnmpValue::TimeTicks(12345),
            SnmpValue::Opaque(vec![1, 2, 3]),
            SnmpValue::Null,
            SnmpValue::NoSuchObject,
            SnmpValue::NoSuchInstance,
            SnmpValue::EndOfMibView,
        ];

        for value in values {
            let serialized = serde_json::to_string(&value).unwrap();
            let deserialized: SnmpValue = serde_json::from_str(&serialized).unwrap();
            assert_eq!(value, deserialized);
        }
    }

    #[test]
    fn test_snmp_value_clone() {
        let original = SnmpValue::String("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let original = SnmpValue::Opaque(vec![1, 2, 3]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_snmp_value_debug_format() {
        let value = SnmpValue::Integer(42);
        let debug_str = format!("{value:?}");
        assert!(debug_str.contains("Integer"));
        assert!(debug_str.contains("42"));

        let value = SnmpValue::NoSuchObject;
        let debug_str = format!("{value:?}");
        assert!(debug_str.contains("NoSuchObject"));
    }

    #[test]
    fn test_snmp_value_large_numbers() {
        // Test edge cases with large numbers
        let max_u32 = SnmpValue::Counter32(u32::MAX);
        assert_eq!(max_u32.to_string(), u32::MAX.to_string());
        assert!(!max_u32.is_error());

        let max_u64 = SnmpValue::Counter64(u64::MAX);
        assert_eq!(max_u64.to_string(), u64::MAX.to_string());
        assert!(!max_u64.is_error());

        let min_i64 = SnmpValue::Integer(i64::MIN);
        assert_eq!(min_i64.to_string(), i64::MIN.to_string());
        assert!(!min_i64.is_error());
    }

    #[test]
    fn test_snmp_value_empty_strings() {
        let empty_string = SnmpValue::String(String::new());
        assert_eq!(empty_string.to_string(), "");
        assert!(!empty_string.is_error());

        let empty_oid = SnmpValue::Oid(String::new());
        assert_eq!(empty_oid.to_string(), "");
        assert!(!empty_oid.is_error());
    }

    #[test]
    fn test_snmp_value_opaque_edge_cases() {
        // Empty opaque data
        let empty_opaque = SnmpValue::Opaque(Vec::new());
        assert_eq!(empty_opaque.to_string(), "Opaque(0 bytes)");
        assert!(!empty_opaque.is_error());

        // Large opaque data
        let large_opaque = SnmpValue::Opaque(vec![0; 1000]);
        assert_eq!(large_opaque.to_string(), "Opaque(1000 bytes)");
        assert!(!large_opaque.is_error());
    }

    #[test]
    fn test_snmp_value_ipv6_address() {
        let ipv6: IpAddr = "2001:db8::1".parse().unwrap();
        let value = SnmpValue::IpAddress(ipv6);
        assert_eq!(value.to_string(), "2001:db8::1");
        assert!(!value.is_error());
    }

    #[test]
    fn test_snmp_value_comparison_different_types() {
        // Test that different numeric types with same value are not equal
        let counter32 = SnmpValue::Counter32(42);
        let counter64 = SnmpValue::Counter64(42);
        let gauge32 = SnmpValue::Gauge32(42);
        let integer = SnmpValue::Integer(42);
        let timeticks = SnmpValue::TimeTicks(42);

        assert_ne!(counter32, counter64);
        assert_ne!(counter32, gauge32);
        assert_ne!(counter32, integer);
        assert_ne!(counter32, timeticks);
        assert_ne!(counter64, gauge32);
        assert_ne!(counter64, integer);
        assert_ne!(gauge32, integer);
    }
}
