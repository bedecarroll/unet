//! Tests for configuration validation

use crate::config::core::Config;
use crate::config::defaults;

#[test]
fn test_validate_and_adjust_default_config() {
    let mut config = Config::default();
    let warnings = config.validate_and_adjust();
    assert!(warnings.is_empty());
}

#[test]
fn test_validate_and_adjust_server_request_size_too_small() {
    let mut config = Config::default();
    config.server.max_request_size = defaults::server::MIN_REQUEST_SIZE - 1;

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("below minimum"));
    assert_eq!(
        config.server.max_request_size,
        defaults::server::MIN_REQUEST_SIZE
    );
}

#[test]
fn test_validate_and_adjust_server_request_size_too_large() {
    let mut config = Config::default();
    config.server.max_request_size = defaults::server::MAX_REQUEST_SIZE + 1;

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("exceeds maximum"));
    assert_eq!(
        config.server.max_request_size,
        defaults::server::MAX_REQUEST_SIZE
    );
}

#[test]
fn test_validate_and_adjust_database_connections_too_small() {
    let mut config = Config::default();
    config.database.max_connections = Some(defaults::database::MIN_DB_CONNECTIONS - 1);

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("below minimum"));
    assert_eq!(
        config.database.max_connections,
        Some(defaults::database::MIN_DB_CONNECTIONS)
    );
}

#[test]
fn test_validate_and_adjust_database_connections_too_large() {
    let mut config = Config::default();
    config.database.max_connections = Some(defaults::database::MAX_DB_CONNECTIONS + 1);

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("exceeds maximum"));
    assert_eq!(
        config.database.max_connections,
        Some(defaults::database::MAX_DB_CONNECTIONS)
    );
}

#[test]
fn test_validate_and_adjust_database_connections_none() {
    let mut config = Config::default();
    config.database.max_connections = None;

    let warnings = config.validate_and_adjust();
    assert!(warnings.is_empty());
    assert_eq!(config.database.max_connections, None);
}

#[test]
fn test_validate_and_adjust_snmp_timeout_too_small() {
    let mut config = Config::default();
    config.snmp.timeout = defaults::snmp::MIN_SNMP_TIMEOUT_SECONDS - 1;

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("below minimum"));
    assert_eq!(
        config.snmp.timeout,
        defaults::snmp::MIN_SNMP_TIMEOUT_SECONDS
    );
}

#[test]
fn test_validate_and_adjust_snmp_timeout_too_large() {
    let mut config = Config::default();
    config.snmp.timeout = defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS + 1;

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("exceeds maximum"));
    assert_eq!(
        config.snmp.timeout,
        defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS
    );
}

#[test]
fn test_validate_and_adjust_snmp_retries_too_large() {
    let mut config = Config::default();
    config.snmp.retries = defaults::snmp::MAX_SNMP_RETRIES + 1;

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("exceeds maximum"));
    assert_eq!(config.snmp.retries, defaults::snmp::MAX_SNMP_RETRIES);
}

#[test]
fn test_validate_and_adjust_multiple_issues() {
    let mut config = Config::default();
    config.server.max_request_size = defaults::server::MIN_REQUEST_SIZE - 1;
    config.snmp.timeout = defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS + 1;
    config.snmp.retries = defaults::snmp::MAX_SNMP_RETRIES + 1;

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 3);
    assert_eq!(
        config.server.max_request_size,
        defaults::server::MIN_REQUEST_SIZE
    );
    assert_eq!(
        config.snmp.timeout,
        defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS
    );
    assert_eq!(config.snmp.retries, defaults::snmp::MAX_SNMP_RETRIES);
}

#[test]
fn test_validate_default_config() {
    let config = Config::default();
    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_empty_database_url() {
    let mut config = Config::default();
    config.database.url = String::new();

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Database URL cannot be empty")
    );
}

#[test]
fn test_validate_whitespace_database_url() {
    let mut config = Config::default();
    config.database.url = "   ".to_string();

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Database URL cannot be empty")
    );
}

#[test]
fn test_validate_empty_server_host() {
    let mut config = Config::default();
    config.server.host = String::new();

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Server host cannot be empty")
    );
}

#[test]
fn test_validate_whitespace_server_host() {
    let mut config = Config::default();
    config.server.host = "   ".to_string();

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Server host cannot be empty")
    );
}

#[test]
fn test_validate_zero_server_port() {
    let mut config = Config::default();
    config.server.port = 0;

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Server port must be greater than 0")
    );
}

#[test]
fn test_validate_valid_git_urls() {
    let valid_urls = vec![
        "https://github.com/user/repo.git",
        "http://example.com/repo.git",
        "git://github.com/user/repo.git",
        "ssh://git@github.com/user/repo.git",
        "git@github.com:user/repo.git",
    ];

    for url in valid_urls {
        let mut config = Config::default();
        config.git.repository_url = Some(url.to_string());

        let result = config.validate();
        assert!(result.is_ok(), "URL should be valid: {url}");
    }
}

#[test]
fn test_validate_invalid_git_url() {
    let mut config = Config::default();
    config.git.repository_url = Some("invalid-url".to_string());

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid Git repository URL format")
    );
}

#[test]
fn test_validate_empty_git_url() {
    let mut config = Config::default();
    config.git.repository_url = Some(String::new());

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_whitespace_git_url() {
    let mut config = Config::default();
    config.git.repository_url = Some("   ".to_string());

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_none_git_url() {
    let mut config = Config::default();
    config.git.repository_url = None;

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_valid_domain() {
    let mut config = Config::default();
    config.domain.default_domain = Some("example.com".to_string());

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_domain_with_space() {
    let mut config = Config::default();
    config.domain.default_domain = Some("example .com".to_string());

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid domain format")
    );
}

#[test]
fn test_validate_domain_starting_with_dot() {
    let mut config = Config::default();
    config.domain.default_domain = Some(".example.com".to_string());

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid domain format")
    );
}

#[test]
fn test_validate_empty_domain() {
    let mut config = Config::default();
    config.domain.default_domain = Some(String::new());

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_whitespace_domain() {
    let mut config = Config::default();
    config.domain.default_domain = Some("   ".to_string());

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_none_domain() {
    let mut config = Config::default();
    config.domain.default_domain = None;

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_and_adjust_warning_messages() {
    let mut config = Config::default();
    config.server.max_request_size = 500; // Below minimum of 1024

    let warnings = config.validate_and_adjust();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("500"));
    assert!(warnings[0].contains("1024"));
    assert!(warnings[0].contains("adjusting to minimum"));
}
