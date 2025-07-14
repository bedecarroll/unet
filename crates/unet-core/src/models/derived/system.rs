//! System information derived from SNMP system group

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::snmp::SnmpValue;

/// System information derived from SNMP system group
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemInfo {
    /// System description (sysDescr)
    pub description: Option<String>,
    /// System object identifier (sysObjectID)
    pub object_id: Option<String>,
    /// System uptime in ticks (sysUpTime)
    pub uptime_ticks: Option<u32>,
    /// System contact (sysContact)
    pub contact: Option<String>,
    /// System name (sysName)
    pub name: Option<String>,
    /// System location (sysLocation)
    pub location: Option<String>,
    /// System services (sysServices)
    pub services: Option<u32>,
}

impl SystemInfo {
    /// Extract system information from SNMP data
    #[must_use]
    pub fn from_snmp(snmp_data: &HashMap<String, SnmpValue>) -> Option<Self> {
        let mut system_info = Self {
            description: None,
            object_id: None,
            uptime_ticks: None,
            contact: None,
            name: None,
            location: None,
            services: None,
        };

        let mut has_data = false;

        // Extract system description
        if let Some(SnmpValue::String(desc)) = snmp_data.get("1.3.6.1.2.1.1.1.0") {
            system_info.description = Some(desc.clone());
            has_data = true;
        }

        // Extract system object ID
        if let Some(SnmpValue::Oid(oid)) = snmp_data.get("1.3.6.1.2.1.1.2.0") {
            system_info.object_id = Some(oid.clone());
            has_data = true;
        }

        // Extract system uptime
        if let Some(SnmpValue::TimeTicks(ticks)) = snmp_data.get("1.3.6.1.2.1.1.3.0") {
            system_info.uptime_ticks = Some(*ticks);
            has_data = true;
        }

        // Extract system contact
        if let Some(SnmpValue::String(contact)) = snmp_data.get("1.3.6.1.2.1.1.4.0") {
            system_info.contact = Some(contact.clone());
            has_data = true;
        }

        // Extract system name
        if let Some(SnmpValue::String(name)) = snmp_data.get("1.3.6.1.2.1.1.5.0") {
            system_info.name = Some(name.clone());
            has_data = true;
        }

        // Extract system location
        if let Some(SnmpValue::String(location)) = snmp_data.get("1.3.6.1.2.1.1.6.0") {
            system_info.location = Some(location.clone());
            has_data = true;
        }

        // Extract system services
        if let Some(SnmpValue::Integer(services)) = snmp_data.get("1.3.6.1.2.1.1.7.0") {
            if let Ok(services_u32) = u32::try_from(*services) {
                system_info.services = Some(services_u32);
                has_data = true;
            }
        }

        if has_data { Some(system_info) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_system_info_from_snmp() {
        let mut snmp_data = HashMap::new();
        snmp_data.insert(
            "1.3.6.1.2.1.1.1.0".to_string(),
            SnmpValue::String("Test Device v1.0".to_string()),
        );
        snmp_data.insert(
            "1.3.6.1.2.1.1.3.0".to_string(),
            SnmpValue::TimeTicks(12_345_678),
        );
        snmp_data.insert(
            "1.3.6.1.2.1.1.5.0".to_string(),
            SnmpValue::String("test-router".to_string()),
        );

        let system_info = SystemInfo::from_snmp(&snmp_data);
        assert!(system_info.is_some());

        let info = system_info.unwrap();
        assert_eq!(info.description, Some("Test Device v1.0".to_string()));
        assert_eq!(info.uptime_ticks, Some(12_345_678));
        assert_eq!(info.name, Some("test-router".to_string()));
    }
}
