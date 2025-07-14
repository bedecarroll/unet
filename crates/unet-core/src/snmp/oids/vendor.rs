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
}
