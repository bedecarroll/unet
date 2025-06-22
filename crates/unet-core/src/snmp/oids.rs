//! Standard and vendor-specific OID definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard SNMP OIDs commonly used for network device monitoring
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StandardOid {
    /// System description (1.3.6.1.2.1.1.1.0)
    SysDescr,
    /// System object identifier (1.3.6.1.2.1.1.2.0)
    SysObjectId,
    /// System uptime (1.3.6.1.2.1.1.3.0)
    SysUpTime,
    /// System contact (1.3.6.1.2.1.1.4.0)
    SysContact,
    /// System name (1.3.6.1.2.1.1.5.0)
    SysName,
    /// System location (1.3.6.1.2.1.1.6.0)
    SysLocation,
    /// System services (1.3.6.1.2.1.1.7.0)
    SysServices,
    /// Interface count (1.3.6.1.2.1.2.1.0)
    IfNumber,
    /// Interface table base (1.3.6.1.2.1.2.2.1)
    IfTable,
    /// Interface index (1.3.6.1.2.1.2.2.1.1)
    IfIndex,
    /// Interface description (1.3.6.1.2.1.2.2.1.2)
    IfDescr,
    /// Interface type (1.3.6.1.2.1.2.2.1.3)
    IfType,
    /// Interface MTU (1.3.6.1.2.1.2.2.1.4)
    IfMtu,
    /// Interface speed (1.3.6.1.2.1.2.2.1.5)
    IfSpeed,
    /// Interface physical address (1.3.6.1.2.1.2.2.1.6)
    IfPhysAddress,
    /// Interface admin status (1.3.6.1.2.1.2.2.1.7)
    IfAdminStatus,
    /// Interface operational status (1.3.6.1.2.1.2.2.1.8)
    IfOperStatus,
    /// Interface last change (1.3.6.1.2.1.2.2.1.9)
    IfLastChange,
    /// Interface input octets (1.3.6.1.2.1.2.2.1.10)
    IfInOctets,
    /// Interface input unicast packets (1.3.6.1.2.1.2.2.1.11)
    IfInUcastPkts,
    /// Interface input errors (1.3.6.1.2.1.2.2.1.14)
    IfInErrors,
    /// Interface output octets (1.3.6.1.2.1.2.2.1.16)
    IfOutOctets,
    /// Interface output unicast packets (1.3.6.1.2.1.2.2.1.17)
    IfOutUcastPkts,
    /// Interface output errors (1.3.6.1.2.1.2.2.1.20)
    IfOutErrors,
}

