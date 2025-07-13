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
    #[must_use]
    pub const fn oid(&self) -> &'static str {
        match self {
            Self::SysDescr => "1.3.6.1.2.1.1.1.0",
            Self::SysObjectId => "1.3.6.1.2.1.1.2.0",
            Self::SysUpTime => "1.3.6.1.2.1.1.3.0",
            Self::SysContact => "1.3.6.1.2.1.1.4.0",
            Self::SysName => "1.3.6.1.2.1.1.5.0",
            Self::SysLocation => "1.3.6.1.2.1.1.6.0",
            Self::SysServices => "1.3.6.1.2.1.1.7.0",
            Self::IfNumber => "1.3.6.1.2.1.2.1.0",
            Self::IfTable => "1.3.6.1.2.1.2.2.1",
            Self::IfIndex => "1.3.6.1.2.1.2.2.1.1",
            Self::IfDescr => "1.3.6.1.2.1.2.2.1.2",
            Self::IfType => "1.3.6.1.2.1.2.2.1.3",
            Self::IfMtu => "1.3.6.1.2.1.2.2.1.4",
            Self::IfSpeed => "1.3.6.1.2.1.2.2.1.5",
            Self::IfPhysAddress => "1.3.6.1.2.1.2.2.1.6",
            Self::IfAdminStatus => "1.3.6.1.2.1.2.2.1.7",
            Self::IfOperStatus => "1.3.6.1.2.1.2.2.1.8",
            Self::IfLastChange => "1.3.6.1.2.1.2.2.1.9",
            Self::IfInOctets => "1.3.6.1.2.1.2.2.1.10",
            Self::IfInUcastPkts => "1.3.6.1.2.1.2.2.1.11",
            Self::IfInErrors => "1.3.6.1.2.1.2.2.1.14",
            Self::IfOutOctets => "1.3.6.1.2.1.2.2.1.16",
            Self::IfOutUcastPkts => "1.3.6.1.2.1.2.2.1.17",
            Self::IfOutErrors => "1.3.6.1.2.1.2.2.1.20",
        }
    }

    /// Get description of this OID
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::SysDescr => "System description",
            Self::SysObjectId => "System object identifier",
            Self::SysUpTime => "System uptime in hundredths of seconds",
            Self::SysContact => "System contact information",
            Self::SysName => "System name",
            Self::SysLocation => "System location",
            Self::SysServices => "System services",
            Self::IfNumber => "Number of network interfaces",
            Self::IfTable => "Network interface table",
            Self::IfIndex => "Interface index",
            Self::IfDescr => "Interface description",
            Self::IfType => "Interface type",
            Self::IfMtu => "Interface MTU",
            Self::IfSpeed => "Interface speed in bits per second",
            Self::IfPhysAddress => "Interface physical address",
            Self::IfAdminStatus => "Interface administrative status",
            Self::IfOperStatus => "Interface operational status",
            Self::IfLastChange => "Interface last change time",
            Self::IfInOctets => "Interface input octets",
            Self::IfInUcastPkts => "Interface input unicast packets",
            Self::IfInErrors => "Interface input errors",
            Self::IfOutOctets => "Interface output octets",
            Self::IfOutUcastPkts => "Interface output unicast packets",
            Self::IfOutErrors => "Interface output errors",
        }
    }

    /// Get all standard system OIDs for basic device information
    #[must_use]
    pub fn system_oids() -> Vec<Self> {
        vec![
            Self::SysDescr,
            Self::SysObjectId,
            Self::SysUpTime,
            Self::SysContact,
            Self::SysName,
            Self::SysLocation,
            Self::SysServices,
        ]
    }

    /// Get all interface table OIDs for interface monitoring
    #[must_use]
    pub fn interface_oids() -> Vec<Self> {
        vec![
            Self::IfNumber,
            Self::IfIndex,
            Self::IfDescr,
            Self::IfType,
            Self::IfMtu,
            Self::IfSpeed,
            Self::IfPhysAddress,
            Self::IfAdminStatus,
            Self::IfOperStatus,
            Self::IfLastChange,
            Self::IfInOctets,
            Self::IfInUcastPkts,
            Self::IfInErrors,
            Self::IfOutOctets,
            Self::IfOutUcastPkts,
            Self::IfOutErrors,
        ]
    }
}

