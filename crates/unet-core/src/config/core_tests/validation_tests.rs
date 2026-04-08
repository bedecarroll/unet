//! Tests for configuration validation

use super::super::core::Config;

#[test]
fn test_config_validate_default_config_requires_snmp_community() {
    let config = Config::default();
    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("SNMP community must be configured explicitly")
    );
}

#[test]
fn test_config_validate_empty_snmp_community() {
    let mut config = Config::default();
    config.snmp.community = String::new();

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("SNMP community must be configured explicitly")
    );
}

#[test]
fn test_config_validate_empty_database_url() {
    let mut config = Config::default();
    config.database.url = String::new();

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Database URL cannot be empty"));
}

#[test]
fn test_config_validate_zero_database_max_connections() {
    let mut config = Config::default();
    config.database.max_connections = Some(0);

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Database max_connections must be greater than 0")
    );
}

#[test]
fn test_config_validate_zero_database_timeout() {
    let mut config = Config::default();
    config.database.timeout = Some(0);

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Database timeout must be greater than 0")
    );
}

#[test]
fn test_config_validate_empty_server_host() {
    let mut config = Config::default();
    config.server.host = String::new();

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Server host cannot be empty"));
}

#[test]
fn test_config_validate_zero_server_port() {
    let mut config = Config::default();
    config.server.port = 0;

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Server port must be greater than 0")
    );
}

#[test]
fn test_config_validate_zero_max_request_size() {
    let mut config = Config::default();
    config.server.max_request_size = 0;

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Server max_request_size must be greater than 0")
    );
}

#[test]
fn test_config_validate_invalid_server_address() {
    let mut config = Config::default();
    config.server.host = "invalid host with spaces".to_string();

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid server address"));
}

#[test]
fn test_config_validate_empty_git_branch() {
    let mut config = Config::default();
    config.git.branch = String::new();

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Git branch cannot be empty"));
}

#[test]
fn test_config_validate_zero_git_sync_interval() {
    let mut config = Config::default();
    config.git.sync_interval = 0;

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Git sync_interval must be greater than 0")
    );
}

#[test]
fn test_config_validate_auth_enabled_without_token() {
    let mut config = Config::default();
    config.auth.enabled = true;

    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Auth token must be set when auth is enabled")
    );
}
