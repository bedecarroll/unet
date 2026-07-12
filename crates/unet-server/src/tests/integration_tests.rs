//! Integration tests for server functionality

use crate::config_loader::*;
use std::io::Write;
use std::path::PathBuf;
use unet_core::config::Config;

#[tokio::test]
async fn test_initialize_app_with_overrides() {
    let args = Args {
        config: None,
        host: Some("0.0.0.0".to_string()),
        port: Some(9000),
        database_url: "sqlite://test.db".to_string(),
        log_level: Some("debug".to_string()),
    };

    let result = initialize_app(args).await;
    assert!(result.is_ok());

    let (config, database_url) = result.unwrap();

    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9000);
    assert_eq!(config.logging.level, "debug");
    assert_eq!(database_url, "sqlite://test.db");
}

#[tokio::test]
async fn test_initialize_app_with_defaults() {
    let args = Args {
        config: None,
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };

    let result = initialize_app(args).await;
    assert!(result.is_ok());

    let (config, database_url) = result.unwrap();

    let default_config = Config::default();
    assert_eq!(config.server.host, default_config.server.host);
    assert_eq!(config.server.port, default_config.server.port);
    assert_eq!(config.logging.level, default_config.logging.level);
    assert_eq!(database_url, default_config.database_url());
}

#[tokio::test]
async fn test_initialize_app_invalid_config() {
    let args = Args {
        config: Some(PathBuf::from("/nonexistent/file.toml")),
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };

    let result = initialize_app(args).await;
    assert!(result.is_err());
}

#[test]
fn test_config_loading_with_file() {
    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        temp_file,
        r#"
[database]
url = "sqlite://test.db"

[server]
host = "127.0.0.1"
port = 8080
max_request_size = 1048576

[logging]
level = "info"
format = "text"

[snmp]
community = "public"
timeout = 5
retries = 3

[git]
repository_url = "https://github.com/example/policies.git"
branch = "main"
sync_interval = 300

[domain]
search_domains = []

[auth]
enabled = false
token_expiry = 3600
"#
    )
    .unwrap();

    let config_path = temp_file.path().to_path_buf();
    let args = Args {
        config: Some(config_path),
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };
    let result = load_configuration(&args);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.logging.level, "info");
    assert_eq!(
        config.git.repository_url,
        Some("https://github.com/example/policies.git".to_string())
    );
}

#[test]
fn test_config_validation_success() {
    let mut config = Config::default();
    config.server.port = 8080;
    assert!(config.validate().is_ok());
}

#[test]
fn test_load_configuration_with_valid_path() {
    let mut temp_file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        temp_file,
        r#"
[database]
url = "sqlite://test.db"

[server]
host = "0.0.0.0"
port = 3000
max_request_size = 2097152

[logging]
level = "debug"
format = "json"

[snmp]
community = "public"
timeout = 5
retries = 3

[git]
branch = "main"
sync_interval = 300

[domain]
search_domains = []

[auth]
enabled = false
token_expiry = 3600
"#
    )
    .unwrap();

    let args = Args {
        config: Some(temp_file.path().to_path_buf()),
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };
    let result = load_configuration(&args);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 3000);
}

#[test]
fn test_load_configuration_from_env_fallback() {
    unsafe {
        std::env::set_var("UNET_CONFIG_PATH", "/nonexistent/path.toml");
    }

    let args = Args {
        config: None,
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };
    let result = load_configuration(&args);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.server.host, Config::default().server.host);

    unsafe {
        std::env::remove_var("UNET_CONFIG_PATH");
    }
}

#[test]
fn test_load_configuration_file_not_found() {
    let non_existent_path = PathBuf::from("/this/file/does/not/exist.toml");
    let args = Args {
        config: Some(non_existent_path),
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };
    let result = load_configuration(&args);

    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("not found"));
}