impl StandardOid {
    /// Get the OID string for this standard OID
    pub fn oid(&self) -> &'static str {
        match self {
            StandardOid::SysDescr => "1.3.6.1.2.1.1.1.0",
            StandardOid::SysObjectId => "1.3.6.1.2.1.1.2.0",
            StandardOid::SysUpTime => "1.3.6.1.2.1.1.3.0",
            StandardOid::SysContact => "1.3.6.1.2.1.1.4.0",
            StandardOid::SysName => "1.3.6.1.2.1.1.5.0",
            StandardOid::SysLocation => "1.3.6.1.2.1.1.6.0",
            StandardOid::SysServices => "1.3.6.1.2.1.1.7.0",
            StandardOid::IfNumber => "1.3.6.1.2.1.2.1.0",
            StandardOid::IfTable => "1.3.6.1.2.1.2.2.1",
            StandardOid::IfIndex => "1.3.6.1.2.1.2.2.1.1",
            StandardOid::IfDescr => "1.3.6.1.2.1.2.2.1.2",
            StandardOid::IfType => "1.3.6.1.2.1.2.2.1.3",
            StandardOid::IfMtu => "1.3.6.1.2.1.2.2.1.4",
            StandardOid::IfSpeed => "1.3.6.1.2.1.2.2.1.5",
            StandardOid::IfPhysAddress => "1.3.6.1.2.1.2.2.1.6",
            StandardOid::IfAdminStatus => "1.3.6.1.2.1.2.2.1.7",
            StandardOid::IfOperStatus => "1.3.6.1.2.1.2.2.1.8",
            StandardOid::IfLastChange => "1.3.6.1.2.1.2.2.1.9",
            StandardOid::IfInOctets => "1.3.6.1.2.1.2.2.1.10",
            StandardOid::IfInUcastPkts => "1.3.6.1.2.1.2.2.1.11",
            StandardOid::IfInErrors => "1.3.6.1.2.1.2.2.1.14",
            StandardOid::IfOutOctets => "1.3.6.1.2.1.2.2.1.16",
            StandardOid::IfOutUcastPkts => "1.3.6.1.2.1.2.2.1.17",
            StandardOid::IfOutErrors => "1.3.6.1.2.1.2.2.1.20",
        }
    }

    /// Get description of this OID
    pub fn description(&self) -> &'static str {
        match self {
            StandardOid::SysDescr => "System description",
            StandardOid::SysObjectId => "System object identifier",
            StandardOid::SysUpTime => "System uptime in hundredths of seconds",
            StandardOid::SysContact => "System contact information",
            StandardOid::SysName => "System name",
            StandardOid::SysLocation => "System location",
            StandardOid::SysServices => "System services",
            StandardOid::IfNumber => "Number of network interfaces",
            StandardOid::IfTable => "Network interface table",
            StandardOid::IfIndex => "Interface index",
            StandardOid::IfDescr => "Interface description",
            StandardOid::IfType => "Interface type",
            StandardOid::IfMtu => "Interface MTU",
            StandardOid::IfSpeed => "Interface speed in bits per second",
            StandardOid::IfPhysAddress => "Interface physical address",
            StandardOid::IfAdminStatus => "Interface administrative status",
            StandardOid::IfOperStatus => "Interface operational status",
            StandardOid::IfLastChange => "Interface last change time",
            StandardOid::IfInOctets => "Interface input octets",
            StandardOid::IfInUcastPkts => "Interface input unicast packets",
            StandardOid::IfInErrors => "Interface input errors",
            StandardOid::IfOutOctets => "Interface output octets",
            StandardOid::IfOutUcastPkts => "Interface output unicast packets",
            StandardOid::IfOutErrors => "Interface output errors",
        }
    }

    /// Get all standard system OIDs for basic device information
    pub fn system_oids() -> Vec<StandardOid> {
        vec![
            StandardOid::SysDescr,
            StandardOid::SysObjectId,
            StandardOid::SysUpTime,
            StandardOid::SysContact,
            StandardOid::SysName,
            StandardOid::SysLocation,
            StandardOid::SysServices,
        ]
    }

    /// Get all interface table OIDs for interface monitoring
    pub fn interface_oids() -> Vec<StandardOid> {
        vec![
            StandardOid::IfNumber,
            StandardOid::IfIndex,
            StandardOid::IfDescr,
            StandardOid::IfType,
            StandardOid::IfMtu,
            StandardOid::IfSpeed,
            StandardOid::IfPhysAddress,
            StandardOid::IfAdminStatus,
            StandardOid::IfOperStatus,
            StandardOid::IfLastChange,
            StandardOid::IfInOctets,
            StandardOid::IfInUcastPkts,
            StandardOid::IfInErrors,
            StandardOid::IfOutOctets,
            StandardOid::IfOutUcastPkts,
            StandardOid::IfOutErrors,
        ]
    }
}

/// Vendor-specific OIDs for extended device information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VendorOid {
    /// Cisco enterprise OIDs
    Cisco {
        /// Specific Cisco OID
        oid: String,
        /// Description of what this OID represents
        description: String,
    },
    /// Juniper enterprise OIDs
    Juniper {
        /// Specific Juniper OID
        oid: String,
        /// Description of what this OID represents
        description: String,
    },
    /// Arista enterprise OIDs
    Arista {
        /// Specific Arista OID
        oid: String,
        /// Description of what this OID represents
        description: String,
    },
    /// Generic vendor OID
    Generic {
        /// Vendor name
        vendor: String,
        /// Specific OID
        oid: String,
        /// Description of what this OID represents
        description: String,
    },
}

