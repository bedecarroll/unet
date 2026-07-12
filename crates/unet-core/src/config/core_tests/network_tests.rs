//! Tests for network configuration and URL handling

use super::super::core::Config;
use super::super::defaults;

#[test]
fn test_config_database_url() {
    let config = Config::default();
    assert_eq!(
        config.database_url(),
        defaults::database::DEFAULT_DATABASE_URL
    );

    let mut custom_config = Config::default();
    custom_config.database.url = "postgres://localhost/test".to_string();
    assert_eq!(custom_config.database_url(), "postgres://localhost/test");
}

#[test]
fn test_config_socket_addr_valid() {
    let config = Config::default();
    let addr = config.socket_addr().unwrap();
    assert_eq!(addr.port(), defaults::network::DEFAULT_SERVER_PORT);
}

#[test]
fn test_config_socket_addr_invalid_host() {
    let mut config = Config::default();
    config.server.host = "invalid host with spaces".to_string();

    let result = config.socket_addr();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid server address"));
}

#[test]
fn test_config_socket_addr_custom_values() {
    let mut config = Config::default();
    config.server.host = "0.0.0.0".to_string();
    config.server.port = 9999;

    let addr = config.socket_addr().unwrap();
    assert_eq!(addr.to_string(), "0.0.0.0:9999");
}
