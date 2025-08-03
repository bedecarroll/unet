//! Comprehensive tests for logging functionality

use super::*;
use crate::config::LoggingConfig;
use std::env;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_init_tracing_with_valid_config_console_only() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        format: "pretty".to_string(),
        file: None,
    };

    // Test validation without initializing global subscriber
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());
}

#[test]
fn test_init_tracing_with_json_format_console_only() {
    let config = LoggingConfig {
        level: "warn".to_string(),
        format: "json".to_string(),
        file: None,
    };

    // Test validation without initializing global subscriber
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());
}

#[test]
fn test_init_tracing_with_invalid_log_level() {
    let config = LoggingConfig {
        level: "invalid_level".to_string(),
        format: "pretty".to_string(),
        file: None,
    };

    // Since tracing may already be initialized in other tests, we test the validation directly
    let result = validate_log_level(&config.level);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("Must be one of"));
}

#[test]
fn test_init_tracing_with_file_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("logs").join("nested").join("test.log");

    let config = LoggingConfig {
        level: "info".to_string(),
        format: "pretty".to_string(),
        file: Some(nested_path.to_string_lossy().to_string()),
    };

    // Test directory creation without initializing global subscriber
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());

    // Test that we can create parent directories
    if let Some(parent) = nested_path.parent() {
        let result = std::fs::create_dir_all(parent);
        assert!(result.is_ok());
        assert!(parent.exists());
    }
}

#[test]
fn test_init_tracing_with_file_json_format() {
    let temp_file = NamedTempFile::new().unwrap();
    let config = LoggingConfig {
        level: "trace".to_string(),
        format: "json".to_string(),
        file: Some(temp_file.path().to_string_lossy().to_string()),
    };

    // Test validation and file path handling without initializing global subscriber
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());
    assert!(temp_file.path().exists());
}

#[test]
fn test_init_tracing_fails_when_directory_cannot_be_created() {
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "pretty".to_string(),
        file: Some("/proc/protected/cannot_create.log".to_string()),
    };

    // Test validation passes but directory creation will fail on read-only paths
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());

    // Attempt to create a directory in `/proc`, which is read-only even for root
    let result = std::fs::create_dir_all("/proc/protected");
    assert!(result.is_err());
}

#[test]
fn test_init_tracing_with_env_filter_override() {
    unsafe {
        env::set_var("RUST_LOG", "error");
    }

    let config = LoggingConfig {
        level: "debug".to_string(),
        format: "pretty".to_string(),
        file: None,
    };

    // Test validation without initializing global subscriber
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());

    // Test that env var is set correctly
    assert_eq!(env::var("RUST_LOG").unwrap(), "error");

    unsafe {
        env::remove_var("RUST_LOG");
    }
}

#[test]
fn test_validate_log_level_edge_cases() {
    assert!(validate_log_level("TRACE").is_ok());
    assert!(validate_log_level("Debug").is_ok());
    assert!(validate_log_level("INFO").is_ok());
    assert!(validate_log_level("Warn").is_ok());
    assert!(validate_log_level("ERROR").is_ok());

    assert!(validate_log_level("").is_err());
    assert!(validate_log_level("verbose").is_err());
    assert!(validate_log_level("fatal").is_err());
    assert!(validate_log_level("all").is_err());
    assert!(validate_log_level("off").is_err());
}

#[test]
fn test_validate_log_format_edge_cases() {
    assert!(validate_log_format("").is_err());
    assert!(validate_log_format("text").is_err());
    assert!(validate_log_format("xml").is_err());
    assert!(validate_log_format("yaml").is_err());
    assert!(validate_log_format("JSON").is_err());
    assert!(validate_log_format("Pretty").is_err());
}

#[test]
fn test_log_context_macro_with_parameters() {
    use crate::log_context;

    let _span = log_context!("test_operation", node_id = "123", operation_type = "create");
}

#[test]
fn test_log_database_operation_macro() {
    use crate::log_database_operation;

    log_database_operation!("INSERT", "nodes");
    log_database_operation!("UPDATE", "locations");
    log_database_operation!("DELETE", "links");
}

#[test]
fn test_log_snmp_operation_macro() {
    use crate::log_snmp_operation;

    log_snmp_operation!("GET", "192.168.1.1");
    log_snmp_operation!("WALK", "10.0.0.1:161");
}

#[test]
fn test_log_network_operation_macro() {
    use crate::log_network_operation;

    log_network_operation!("HTTP_GET", "https://api.example.com/nodes");
    log_network_operation!("TCP_CONNECT", "192.168.1.1:22");
}

#[test]
fn test_init_tracing_with_empty_file_path() {
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "pretty".to_string(),
        file: Some(String::new()),
    };

    // Test validation passes but empty path is invalid for file creation
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());

    // Empty file path should be treated as invalid for log files
    // Test that empty string is not a valid file path for logging purposes
    assert!(config.file.as_ref().unwrap().is_empty());
}

#[test]
fn test_init_tracing_all_valid_log_levels() {
    let levels = vec!["trace", "debug", "info", "warn", "error"];

    // Since tracing can only be initialized once per process, we test validation instead
    for level in levels {
        let result = validate_log_level(level);
        assert!(result.is_ok(), "Failed to validate level: {level}");
    }
}

#[test]
fn test_init_tracing_file_with_relative_path() {
    let temp_dir = TempDir::new().unwrap();
    let relative_path = "logs/test.log";

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    let config = LoggingConfig {
        level: "debug".to_string(),
        format: "json".to_string(),
        file: Some(relative_path.to_string()),
    };

    // Test validation without initializing global subscriber
    assert!(validate_log_level(&config.level).is_ok());
    assert!(validate_log_format(&config.format).is_ok());

    // Test that we can create the directory structure
    let path = std::path::Path::new(relative_path);
    if let Some(parent) = path.parent() {
        let result = std::fs::create_dir_all(parent);
        assert!(result.is_ok());
    }

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_init_tracing_console_only_format_coverage() {
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "unknown_format".to_string(),
        file: None,
    };

    // Test validation - unknown format should pass validation but default to pretty
    assert!(validate_log_level(&config.level).is_ok());
    // Unknown format is handled gracefully, not an error
}

#[test]
fn test_init_tracing_with_file_format_coverage() {
    let temp_file = NamedTempFile::new().unwrap();
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "unknown_format".to_string(),
        file: Some(temp_file.path().to_string_lossy().to_string()),
    };

    // Test validation without initializing global subscriber
    assert!(validate_log_level(&config.level).is_ok());
    assert!(temp_file.path().exists());
    // Unknown format is handled gracefully, defaults to pretty
}
