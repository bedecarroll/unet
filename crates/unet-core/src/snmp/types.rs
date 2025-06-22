//! SNMP data types

pub use super::{SnmpValue, SnmpError, SnmpResult};

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
    pub fn as_str(&self) -> &'static str {
        match self {
            SnmpType::Integer => "INTEGER",
            SnmpType::OctetString => "OCTET STRING",
            SnmpType::ObjectIdentifier => "OBJECT IDENTIFIER",
            SnmpType::IpAddress => "IpAddress",
            SnmpType::Counter32 => "Counter32",
            SnmpType::Counter64 => "Counter64",
            SnmpType::Gauge32 => "Gauge32",
            SnmpType::TimeTicks => "TimeTicks",
            SnmpType::Opaque => "Opaque",
            SnmpType::Null => "NULL",
        }
    }
}