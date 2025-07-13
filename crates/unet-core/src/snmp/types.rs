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
