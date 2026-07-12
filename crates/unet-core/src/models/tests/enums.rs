//! Tests for enum types (`Lifecycle`, `DeviceRole`, `Vendor`)

use crate::models::*;
use serde_json;

#[test]
fn test_lifecycle_display() {
    assert_eq!(Lifecycle::Planned.to_string(), "planned");
    assert_eq!(Lifecycle::Implementing.to_string(), "implementing");
    assert_eq!(Lifecycle::Live.to_string(), "live");
    assert_eq!(Lifecycle::Decommissioned.to_string(), "decommissioned");
}

#[test]
fn test_lifecycle_from_str() {
    assert_eq!("planned".parse::<Lifecycle>().unwrap(), Lifecycle::Planned);
    assert_eq!(
        "IMPLEMENTING".parse::<Lifecycle>().unwrap(),
        Lifecycle::Implementing
    );
    assert_eq!("Live".parse::<Lifecycle>().unwrap(), Lifecycle::Live);
    assert_eq!(
        "DECOMMISSIONED".parse::<Lifecycle>().unwrap(),
        Lifecycle::Decommissioned
    );

    assert!("invalid".parse::<Lifecycle>().is_err());
}

#[test]
fn test_lifecycle_from_string() {
    assert_eq!(Lifecycle::from("planned".to_string()), Lifecycle::Planned);
    assert_eq!(Lifecycle::from("invalid".to_string()), Lifecycle::Planned); // fallback
}

#[test]
fn test_lifecycle_serde() {
    let planned = Lifecycle::Planned;
    let json = serde_json::to_string(&planned).unwrap();
    assert_eq!(json, "\"planned\"");

    let deserialized: Lifecycle = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, planned);
}

#[test]
fn test_device_role_display() {
    assert_eq!(DeviceRole::Router.to_string(), "router");
    assert_eq!(DeviceRole::Switch.to_string(), "switch");
    assert_eq!(DeviceRole::Firewall.to_string(), "firewall");
    assert_eq!(DeviceRole::LoadBalancer.to_string(), "loadbalancer");
    assert_eq!(DeviceRole::AccessPoint.to_string(), "accesspoint");
    assert_eq!(
        DeviceRole::SecurityAppliance.to_string(),
        "securityappliance"
    );
    assert_eq!(DeviceRole::Monitor.to_string(), "monitor");
    assert_eq!(DeviceRole::Server.to_string(), "server");
    assert_eq!(DeviceRole::Storage.to_string(), "storage");
    assert_eq!(DeviceRole::Other.to_string(), "other");
}

#[test]
fn test_device_role_from_str() {
    assert_eq!("router".parse::<DeviceRole>().unwrap(), DeviceRole::Router);
    assert_eq!("SWITCH".parse::<DeviceRole>().unwrap(), DeviceRole::Switch);
    assert_eq!(
        "Firewall".parse::<DeviceRole>().unwrap(),
        DeviceRole::Firewall
    );

    assert!("invalid".parse::<DeviceRole>().is_err());
}

#[test]
fn test_device_role_from_string() {
    assert_eq!(DeviceRole::from("router".to_string()), DeviceRole::Router);
    assert_eq!(DeviceRole::from("invalid".to_string()), DeviceRole::Other); // fallback
}

#[test]
fn test_device_role_serde() {
    let router = DeviceRole::Router;
    let json = serde_json::to_string(&router).unwrap();
    assert_eq!(json, "\"router\"");

    let deserialized: DeviceRole = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, router);
}

#[test]
fn test_vendor_display() {
    assert_eq!(Vendor::Cisco.to_string(), "cisco");
    assert_eq!(Vendor::Juniper.to_string(), "juniper");
    assert_eq!(Vendor::Arista.to_string(), "arista");
    assert_eq!(Vendor::PaloAlto.to_string(), "paloalto");
    assert_eq!(Vendor::Fortinet.to_string(), "fortinet");
    assert_eq!(Vendor::Hpe.to_string(), "hpe");
    assert_eq!(Vendor::Dell.to_string(), "dell");
    assert_eq!(Vendor::Extreme.to_string(), "extreme");
    assert_eq!(Vendor::Mikrotik.to_string(), "mikrotik");
    assert_eq!(Vendor::Ubiquiti.to_string(), "ubiquiti");
    assert_eq!(Vendor::Generic.to_string(), "generic");
}

#[test]
fn test_vendor_from_str() {
    assert_eq!("cisco".parse::<Vendor>().unwrap(), Vendor::Cisco);
    assert_eq!("JUNIPER".parse::<Vendor>().unwrap(), Vendor::Juniper);
    assert_eq!("Arista".parse::<Vendor>().unwrap(), Vendor::Arista);

    assert!("invalid".parse::<Vendor>().is_err());
}

#[test]
fn test_vendor_from_string() {
    assert_eq!(Vendor::from("cisco".to_string()), Vendor::Cisco);
    assert_eq!(Vendor::from("invalid".to_string()), Vendor::Generic); // fallback
}

#[test]
fn test_vendor_serde() {
    let cisco = Vendor::Cisco;
    let json = serde_json::to_string(&cisco).unwrap();
    assert_eq!(json, "\"cisco\"");

    let deserialized: Vendor = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, cisco);
}

#[test]
fn test_all_enum_variants_coverage() {
    // Test that all variants are handled in serialization/deserialization
    let lifecycles = [
        Lifecycle::Planned,
        Lifecycle::Implementing,
        Lifecycle::Live,
        Lifecycle::Decommissioned,
    ];

    for lifecycle in lifecycles {
        let json = serde_json::to_string(&lifecycle).unwrap();
        let deserialized: Lifecycle = serde_json::from_str(&json).unwrap();
        assert_eq!(lifecycle, deserialized);
    }

    let roles = [
        DeviceRole::Router,
        DeviceRole::Switch,
        DeviceRole::Firewall,
        DeviceRole::LoadBalancer,
        DeviceRole::AccessPoint,
        DeviceRole::SecurityAppliance,
        DeviceRole::Monitor,
        DeviceRole::Server,
        DeviceRole::Storage,
        DeviceRole::Other,
    ];

    for role in roles {
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: DeviceRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, deserialized);
    }

    let vendors = [
        Vendor::Cisco,
        Vendor::Juniper,
        Vendor::Arista,
        Vendor::PaloAlto,
        Vendor::Fortinet,
        Vendor::Hpe,
        Vendor::Dell,
        Vendor::Extreme,
        Vendor::Mikrotik,
        Vendor::Ubiquiti,
        Vendor::Generic,
    ];

    for vendor in vendors {
        let json = serde_json::to_string(&vendor).unwrap();
        let deserialized: Vendor = serde_json::from_str(&json).unwrap();
        assert_eq!(vendor, deserialized);
    }
}
