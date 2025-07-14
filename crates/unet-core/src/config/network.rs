//! Network configuration utilities

use crate::error::{Error, Result};
use std::net::{IpAddr, SocketAddr};

/// Parse a socket address with a default port if none is specified
///
/// This function provides better error handling than the standard library's
/// `parse().unwrap()` pattern and supports both IPv4 and IPv6 addresses.
///
/// # Arguments
/// * `addr_str` - The address string to parse (e.g., "127.0.0.1:22" or "127.0.0.1")
/// * `default_port` - Port to use if none is specified in the address string
///
/// # Returns
/// * `Ok(SocketAddr)` - Successfully parsed socket address
/// * `Err(Error)` - If the address string is invalid
///
/// # Errors
/// Returns an error if the address string cannot be parsed as a valid IP address or socket address
///
/// # Examples
/// ```ignore
/// use unet_core::config::network::parse_socket_addr_with_default_port;
///
/// let addr = parse_socket_addr_with_default_port("127.0.0.1:22", 161)?;
/// assert_eq!(addr.port(), 22);
///
/// let addr = parse_socket_addr_with_default_port("127.0.0.1", 161)?;
/// assert_eq!(addr.port(), 161);
/// ```
pub fn parse_socket_addr_with_default_port(
    addr_str: &str,
    default_port: u16,
) -> Result<SocketAddr> {
    // Check if address already contains a port
    if addr_str.contains(':') && !addr_str.starts_with('[') {
        // For IPv4 with port, or IPv6 with brackets and port
        addr_str
            .parse()
            .map_err(|e| Error::config(format!("Invalid socket address '{addr_str}': {e}")))
    } else if addr_str.starts_with('[') && addr_str.contains("]:") {
        // IPv6 with brackets and port
        addr_str
            .parse()
            .map_err(|e| Error::config(format!("Invalid IPv6 socket address '{addr_str}': {e}")))
    } else {
        // Address without port - add default port
        let ip: IpAddr = addr_str
            .parse()
            .map_err(|e| Error::config(format!("Invalid IP address '{addr_str}': {e}")))?;
        Ok(SocketAddr::new(ip, default_port))
    }
}

/// Parse an IP address string
///
/// This is a simple wrapper around the standard library's parse method
/// with better error handling.
///
/// # Arguments
/// * `ip_str` - The IP address string to parse
///
/// # Returns
/// * `Ok(IpAddr)` - Successfully parsed IP address
/// * `Err(Error)` - If the address string is invalid
///
/// # Errors
/// Returns an error if the address string cannot be parsed as a valid IP address
pub fn parse_ip_addr(ip_str: &str) -> Result<IpAddr> {
    ip_str
        .parse()
        .map_err(|e| Error::config(format!("Invalid IP address '{ip_str}': {e}")))
}

/// Parse a socket address string
///
/// This is a simple wrapper around the standard library's parse method
/// with better error handling.
///
/// # Arguments
/// * `addr_str` - The socket address string to parse
///
/// # Returns
/// * `Ok(SocketAddr)` - Successfully parsed socket address
/// * `Err(Error)` - If the address string is invalid
///
/// # Errors
/// Returns an error if the address string cannot be parsed as a valid socket address
pub fn parse_socket_addr(addr_str: &str) -> Result<SocketAddr> {
    addr_str
        .parse()
        .map_err(|e| Error::config(format!("Invalid socket address '{addr_str}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_socket_addr_with_default_port() {
        // Address with port should use specified port
        let addr = parse_socket_addr_with_default_port("127.0.0.1:22", 161).unwrap();
        assert_eq!(addr.port(), 22);

        // Address without port should use default
        let addr = parse_socket_addr_with_default_port("127.0.0.1", 161).unwrap();
        assert_eq!(addr.port(), 161);

        // Invalid address should fail
        assert!(parse_socket_addr_with_default_port("invalid", 161).is_err());
    }

    #[test]
    fn test_parse_ip_addr() {
        // Valid IPv4 address should parse correctly
        let ip = parse_ip_addr("192.168.1.1").unwrap();
        assert!(ip.is_ipv4());

        // Valid IPv6 address should parse correctly
        let ip = parse_ip_addr("::1").unwrap();
        assert!(ip.is_ipv6());

        // Invalid address should fail
        assert!(parse_ip_addr("invalid").is_err());
    }

    #[test]
    fn test_parse_socket_addr() {
        // Valid address should parse correctly
        let addr = parse_socket_addr("127.0.0.1:8080").unwrap();
        assert_eq!(addr.port(), 8080);

        // Invalid address should fail
        assert!(parse_socket_addr("invalid").is_err());
    }
}
