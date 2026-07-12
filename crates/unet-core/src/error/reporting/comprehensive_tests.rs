//! Comprehensive tests for missing error reporting functionality

use super::super::{Error, Result};
use super::core::ErrorReporting;

#[test]
fn test_error_log_template_error() {
    let error = Error::Template {
        template: "node_config.j2".to_string(),
        message: "Template rendering failed".to_string(),
        source: None,
    };
    error.log(); // Should not panic

    match error {
        Error::Template {
            template, message, ..
        } => {
            assert_eq!(template, "node_config.j2");
            assert_eq!(message, "Template rendering failed");
        }
        _ => panic!("Expected Template error"),
    }
}

#[test]
fn test_error_log_snmp_error() {
    let error = Error::snmp("192.168.1.1", "SNMP timeout");
    error.log(); // Should not panic

    match error {
        Error::Snmp {
            target, message, ..
        } => {
            assert_eq!(target, "192.168.1.1");
            assert_eq!(message, "SNMP timeout");
        }
        _ => panic!("Expected SNMP error"),
    }
}

#[test]
fn test_error_log_validation_error_without_value() {
    let error = Error::Validation {
        field: "email".to_string(),
        message: "Required field".to_string(),
        value: None,
    };
    error.log(); // Should not panic - tests the None branch

    match error {
        Error::Validation {
            field,
            message,
            value,
        } => {
            assert_eq!(field, "email");
            assert_eq!(message, "Required field");
            assert_eq!(value, None);
        }
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_error_reporting_with_template_error() {
    let result: Result<String> = Err(Error::Template {
        template: "test.j2".to_string(),
        message: "Syntax error".to_string(),
        source: None,
    });

    let logged_result = result.log_error();
    assert!(logged_result.is_err());

    let user_result = logged_result.user_friendly();
    assert!(user_result.is_err());
    let error_msg = user_result.unwrap_err();
    assert!(error_msg.contains("test.j2") || error_msg.contains("Syntax error"));
}

#[test]
fn test_error_reporting_with_snmp_error() {
    let result: Result<String> = Err(Error::snmp("10.0.0.1", "Community string invalid"));

    let logged_result = result.log_error_with_context("SNMP polling");
    assert!(logged_result.is_err());

    match logged_result.unwrap_err() {
        Error::Snmp {
            target, message, ..
        } => {
            assert_eq!(target, "10.0.0.1");
            assert_eq!(message, "Community string invalid");
        }
        _ => panic!("Expected SNMP error"),
    }
}

#[test]
fn test_error_reporting_chaining_with_all_error_types() {
    // Test config error chaining
    let config_result: Result<i32> = Err(Error::config("Config issue"));
    let final_config = config_result.log_error().user_friendly();
    assert!(final_config.is_err());

    // Test database error chaining
    let db_result: Result<i32> = Err(Error::database("UPDATE", "Lock timeout"));
    let final_db = db_result.log_error().user_friendly();
    assert!(final_db.is_err());

    // Test network error chaining
    let net_result: Result<i32> = Err(Error::network("example.com", "DNS resolution failed"));
    let final_net = net_result.log_error().user_friendly();
    assert!(final_net.is_err());

    // Test IO error chaining
    let io_result: Result<i32> = Err(Error::Io {
        path: "/test/path".to_string(),
        message: "Permission denied".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied"),
    });
    let final_io = io_result.log_error().user_friendly();
    assert!(final_io.is_err());
}

#[test]
fn test_error_logging_all_branches() {
    // Test all error type logging branches
    let errors = vec![
        Error::config("Config test"),
        Error::database("SELECT", "DB test"),
        Error::Policy {
            rule: "test_rule".to_string(),
            message: "Policy test".to_string(),
            source: None,
        },
        Error::Template {
            template: "test.tmpl".to_string(),
            message: "Template test".to_string(),
            source: None,
        },
        Error::snmp("host", "SNMP test"),
        Error::validation("field", "Validation test"),
        Error::network("endpoint", "Network test"),
        Error::Io {
            path: "path".to_string(),
            message: "IO test".to_string(),
            source: std::io::Error::other("test"),
        },
        Error::Serialization {
            format: "JSON".to_string(),
            message: "Serialization test".to_string(),
            source: Box::new(std::io::Error::other("test")),
        },
        Error::Other {
            context: "context".to_string(),
            message: "Other test".to_string(),
            source: None,
        },
    ];

    // Log all errors - should not panic
    for error in errors {
        error.log();
    }
}

#[test]
fn test_error_reporting_context_preservation() {
    let result: Result<String> = Err(Error::config("Original error"));

    // Chain multiple context additions
    let logged_once = result.log_error();
    let logged_twice = logged_once.log_error_with_context("First context");
    let final_result = logged_twice.log_error_with_context("Second context");

    assert!(final_result.is_err());

    // Original error should be preserved
    match final_result.unwrap_err() {
        Error::Config { message, .. } => {
            assert_eq!(message, "Original error");
        }
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn test_validation_error_with_different_value_types() {
    // Test validation error with empty string value
    let error_empty = Error::Validation {
        field: "test".to_string(),
        message: "Test message".to_string(),
        value: Some(String::new()),
    };
    error_empty.log(); // Should handle empty string value

    // Test validation error with long value
    let error_long = Error::Validation {
        field: "test".to_string(),
        message: "Test message".to_string(),
        value: Some("very_long_value_that_might_be_truncated_in_logs".to_string()),
    };
    error_long.log(); // Should handle long value

    // Test validation error with special characters
    let error_special = Error::Validation {
        field: "test".to_string(),
        message: "Test message".to_string(),
        value: Some("value_with_special_chars_!@#$%^&*()".to_string()),
    };
    error_special.log(); // Should handle special characters
}

#[test]
fn test_user_friendly_error_message_content() {
    // Test that user_friendly preserves important information
    let validation_error = Error::validation("email", "Invalid format");
    let user_msg = validation_error.user_message();
    assert!(user_msg.contains("email") || user_msg.contains("Invalid format"));

    let config_error = Error::config("Missing required setting");
    let config_msg = config_error.user_message();
    assert!(config_msg.contains("Missing required setting"));

    let network_error = Error::network("api.example.com", "Connection timeout");
    let network_msg = network_error.user_message();
    assert!(network_msg.contains("api.example.com") || network_msg.contains("Connection timeout"));
}

#[test]
fn test_error_reporting_success_paths() {
    // Ensure success cases don't modify the result
    let success_result: Result<i32> = Ok(42);

    let logged = success_result.log_error();
    assert_eq!(logged.unwrap(), 42);

    let success_result2: Result<String> = Ok("test".to_string());
    let logged_with_context = success_result2.log_error_with_context("test context");
    assert_eq!(logged_with_context.unwrap(), "test");

    let success_result3: Result<bool> = Ok(true);
    let user_friendly = success_result3.user_friendly();
    assert!(user_friendly.unwrap());
}