/// A vendor-specific OID entry
///
/// This design supports any vendor without requiring code changes.
/// Vendors can be defined at runtime through configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VendorOid {
    /// Vendor name (e.g., "Cisco", "Juniper", "Custom Corp")
    pub vendor: String,
    /// Specific OID string
    pub oid: String,
    /// Description of what this OID represents
    pub description: String,
}

impl VendorOid {
    /// Create a new vendor OID entry
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Takes String parameters
    pub fn new(vendor: String, oid: String, description: String) -> Self {
        Self {
            vendor,
            oid,
            description,
        }
    }

    /// Get the OID string
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // String deref coercion not const
    pub fn oid(&self) -> &str {
        &self.oid
    }

    /// Get the description
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // String deref coercion not const
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the vendor name
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // String deref coercion not const
    pub fn vendor(&self) -> &str {
        &self.vendor
    }

    /// Create common Cisco OIDs
    #[must_use]
    pub fn cisco_common() -> Vec<Self> {
        vec![
            Self::new(
                "Cisco".to_string(),
                "1.3.6.1.4.1.9.2.1.3.0".to_string(),
                "Cisco CPU utilization".to_string(),
            ),
            Self::new(
                "Cisco".to_string(),
                "1.3.6.1.4.1.9.2.1.8.0".to_string(),
                "Cisco memory utilization".to_string(),
            ),
            Self::new(
                "Cisco".to_string(),
                "1.3.6.1.4.1.9.9.13.1.3.1.3".to_string(),
                "Cisco temperature sensor".to_string(),
            ),
        ]
    }

    /// Create common Juniper OIDs
    #[must_use]
    pub fn juniper_common() -> Vec<Self> {
        vec![
            Self::new(
                "Juniper".to_string(),
                "1.3.6.1.4.1.2636.3.1.13.1.8".to_string(),
                "Juniper CPU utilization".to_string(),
            ),
            Self::new(
                "Juniper".to_string(),
                "1.3.6.1.4.1.2636.3.1.13.1.11".to_string(),
                "Juniper memory utilization".to_string(),
            ),
            Self::new(
                "Juniper".to_string(),
                "1.3.6.1.4.1.2636.3.1.13.1.7".to_string(),
                "Juniper temperature".to_string(),
            ),
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
            map.standard.insert(format!("{oid:?}"), oid);
        }
        for oid in StandardOid::interface_oids() {
            map.standard.insert(format!("{oid:?}"), oid);
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
    #[must_use]
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
    #[must_use]
    pub fn get_standard(&self, name: &str) -> Option<&StandardOid> {
        self.standard.get(name)
    }

    /// Get vendor OID by name
    #[must_use]
    pub fn get_vendor(&self, name: &str) -> Option<&VendorOid> {
        self.vendor.get(name)
    }

    /// Get custom OID by name
    #[must_use]
    pub fn get_custom(&self, name: &str) -> Option<&str> {
        self.custom.get(name).map(std::string::String::as_str)
    }

    /// Resolve any OID name to its string representation
    #[must_use]
    pub fn resolve(&self, name: &str) -> Option<String> {
        self.get_standard(name)
            .map(|oid| oid.oid().to_string())
            .or_else(|| self.get_vendor(name).map(|oid| oid.oid().to_string()))
            .or_else(|| self.get_custom(name).map(str::to_string))
    }

    /// Get all OID names in the map
    #[must_use]
    pub fn list_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.standard.keys().cloned());
        names.extend(self.vendor.keys().cloned());
        names.extend(self.custom.keys().cloned());
        names.sort();
        names
    }

    /// Get all standard OID names
    #[must_use]
    pub fn list_standard(&self) -> Vec<String> {
        let mut names: Vec<String> = self.standard.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all vendor OID names  
    #[must_use]
    pub fn list_vendor(&self) -> Vec<String> {
        let mut names: Vec<String> = self.vendor.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all custom OID names
    #[must_use]
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
