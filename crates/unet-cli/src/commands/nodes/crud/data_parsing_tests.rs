/// Tests for data parsing and validation
use std::net::IpAddr;
use unet_core::prelude::*;

#[tokio::test]
async fn test_vendor_parsing_valid() {
    let vendor_str = "cisco";
    let vendor = Vendor::from(vendor_str.to_string());
    assert_eq!(vendor, Vendor::Cisco);
}

#[tokio::test]
async fn test_device_role_parsing_valid() {
    let role_str = "router";
    let role = DeviceRole::from(role_str.to_string());
    assert_eq!(role, DeviceRole::Router);
}

#[tokio::test]
async fn test_lifecycle_parsing_valid() {
    let lifecycle_str = "live";
    let lifecycle = Lifecycle::from(lifecycle_str.to_string());
    assert_eq!(lifecycle, Lifecycle::Live);
}

#[tokio::test]
async fn test_ip_address_parsing_valid() {
    let ip_str = "192.168.1.1";
    let ip_addr: std::result::Result<IpAddr, std::net::AddrParseError> = ip_str.parse();
    assert!(ip_addr.is_ok());
    assert_eq!(ip_addr.unwrap(), IpAddr::V4([192, 168, 1, 1].into()));
}

#[tokio::test]
async fn test_ip_address_parsing_invalid() {
    let invalid_ip = "not.an.ip.address";
    let ip_addr: std::result::Result<IpAddr, std::net::AddrParseError> = invalid_ip.parse();
    assert!(ip_addr.is_err());
}

#[tokio::test]
async fn test_json_parsing_valid() {
    let json_str = r#"{"environment": "production", "rack": "A1", "slot": 5}"#;
    let json_value: std::result::Result<serde_json::Value, serde_json::Error> = serde_json::from_str(json_str);
    assert!(json_value.is_ok());

    let parsed = json_value.unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed["environment"], "production");
    assert_eq!(parsed["rack"], "A1");
    assert_eq!(parsed["slot"], 5);
}

#[tokio::test]
async fn test_json_parsing_invalid() {
    let invalid_json = r#"{"environment": "production", "rack": "A1", "slot": }"#;
    let json_value: std::result::Result<serde_json::Value, serde_json::Error> = serde_json::from_str(invalid_json);
    assert!(json_value.is_err());

    // Test with completely malformed JSON
    let malformed_json = "not json at all";
    let json_value2: std::result::Result<serde_json::Value, serde_json::Error> = serde_json::from_str(malformed_json);
    assert!(json_value2.is_err());
}