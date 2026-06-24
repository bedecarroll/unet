use std::net::IpAddr;

use crate::commands::test_support::{expect_json_object, expect_json_parse_error};
use unet_core::models::{DeviceRole, Lifecycle, Vendor};

#[tokio::test]
async fn test_vendor_parsing_valid() {
    let valid_vendors = vec![
        ("cisco", Vendor::Cisco),
        ("juniper", Vendor::Juniper),
        ("arista", Vendor::Arista),
        ("generic", Vendor::Generic),
    ];

    for (vendor_str, expected) in valid_vendors {
        let result = vendor_str.parse::<Vendor>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

#[tokio::test]
async fn test_device_role_parsing_valid() {
    let valid_roles = vec![
        ("router", DeviceRole::Router),
        ("switch", DeviceRole::Switch),
        ("server", DeviceRole::Server),
        ("firewall", DeviceRole::Firewall),
    ];

    for (role_str, expected) in valid_roles {
        let result = role_str.parse::<DeviceRole>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

#[tokio::test]
async fn test_lifecycle_parsing_valid() {
    let valid_lifecycles = vec![
        ("planned", Lifecycle::Planned),
        ("implementing", Lifecycle::Implementing),
        ("live", Lifecycle::Live),
        ("decommissioned", Lifecycle::Decommissioned),
    ];

    for (lifecycle_str, expected) in valid_lifecycles {
        let result = lifecycle_str.parse::<Lifecycle>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

#[tokio::test]
async fn test_management_ip_parsing_valid() {
    let valid_ips = vec![
        "192.168.1.1",
        "10.0.0.1",
        "172.16.1.1",
        "2001:db8::1",
        "::1",
        "127.0.0.1",
    ];

    for ip_str in valid_ips {
        let result = ip_str.parse::<IpAddr>();
        assert!(result.is_ok(), "Failed to parse valid IP: {ip_str}");
    }
}

#[tokio::test]
async fn test_management_ip_parsing_invalid() {
    let invalid_ips = vec![
        "256.256.256.256",
        "192.168.1",
        "not-an-ip",
        "192.168.1.1.1",
        "192.168.1.256",
        "",
    ];

    for ip_str in invalid_ips {
        let result = ip_str.parse::<IpAddr>();
        assert!(result.is_err(), "Should have failed to parse IP: {ip_str}");
    }
}

#[tokio::test]
async fn test_custom_data_json_parsing_valid() {
    let value = expect_json_object(
        r#"{"rack": "A1", "port_count": 48, "power_consumption": 125.5}"#,
    );

    assert_eq!(value["rack"], "A1");
    assert_eq!(value["port_count"], 48);
    assert_eq!(value["power_consumption"], 125.5);
}

#[tokio::test]
async fn test_custom_data_json_parsing_invalid() {
    expect_json_parse_error(r#"{"rack": "A1", "port_count": }"#);
}

#[tokio::test]
async fn test_custom_data_json_parsing_empty_object() {
    let value = expect_json_object("{}");
    assert!(value.as_object().unwrap().is_empty());
}

#[tokio::test]
async fn test_custom_data_json_parsing_complex() {
    let value = expect_json_object(
        r#"{
            "hardware": {
                "cpu": "Intel Xeon",
                "memory": "32GB",
                "storage": {
                    "type": "SSD",
                    "capacity": "1TB"
                }
            },
            "network_interfaces": [
                {"name": "eth0", "speed": "1Gbps"},
                {"name": "eth1", "speed": "10Gbps"}
            ],
            "monitoring": {
                "enabled": true,
                "polling_interval": 60,
                "snmp_community": "public"
            }
        }"#,
    );

    assert_eq!(value["hardware"]["cpu"], "Intel Xeon");
    assert!(value["network_interfaces"].is_array());
    assert!(value["monitoring"]["enabled"].as_bool().unwrap());
}