impl VendorOid {
    /// Get the OID string
    pub fn oid(&self) -> &str {
        match self {
            VendorOid::Cisco { oid, .. } => oid,
            VendorOid::Juniper { oid, .. } => oid,
            VendorOid::Arista { oid, .. } => oid,
            VendorOid::Generic { oid, .. } => oid,
        }
    }

    /// Get the description
    pub fn description(&self) -> &str {
        match self {
            VendorOid::Cisco { description, .. } => description,
            VendorOid::Juniper { description, .. } => description,
            VendorOid::Arista { description, .. } => description,
            VendorOid::Generic { description, .. } => description,
        }
    }

    /// Get the vendor name
    pub fn vendor(&self) -> &str {
        match self {
            VendorOid::Cisco { .. } => "Cisco",
            VendorOid::Juniper { .. } => "Juniper",
            VendorOid::Arista { .. } => "Arista",
            VendorOid::Generic { vendor, .. } => vendor,
        }
    }

    /// Create common Cisco OIDs
    pub fn cisco_common() -> Vec<VendorOid> {
        vec![
            VendorOid::Cisco {
                oid: "1.3.6.1.4.1.9.2.1.3.0".to_string(),
                description: "Cisco CPU utilization".to_string(),
            },
            VendorOid::Cisco {
                oid: "1.3.6.1.4.1.9.2.1.8.0".to_string(),
                description: "Cisco memory utilization".to_string(),
            },
            VendorOid::Cisco {
                oid: "1.3.6.1.4.1.9.9.13.1.3.1.3".to_string(),
                description: "Cisco temperature sensor".to_string(),
            },
        ]
    }

    /// Create common Juniper OIDs
    pub fn juniper_common() -> Vec<VendorOid> {
        vec![
            VendorOid::Juniper {
                oid: "1.3.6.1.4.1.2636.3.1.13.1.8".to_string(),
                description: "Juniper CPU utilization".to_string(),
            },
            VendorOid::Juniper {
                oid: "1.3.6.1.4.1.2636.3.1.13.1.11".to_string(),
                description: "Juniper memory utilization".to_string(),
            },
            VendorOid::Juniper {
                oid: "1.3.6.1.4.1.2636.3.1.13.1.7".to_string(),
                description: "Juniper temperature".to_string(),
            },
        ]
    }
}

/// OID mapping utility for organizing and accessing OIDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidMap {
    /// Standard OIDs mapped by name
    standard: HashMap<String, StandardOid>,
    /// Vendor-specific OIDs mapped by name
    vendor: HashMap<String, VendorOid>,
    /// Custom OIDs defined by user
    custom: HashMap<String, String>,
}

impl Default for OidMap {
    fn default() -> Self {
        let mut map = Self {
            standard: HashMap::new(),
            vendor: HashMap::new(),
            custom: HashMap::new(),
        };

        // Populate with all standard OIDs
        for oid in StandardOid::system_oids() {
            map.standard.insert(format!("{:?}", oid), oid);
        }
        for oid in StandardOid::interface_oids() {
            map.standard.insert(format!("{:?}", oid), oid);
        }

        // Populate with common vendor OIDs
        for oid in VendorOid::cisco_common() {
            map.vendor.insert(
                format!("Cisco_{}", oid.description().replace(' ', "_")),
                oid,
            );
        }
        for oid in VendorOid::juniper_common() {
            map.vendor.insert(
                format!("Juniper_{}", oid.description().replace(' ', "_")),
                oid,
            );
        }

        map
    }
}

impl OidMap {
    /// Create new empty OID map
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a standard OID to the map
    pub fn add_standard(&mut self, name: String, oid: StandardOid) {
        self.standard.insert(name, oid);
    }

