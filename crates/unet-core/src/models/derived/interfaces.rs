//! Interface status and statistics for network devices

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::snmp::SnmpValue;

/// Status of a network interface
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceStatus {
    /// Interface index
    pub index: u32,
    /// Interface name/description
    pub name: String,
    /// Interface type (see RFC 1213)
    pub interface_type: u32,
    /// Interface MTU
    pub mtu: Option<u32>,
    /// Interface speed in bits per second
    pub speed: Option<u64>,
    /// Physical address (MAC address)
    pub physical_address: Option<String>,
    /// Administrative status
    pub admin_status: InterfaceAdminStatus,
    /// Operational status
    pub oper_status: InterfaceOperStatus,
    /// Last change timestamp (in ticks)
    pub last_change: Option<u32>,
    /// Input statistics
    pub input_stats: InterfaceStats,
    /// Output statistics
    pub output_stats: InterfaceStats,
}

impl InterfaceStatus {
    /// Extract interface statuses from SNMP data
    #[must_use]
    pub fn from_snmp(snmp_data: &HashMap<String, SnmpValue>) -> Vec<Self> {
        let mut interfaces = Vec::new();
        let mut interface_indexes = std::collections::BTreeSet::new();

        // First, collect all interface indexes
        for oid in snmp_data.keys() {
            if oid.starts_with("1.3.6.1.2.1.2.2.1.1.") {
                if let Some(index_str) = oid.strip_prefix("1.3.6.1.2.1.2.2.1.1.") {
                    if let Ok(index) = index_str.parse::<u32>() {
                        interface_indexes.insert(index);
                    }
                }
            }
        }

        // For each interface index, extract all available data
        for index in interface_indexes {
            let mut interface = Self {
                index,
                name: format!("Interface {index}"),
                interface_type: 1, // Default to other
                mtu: None,
                speed: None,
                physical_address: None,
                admin_status: InterfaceAdminStatus::Unknown,
                oper_status: InterfaceOperStatus::Unknown,
                last_change: None,
                input_stats: InterfaceStats::default(),
                output_stats: InterfaceStats::default(),
            };

            // Extract interface description
            if let Some(SnmpValue::String(desc)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.2.{index}"))
            {
                interface.name.clone_from(desc);
            }

            // Extract interface type
            if let Some(SnmpValue::Integer(iface_type)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.3.{index}"))
            {
                if let Ok(iface_type_u32) = u32::try_from(*iface_type) {
                    interface.interface_type = iface_type_u32;
                }
            }

            // Extract MTU
            if let Some(SnmpValue::Integer(mtu)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.4.{index}"))
            {
                if let Ok(mtu_u32) = u32::try_from(*mtu) {
                    interface.mtu = Some(mtu_u32);
                }
            }

            // Extract speed
            if let Some(SnmpValue::Gauge32(speed)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.5.{index}"))
            {
                interface.speed = Some(u64::from(*speed));
            }

            // Extract administrative status
            if let Some(SnmpValue::Integer(status)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.7.{index}"))
            {
                if let Ok(status_u8) = u8::try_from(*status) {
                    interface.admin_status = InterfaceAdminStatus::from(status_u8);
                }
            }

            // Extract operational status
            if let Some(SnmpValue::Integer(status)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.8.{index}"))
            {
                if let Ok(status_u8) = u8::try_from(*status) {
                    interface.oper_status = InterfaceOperStatus::from(status_u8);
                }
            }

            // Extract input octets
            if let Some(SnmpValue::Counter32(octets)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.10.{index}"))
            {
                interface.input_stats.octets = u64::from(*octets);
            }

            // Extract input packets
            if let Some(SnmpValue::Counter32(packets)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.11.{index}"))
            {
                interface.input_stats.packets = u64::from(*packets);
            }

            // Extract input errors
            if let Some(SnmpValue::Counter32(errors)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.14.{index}"))
            {
                interface.input_stats.errors = u64::from(*errors);
            }

            // Extract output octets
            if let Some(SnmpValue::Counter32(octets)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.16.{index}"))
            {
                interface.output_stats.octets = u64::from(*octets);
            }

            // Extract output packets
            if let Some(SnmpValue::Counter32(packets)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.17.{index}"))
            {
                interface.output_stats.packets = u64::from(*packets);
            }

            // Extract output errors
            if let Some(SnmpValue::Counter32(errors)) =
                snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.20.{index}"))
            {
                interface.output_stats.errors = u64::from(*errors);
            }

            interfaces.push(interface);
        }

        interfaces
    }
}

/// Interface administrative status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceAdminStatus {
    /// Interface is administratively up
    Up,
    /// Interface is administratively down
    Down,
    /// Interface is in testing mode
    Testing,
    /// Unknown status
    Unknown,
}

impl From<u8> for InterfaceAdminStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Up,
            2 => Self::Down,
            3 => Self::Testing,
            _ => Self::Unknown,
        }
    }
}

/// Interface operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceOperStatus {
    /// Interface is operationally up
    Up,
    /// Interface is operationally down
    Down,
    /// Interface is in testing mode
    Testing,
    /// Status is unknown
    Unknown,
    /// Interface is dormant
    Dormant,
    /// Interface is not present
    NotPresent,
    /// Lower layer is down
    LowerLayerDown,
}

impl From<u8> for InterfaceOperStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Up,
            2 => Self::Down,
            3 => Self::Testing,
            5 => Self::Dormant,
            6 => Self::NotPresent,
            7 => Self::LowerLayerDown,
            _ => Self::Unknown,
        }
    }
}

/// Interface traffic statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InterfaceStats {
    /// Number of octets (bytes)
    pub octets: u64,
    /// Number of packets
    pub packets: u64,
    /// Number of errors
    pub errors: u64,
    /// Number of discarded packets
    pub discards: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_interface_status_from_snmp() {
        let mut snmp_data = HashMap::new();

        // Add interface 1 data
        snmp_data.insert("1.3.6.1.2.1.2.2.1.1.1".to_string(), SnmpValue::Integer(1));
        snmp_data.insert(
            "1.3.6.1.2.1.2.2.1.2.1".to_string(),
            SnmpValue::String("eth0".to_string()),
        );
        snmp_data.insert("1.3.6.1.2.1.2.2.1.3.1".to_string(), SnmpValue::Integer(6));
        snmp_data.insert("1.3.6.1.2.1.2.2.1.7.1".to_string(), SnmpValue::Integer(1)); // admin up
        snmp_data.insert("1.3.6.1.2.1.2.2.1.8.1".to_string(), SnmpValue::Integer(1)); // oper up

        let interfaces = InterfaceStatus::from_snmp(&snmp_data);
        assert_eq!(interfaces.len(), 1);

        let interface = &interfaces[0];
        assert_eq!(interface.index, 1);
        assert_eq!(interface.name, "eth0");
        assert_eq!(interface.admin_status, InterfaceAdminStatus::Up);
        assert_eq!(interface.oper_status, InterfaceOperStatus::Up);
    }

    #[test]
    fn test_interface_admin_status_conversion() {
        assert_eq!(InterfaceAdminStatus::from(1), InterfaceAdminStatus::Up);
        assert_eq!(InterfaceAdminStatus::from(2), InterfaceAdminStatus::Down);
        assert_eq!(InterfaceAdminStatus::from(3), InterfaceAdminStatus::Testing);
        assert_eq!(
            InterfaceAdminStatus::from(255),
            InterfaceAdminStatus::Unknown
        );
    }

    #[test]
    fn test_interface_oper_status_conversion() {
        assert_eq!(InterfaceOperStatus::from(1), InterfaceOperStatus::Up);
        assert_eq!(InterfaceOperStatus::from(2), InterfaceOperStatus::Down);
        assert_eq!(
            InterfaceOperStatus::from(7),
            InterfaceOperStatus::LowerLayerDown
        );
        assert_eq!(InterfaceOperStatus::from(255), InterfaceOperStatus::Unknown);
    }
}
