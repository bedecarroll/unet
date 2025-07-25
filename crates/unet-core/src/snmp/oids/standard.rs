//! Standard SNMP OID definitions for RFC-compliant network monitoring

use serde::{Deserialize, Serialize};

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
    fn test_all_standard_oid_variants() {
        // Test OID method for all variants (covers lines 64-66, 68-86)
        assert_eq!(StandardOid::SysObjectId.oid(), "1.3.6.1.2.1.1.2.0");
        assert_eq!(StandardOid::SysUpTime.oid(), "1.3.6.1.2.1.1.3.0");
        assert_eq!(StandardOid::SysContact.oid(), "1.3.6.1.2.1.1.4.0");
        assert_eq!(StandardOid::SysLocation.oid(), "1.3.6.1.2.1.1.6.0");
        assert_eq!(StandardOid::SysServices.oid(), "1.3.6.1.2.1.1.7.0");
        assert_eq!(StandardOid::IfNumber.oid(), "1.3.6.1.2.1.2.1.0");
        assert_eq!(StandardOid::IfTable.oid(), "1.3.6.1.2.1.2.2.1");
        assert_eq!(StandardOid::IfIndex.oid(), "1.3.6.1.2.1.2.2.1.1");
        assert_eq!(StandardOid::IfDescr.oid(), "1.3.6.1.2.1.2.2.1.2");
        assert_eq!(StandardOid::IfType.oid(), "1.3.6.1.2.1.2.2.1.3");
        assert_eq!(StandardOid::IfMtu.oid(), "1.3.6.1.2.1.2.2.1.4");
        assert_eq!(StandardOid::IfSpeed.oid(), "1.3.6.1.2.1.2.2.1.5");
        assert_eq!(StandardOid::IfPhysAddress.oid(), "1.3.6.1.2.1.2.2.1.6");
        assert_eq!(StandardOid::IfAdminStatus.oid(), "1.3.6.1.2.1.2.2.1.7");
        assert_eq!(StandardOid::IfOperStatus.oid(), "1.3.6.1.2.1.2.2.1.8");
        assert_eq!(StandardOid::IfLastChange.oid(), "1.3.6.1.2.1.2.2.1.9");
        assert_eq!(StandardOid::IfInOctets.oid(), "1.3.6.1.2.1.2.2.1.10");
        assert_eq!(StandardOid::IfInUcastPkts.oid(), "1.3.6.1.2.1.2.2.1.11");
        assert_eq!(StandardOid::IfInErrors.oid(), "1.3.6.1.2.1.2.2.1.14");
        assert_eq!(StandardOid::IfOutOctets.oid(), "1.3.6.1.2.1.2.2.1.16");
        assert_eq!(StandardOid::IfOutUcastPkts.oid(), "1.3.6.1.2.1.2.2.1.17");
        assert_eq!(StandardOid::IfOutErrors.oid(), "1.3.6.1.2.1.2.2.1.20");
    }

    #[test]
    fn test_all_standard_oid_descriptions() {
        // Test description method for all variants (covers lines 95-117)
        assert_eq!(
            StandardOid::SysObjectId.description(),
            "System object identifier"
        );
        assert_eq!(
            StandardOid::SysUpTime.description(),
            "System uptime in hundredths of seconds"
        );
        assert_eq!(
            StandardOid::SysContact.description(),
            "System contact information"
        );
        assert_eq!(StandardOid::SysName.description(), "System name");
        assert_eq!(StandardOid::SysLocation.description(), "System location");
        assert_eq!(StandardOid::SysServices.description(), "System services");
        assert_eq!(
            StandardOid::IfNumber.description(),
            "Number of network interfaces"
        );
        assert_eq!(
            StandardOid::IfTable.description(),
            "Network interface table"
        );
        assert_eq!(StandardOid::IfIndex.description(), "Interface index");
        assert_eq!(StandardOid::IfDescr.description(), "Interface description");
        assert_eq!(StandardOid::IfType.description(), "Interface type");
        assert_eq!(StandardOid::IfMtu.description(), "Interface MTU");
        assert_eq!(
            StandardOid::IfSpeed.description(),
            "Interface speed in bits per second"
        );
        assert_eq!(
            StandardOid::IfPhysAddress.description(),
            "Interface physical address"
        );
        assert_eq!(
            StandardOid::IfAdminStatus.description(),
            "Interface administrative status"
        );
        assert_eq!(
            StandardOid::IfOperStatus.description(),
            "Interface operational status"
        );
        assert_eq!(
            StandardOid::IfLastChange.description(),
            "Interface last change time"
        );
        assert_eq!(
            StandardOid::IfInOctets.description(),
            "Interface input octets"
        );
        assert_eq!(
            StandardOid::IfInUcastPkts.description(),
            "Interface input unicast packets"
        );
        assert_eq!(
            StandardOid::IfInErrors.description(),
            "Interface input errors"
        );
        assert_eq!(
            StandardOid::IfOutOctets.description(),
            "Interface output octets"
        );
        assert_eq!(
            StandardOid::IfOutUcastPkts.description(),
            "Interface output unicast packets"
        );
        assert_eq!(
            StandardOid::IfOutErrors.description(),
            "Interface output errors"
        );
    }

    #[test]
    fn test_oid_collections() {
        let system_oids = StandardOid::system_oids();
        assert_eq!(system_oids.len(), 7);
        assert!(system_oids.contains(&StandardOid::SysDescr));
        assert!(system_oids.contains(&StandardOid::SysServices));

        let interface_oids = StandardOid::interface_oids();
        assert_eq!(interface_oids.len(), 16);
        assert!(interface_oids.contains(&StandardOid::IfNumber));
        assert!(interface_oids.contains(&StandardOid::IfOutErrors));
    }
}