    /// Add a vendor OID to the map  
    pub fn add_vendor(&mut self, name: String, oid: VendorOid) {
        self.vendor.insert(name, oid);
    }

    /// Add a custom OID to the map
    pub fn add_custom(&mut self, name: String, oid: String) {
        self.custom.insert(name, oid);
    }

    /// Get standard OID by name
    pub fn get_standard(&self, name: &str) -> Option<&StandardOid> {
        self.standard.get(name)
    }

    /// Get vendor OID by name
    pub fn get_vendor(&self, name: &str) -> Option<&VendorOid> {
        self.vendor.get(name)
    }

    /// Get custom OID by name
    pub fn get_custom(&self, name: &str) -> Option<&str> {
        self.custom.get(name).map(|s| s.as_str())
    }

    /// Resolve any OID name to its string representation
    pub fn resolve(&self, name: &str) -> Option<String> {
        if let Some(oid) = self.get_standard(name) {
            Some(oid.oid().to_string())
        } else if let Some(oid) = self.get_vendor(name) {
            Some(oid.oid().to_string())
        } else if let Some(oid) = self.get_custom(name) {
            Some(oid.to_string())
        } else {
            None
        }
    }

    /// Get all OID names in the map
    pub fn list_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.standard.keys().cloned());
        names.extend(self.vendor.keys().cloned());
        names.extend(self.custom.keys().cloned());
        names.sort();
        names
    }

    /// Get all standard OID names
    pub fn list_standard(&self) -> Vec<String> {
        let mut names: Vec<String> = self.standard.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all vendor OID names  
    pub fn list_vendor(&self) -> Vec<String> {
        let mut names: Vec<String> = self.vendor.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all custom OID names
    pub fn list_custom(&self) -> Vec<String> {
        let mut names: Vec<String> = self.custom.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_oid_basic() {
        assert_eq!(StandardOid::SysDescr.oid(), "1.3.6.1.2.1.1.1.0");
        assert_eq!(StandardOid::SysName.oid(), "1.3.6.1.2.1.1.5.0");
        assert!(StandardOid::SysDescr.description().contains("System"));
    }

    #[test]
    fn test_vendor_oid_cisco() {
        let cisco_oids = VendorOid::cisco_common();
        assert!(!cisco_oids.is_empty());
        assert!(cisco_oids.iter().all(|oid| oid.vendor() == "Cisco"));
    }

    #[test]
    fn test_vendor_oid_juniper() {
        let juniper_oids = VendorOid::juniper_common();
        assert!(!juniper_oids.is_empty());
        assert!(juniper_oids.iter().all(|oid| oid.vendor() == "Juniper"));
    }

    #[test]
    fn test_oid_map_default() {
        let map = OidMap::default();
        assert!(!map.standard.is_empty());
        assert!(!map.vendor.is_empty());
        assert!(map.custom.is_empty());
    }

    #[test]
    fn test_oid_map_operations() {
        let mut map = OidMap::new();

        // Add custom OID
        map.add_custom("test_oid".to_string(), "1.2.3.4.5".to_string());
        assert_eq!(map.get_custom("test_oid"), Some("1.2.3.4.5"));

        // Test resolution
        assert!(map.resolve("SysDescr").is_some());
        assert_eq!(map.resolve("test_oid"), Some("1.2.3.4.5".to_string()));
        assert_eq!(map.resolve("nonexistent"), None);
    }

    #[test]
    fn test_oid_map_list_operations() {
        let map = OidMap::default();

        let all_names = map.list_names();
        let standard_names = map.list_standard();
        let vendor_names = map.list_vendor();
        let custom_names = map.list_custom();

        assert!(!standard_names.is_empty());
        assert!(!vendor_names.is_empty());
        assert!(custom_names.is_empty());
        assert_eq!(all_names.len(), standard_names.len() + vendor_names.len());
    }
}
