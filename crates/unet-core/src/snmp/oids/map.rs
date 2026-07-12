//! OID mapping utility for organizing and accessing all types of OIDs

use super::{StandardOid, VendorOid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    #[test]
    fn test_add_standard_and_vendor_methods() {
        let mut map = OidMap::new();

        // Test add_standard method (lines 60-61)
        map.add_standard("TestStandard".to_string(), StandardOid::SysDescr);
        assert_eq!(
            map.get_standard("TestStandard"),
            Some(&StandardOid::SysDescr)
        );

        // Test add_vendor method (lines 65-66)
        let vendor_oid = VendorOid::cisco_common()[0].clone();
        map.add_vendor("TestVendor".to_string(), vendor_oid.clone());
        assert_eq!(map.get_vendor("TestVendor"), Some(&vendor_oid));
    }
}
