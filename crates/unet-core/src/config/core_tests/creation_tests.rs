//! Tests for configuration creation and defaults

use super::super::core::Config;
use super::super::defaults;

#[test]
fn test_config_new() {
    let config = Config::new();
    assert_eq!(
        config.database.url,
        defaults::database::DEFAULT_DATABASE_URL
    );
    assert_eq!(config.logging.level, defaults::logging::DEFAULT_LOG_LEVEL);
    assert_eq!(config.server.host, defaults::server::DEFAULT_SERVER_HOST);
}

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(
        config.database.url,
        defaults::database::DEFAULT_DATABASE_URL
    );
    assert_eq!(
        config.database.max_connections,
        Some(defaults::database::DEFAULT_DB_MAX_CONNECTIONS)
    );
    assert_eq!(
        config.database.timeout,
        Some(defaults::database::DEFAULT_DB_TIMEOUT_SECONDS)
    );

    assert_eq!(config.logging.level, defaults::logging::DEFAULT_LOG_LEVEL);
    assert_eq!(config.logging.format, defaults::logging::DEFAULT_LOG_FORMAT);
    assert!(config.logging.file.is_none());

    assert_eq!(
        config.snmp.community,
        defaults::snmp::DEFAULT_SNMP_COMMUNITY
    );
    assert_eq!(
        config.snmp.timeout,
        defaults::snmp::DEFAULT_SNMP_TIMEOUT_SECONDS
    );
    assert_eq!(config.snmp.retries, defaults::snmp::DEFAULT_SNMP_RETRIES);

    assert_eq!(config.server.host, defaults::server::DEFAULT_SERVER_HOST);
    assert_eq!(config.server.port, defaults::network::DEFAULT_SERVER_PORT);
    assert_eq!(
        config.server.max_request_size,
        defaults::server::DEFAULT_MAX_REQUEST_SIZE
    );

    assert_eq!(config.git.branch, defaults::git::DEFAULT_GIT_BRANCH);
    assert_eq!(
        config.git.sync_interval,
        defaults::git::DEFAULT_SYNC_INTERVAL_SECONDS
    );
    assert!(config.git.repository_url.is_none());
    assert_eq!(config.git.local_directory, Some("./policies".to_string()));

    assert!(!config.auth.enabled);
    assert_eq!(
        config.auth.token_expiry,
        defaults::auth::DEFAULT_TOKEN_EXPIRY_SECONDS
    );
}
