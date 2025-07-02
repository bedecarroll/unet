//! Core validation utilities and patterns

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Regex for validating usernames (alphanumeric, hyphens, underscores, 3-50 chars)
    pub static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{3,50}$").unwrap();

    /// Regex for validating node names (alphanumeric, hyphens, underscores, dots, 1-255 chars)
    pub static ref NODE_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_.-]{1,255}$").unwrap();

    /// Regex for validating policy names (alphanumeric, hyphens, underscores, 1-100 chars)
    pub static ref POLICY_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{1,100}$").unwrap();

    /// Regex for validating template names (alphanumeric, hyphens, underscores, dots, 1-100 chars)
    pub static ref TEMPLATE_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_.-]{1,100}$").unwrap();

    /// Regex for validating IP addresses (IPv4 and IPv6)
    pub static ref IP_ADDRESS_REGEX: Regex = Regex::new(
        r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$|^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$|^::1$|^::$"
    ).unwrap();

    /// Regex for validating SNMP community strings (printable ASCII, no spaces, 1-64 chars)
    pub static ref SNMP_COMMUNITY_REGEX: Regex = Regex::new(r"^[!-~]{1,64}$").unwrap();
}

/// Validation functions for common input types
pub mod validators {
    use super::*;

    /// Validate username format
    pub fn validate_username(username: &str) -> bool {
        USERNAME_REGEX.is_match(username)
    }

    /// Validate node name format
    pub fn validate_node_name(name: &str) -> bool {
        NODE_NAME_REGEX.is_match(name)
    }

    /// Validate policy name format
    pub fn validate_policy_name(name: &str) -> bool {
        POLICY_NAME_REGEX.is_match(name)
    }

    /// Validate template name format
    pub fn validate_template_name(name: &str) -> bool {
        TEMPLATE_NAME_REGEX.is_match(name)
    }

    /// Validate IP address format
    pub fn validate_ip_address(ip: &str) -> bool {
        ip.parse::<std::net::IpAddr>().is_ok()
    }

    /// Validate SNMP community string format
    pub fn validate_snmp_community(community: &str) -> bool {
        SNMP_COMMUNITY_REGEX.is_match(community)
    }

    /// Validate port number range
    pub fn validate_port(port: u16) -> bool {
        port > 0
    }

    /// Validate VLAN ID range
    pub fn validate_vlan_id(vlan: u16) -> bool {
        vlan >= 1 && vlan <= 4094
    }
}

#[cfg(test)]
mod tests {
    use super::validators::*;

    #[test]
    fn test_username_validation() {
        assert!(validate_username("user123"));
        assert!(validate_username("test_user"));
        assert!(validate_username("user-name"));
        assert!(!validate_username("us")); // too short
        assert!(!validate_username("user@domain")); // invalid character
        assert!(!validate_username("user name")); // space
    }

    #[test]
    fn test_node_name_validation() {
        assert!(validate_node_name("router-01"));
        assert!(validate_node_name("switch.example.com"));
        assert!(validate_node_name("server_01"));
        assert!(!validate_node_name("")); // empty
        assert!(!validate_node_name("node@domain")); // invalid character
    }

    #[test]
    fn test_ip_address_validation() {
        assert!(validate_ip_address("192.168.1.1"));
        assert!(validate_ip_address("10.0.0.1"));
        assert!(validate_ip_address("::1"));
        assert!(validate_ip_address("2001:db8::1"));
        assert!(!validate_ip_address("256.1.1.1")); // invalid IPv4
        assert!(!validate_ip_address("not-an-ip")); // not an IP
    }

    #[test]
    fn test_port_validation() {
        assert!(validate_port(80));
        assert!(validate_port(443));
        assert!(validate_port(65535));
        assert!(!validate_port(0)); // invalid
        assert!(validate_port(1)); // minimum valid
    }

    #[test]
    fn test_vlan_validation() {
        assert!(validate_vlan_id(1));
        assert!(validate_vlan_id(100));
        assert!(validate_vlan_id(4094));
        assert!(!validate_vlan_id(0)); // invalid
        assert!(!validate_vlan_id(4095)); // reserved
    }
}
