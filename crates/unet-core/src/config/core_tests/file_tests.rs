//! Tests for configuration file loading and saving

use super::super::core::Config;
use tempfile::NamedTempFile;

#[test]
fn test_config_from_file_valid_toml() {
    let toml_content = r#"
[database]
url = "sqlite://test.db"
max_connections = 10
timeout = 30

[logging]
level = "debug"
format = "json"

[server]
host = "127.0.0.1"
port = 8080
max_request_size = 1048576

[snmp]
community = "test"
timeout = 10
retries = 5

[git]
branch = "develop"
sync_interval = 600

[domain]
search_domains = []

[auth]
enabled = true
token_expiry = 7200
"#;

    let temp_file = NamedTempFile::with_suffix(".toml").unwrap();
    std::fs::write(temp_file.path(), toml_content).unwrap();

    let config = Config::from_file(temp_file.path()).unwrap();

    assert_eq!(config.database.url, "sqlite://test.db");
    assert_eq!(config.database.max_connections, Some(10));
    assert_eq!(config.database.timeout, Some(30));

    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.logging.format, "json");

    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.max_request_size, 1_048_576);

    assert_eq!(config.snmp.community, "test");
    assert_eq!(config.snmp.timeout, 10);
    assert_eq!(config.snmp.retries, 5);

    assert_eq!(config.git.branch, "develop");
    assert_eq!(config.git.sync_interval, 600);

    assert!(config.auth.enabled);
    assert_eq!(config.auth.token_expiry, 7200);
}

#[test]
fn test_config_from_file_nonexistent() {
    let result = Config::from_file("/nonexistent/path/config.toml");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Failed to load configuration"));
}

#[test]
fn test_config_from_file_invalid_toml() {
    let invalid_toml = "invalid toml content [[[";
    let temp_file = NamedTempFile::with_suffix(".toml").unwrap();
    std::fs::write(temp_file.path(), invalid_toml).unwrap();

    let result = Config::from_file(temp_file.path());
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Failed to load configuration"));
}

#[test]
fn test_config_from_file_invalid_structure() {
    let invalid_structure = r#"
[database]
url = "sqlite://test.db"
max_connections = "not_a_number"
"#;
    let temp_file = NamedTempFile::with_suffix(".toml").unwrap();
    std::fs::write(temp_file.path(), invalid_structure).unwrap();

    let result = Config::from_file(temp_file.path());
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Failed to parse configuration"));
}

#[test]
fn test_config_save_to_file() {
    let config = Config::new();
    let temp_file = NamedTempFile::with_suffix(".toml").unwrap();

    let result = config.save_to_file(temp_file.path());
    assert!(result.is_ok());

    // Verify file was created and contains TOML content
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("[database]"));
    assert!(content.contains("[logging]"));
}

#[test]
fn test_config_save_to_file_invalid_path() {
    let config = Config::new();
    let result = config.save_to_file("/invalid/path/that/does/not/exist/config.toml");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Failed to write config"));
}

#[test]
fn test_config_roundtrip_save_and_load() {
    let original_config = Config::new();
    let temp_file = NamedTempFile::with_suffix(".toml").unwrap();

    // Save config to file
    original_config.save_to_file(temp_file.path()).unwrap();

    // Load config from file
    let loaded_config = Config::from_file(temp_file.path()).unwrap();

    // Compare key fields
    assert_eq!(original_config.database.url, loaded_config.database.url);
    assert_eq!(original_config.logging.level, loaded_config.logging.level);
    assert_eq!(original_config.server.host, loaded_config.server.host);
    assert_eq!(original_config.server.port, loaded_config.server.port);
    assert_eq!(original_config.git.branch, loaded_config.git.branch);
}
