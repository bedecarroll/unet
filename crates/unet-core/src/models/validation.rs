//! Validation functions for model fields
//!
//! Contains helper functions for validating various field types used across
//! the model structs.

/// Validates interface names for network devices
///
/// Interface names should be non-empty, reasonable length, and contain
/// only allowed characters for common network interface naming patterns.
///
/// # Examples
/// - `eth0` - Valid
/// - `GigabitEthernet0/0/1` - Valid
/// - `xe-0/0/0` - Valid
/// - `lo:1` - Valid
/// - `very-long-interface-name-that-exceeds-the-maximum-allowed-length` - Invalid
/// - `""` - Invalid (empty)
/// - `eth0@vlan` - Invalid (contains @)
#[must_use]
pub fn is_valid_interface_name(interface: &str) -> bool {
    if interface.is_empty() || interface.len() > 64 {
        return false;
    }

    // Allow alphanumeric characters, slashes, dashes, dots, and colons
    // Common patterns: eth0, GigabitEthernet0/0/1, xe-0/0/0, etc.
    interface
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '.' || c == ':')
}

/// Validates domain names according to basic RFC requirements
///
/// Domain names must be non-empty, within length limits, and contain only
/// valid characters in valid formats.
///
/// # Examples
/// - `example.com` - Valid
/// - `sub.example.com` - Valid
/// - `test-domain.example.org` - Valid
/// - `.example.com` - Invalid (starts with dot)
/// - `example..com` - Invalid (double dot)
/// - `example-.com` - Invalid (label ends with dash)
/// - `-example.com` - Invalid (label starts with dash)
#[must_use]
pub fn is_valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }

    // Check for valid domain format (simplified validation)
    domain.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_interface_name() {
        // Valid interface names
        assert!(is_valid_interface_name("eth0"));
        assert!(is_valid_interface_name("GigabitEthernet0/0/1"));
        assert!(is_valid_interface_name("xe-0/0/0"));
        assert!(is_valid_interface_name("lo"));
        assert!(is_valid_interface_name("vlan100"));
        assert!(is_valid_interface_name("eth0:1"));
        assert!(is_valid_interface_name("bond0.100"));

        // Invalid interface names
        assert!(!is_valid_interface_name("")); // Empty
        assert!(!is_valid_interface_name("eth0@vlan")); // Invalid character @
        assert!(!is_valid_interface_name("eth0#1")); // Invalid character #
        assert!(!is_valid_interface_name("eth 0")); // Space
        assert!(!is_valid_interface_name(&"a".repeat(65))); // Too long
    }

    #[test]
    fn test_is_valid_domain() {
        // Valid domains
        assert!(is_valid_domain("example.com"));
        assert!(is_valid_domain("sub.example.com"));
        assert!(is_valid_domain("test-domain.example.org"));
        assert!(is_valid_domain("a.b"));
        assert!(is_valid_domain("localhost"));
        assert!(is_valid_domain("test123.example-domain.com"));

        // Invalid domains
        assert!(!is_valid_domain("")); // Empty
        assert!(!is_valid_domain(".example.com")); // Starts with dot
        assert!(!is_valid_domain("example.com.")); // Ends with dot
        assert!(!is_valid_domain("example..com")); // Double dot
        assert!(!is_valid_domain("example-.com")); // Label ends with dash
        assert!(!is_valid_domain("-example.com")); // Label starts with dash
        assert!(!is_valid_domain("example.c_m")); // Invalid character
        assert!(!is_valid_domain(&format!("{}.com", "a".repeat(64)))); // Label too long
        assert!(!is_valid_domain(&"a.".repeat(127))); // Domain too long
    }
}
