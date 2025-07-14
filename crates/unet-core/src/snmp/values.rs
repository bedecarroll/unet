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
