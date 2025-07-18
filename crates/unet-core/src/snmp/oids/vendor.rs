//! Vendor-specific OID definitions for proprietary network device monitoring

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_oid_new() {
        let oid = VendorOid::new(
            "Test Vendor".to_string(),
            "1.2.3.4.5".to_string(),
            "Test description".to_string(),
        );

        assert_eq!(oid.vendor(), "Test Vendor");
        assert_eq!(oid.oid(), "1.2.3.4.5");
        assert_eq!(oid.description(), "Test description");
        assert_eq!(oid.vendor, "Test Vendor");
        assert_eq!(oid.oid, "1.2.3.4.5");
        assert_eq!(oid.description, "Test description");
    }

    #[test]
    fn test_vendor_oid_getters() {
        let oid = VendorOid {
            vendor: "Custom Corp".to_string(),
            oid: "1.3.6.1.4.1.12345.1.2.3".to_string(),
            description: "Custom metric for testing".to_string(),
        };

        assert_eq!(oid.vendor(), "Custom Corp");
        assert_eq!(oid.oid(), "1.3.6.1.4.1.12345.1.2.3");
        assert_eq!(oid.description(), "Custom metric for testing");
    }

    #[test]
    fn test_vendor_oid_equality() {
        let oid1 = VendorOid::new(
            "Vendor".to_string(),
            "1.2.3".to_string(),
            "Description".to_string(),
        );
        let oid2 = VendorOid::new(
            "Vendor".to_string(),
            "1.2.3".to_string(),
            "Description".to_string(),
        );
        let oid3 = VendorOid::new(
            "Different".to_string(),
            "1.2.3".to_string(),
            "Description".to_string(),
        );

        assert_eq!(oid1, oid2);
        assert_ne!(oid1, oid3);
    }

    #[test]
    fn test_vendor_oid_serialization() {
        let oid = VendorOid::new(
            "SerializeTest".to_string(),
            "1.9.8.7".to_string(),
            "Serialization test".to_string(),
        );

        let serialized = serde_json::to_string(&oid).unwrap();
        let deserialized: VendorOid = serde_json::from_str(&serialized).unwrap();

        assert_eq!(oid, deserialized);
        assert_eq!(oid.vendor(), deserialized.vendor());
        assert_eq!(oid.oid(), deserialized.oid());
        assert_eq!(oid.description(), deserialized.description());
    }

    #[test]
    fn test_vendor_oid_debug() {
        let oid = VendorOid::new(
            "DebugTest".to_string(),
            "1.2.3.4".to_string(),
            "Debug format test".to_string(),
        );

        let debug_str = format!("{oid:?}");
        assert!(debug_str.contains("VendorOid"));
        assert!(debug_str.contains("DebugTest"));
        assert!(debug_str.contains("1.2.3.4"));
        assert!(debug_str.contains("Debug format test"));
    }

    #[test]
    fn test_vendor_oid_cisco() {
        let cisco_oids = VendorOid::cisco_common();
        assert!(!cisco_oids.is_empty());
        assert!(cisco_oids.iter().all(|oid| oid.vendor() == "Cisco"));

        // Verify specific OIDs
        assert_eq!(cisco_oids.len(), 3);
        assert!(
            cisco_oids
                .iter()
                .any(|oid| oid.oid() == "1.3.6.1.4.1.9.2.1.3.0")
        );
        assert!(
            cisco_oids
                .iter()
                .any(|oid| oid.oid() == "1.3.6.1.4.1.9.2.1.8.0")
        );
        assert!(
            cisco_oids
                .iter()
                .any(|oid| oid.oid() == "1.3.6.1.4.1.9.9.13.1.3.1.3")
        );

        // Verify descriptions
        assert!(
            cisco_oids
                .iter()
                .any(|oid| oid.description().contains("CPU"))
        );
        assert!(
            cisco_oids
                .iter()
                .any(|oid| oid.description().contains("memory"))
        );
        assert!(
            cisco_oids
                .iter()
                .any(|oid| oid.description().contains("temperature"))
        );
    }

    #[test]
    fn test_vendor_oid_juniper() {
        let juniper_oids = VendorOid::juniper_common();
        assert!(!juniper_oids.is_empty());
        assert!(juniper_oids.iter().all(|oid| oid.vendor() == "Juniper"));

        // Verify specific OIDs
        assert_eq!(juniper_oids.len(), 3);
        assert!(
            juniper_oids
                .iter()
                .any(|oid| oid.oid() == "1.3.6.1.4.1.2636.3.1.13.1.8")
        );
        assert!(
            juniper_oids
                .iter()
                .any(|oid| oid.oid() == "1.3.6.1.4.1.2636.3.1.13.1.11")
        );
        assert!(
            juniper_oids
                .iter()
                .any(|oid| oid.oid() == "1.3.6.1.4.1.2636.3.1.13.1.7")
        );

        // Verify descriptions
        assert!(
            juniper_oids
                .iter()
                .any(|oid| oid.description().contains("CPU"))
        );
        assert!(
            juniper_oids
                .iter()
                .any(|oid| oid.description().contains("memory"))
        );
        assert!(
            juniper_oids
                .iter()
                .any(|oid| oid.description().contains("temperature"))
        );
    }

    #[test]
    fn test_vendor_oid_hash() {
        use std::collections::HashSet;

        let oid1 = VendorOid::new("A".to_string(), "1".to_string(), "D1".to_string());
        let oid2 = VendorOid::new("A".to_string(), "1".to_string(), "D1".to_string());
        let oid3 = VendorOid::new("B".to_string(), "2".to_string(), "D2".to_string());

        let mut set = HashSet::new();
        set.insert(oid1.clone());
        set.insert(oid2); // Should not increase size (duplicate)
        set.insert(oid3);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&oid1));
    }

    #[test]
    fn test_vendor_oid_empty_strings() {
        let oid = VendorOid::new(String::new(), String::new(), String::new());

        assert_eq!(oid.vendor(), "");
        assert_eq!(oid.oid(), "");
        assert_eq!(oid.description(), "");
    }
}
