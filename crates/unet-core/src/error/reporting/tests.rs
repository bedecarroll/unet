//! Tests for error reporting functionality

#[cfg(test)]
mod error_reporting_tests {
    use super::super::super::{Error, Result};
    use super::super::core::ErrorReporting;

    #[test]
    fn test_error_reporting_log_error() {
        let result: Result<i32> = Err(Error::config("Test config error"));
        let logged_result = result.log_error();

        assert!(logged_result.is_err());
        match logged_result.unwrap_err() {
            Error::Config { message, .. } => {
                assert_eq!(message, "Test config error");
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_error_reporting_log_error_success() {
        let result: Result<i32> = Ok(42);
        let logged_result = result.log_error();

        assert!(logged_result.is_ok());
        assert_eq!(logged_result.unwrap(), 42);
    }

    #[test]
    fn test_error_reporting_log_error_with_context() {
        let result: Result<String> = Err(Error::database("INSERT", "Connection failed"));
        let logged_result = result.log_error_with_context("Database operation");

        assert!(logged_result.is_err());
        match logged_result.unwrap_err() {
            Error::Database {
                operation, message, ..
            } => {
                assert_eq!(operation, "INSERT");
                assert_eq!(message, "Connection failed");
            }
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_error_reporting_log_error_with_context_success() {
        let result: Result<String> = Ok("success".to_string());
        let logged_result = result.log_error_with_context("Test operation");

        assert!(logged_result.is_ok());
        assert_eq!(logged_result.unwrap(), "success");
    }

    #[test]
    fn test_error_reporting_user_friendly() {
        let result: Result<i32> = Err(Error::validation("email", "Invalid email format"));
        let user_result = result.user_friendly();

        assert!(user_result.is_err());
        let error_msg = user_result.unwrap_err();
        assert!(error_msg.contains("email"));
        assert!(error_msg.contains("Invalid email format"));
    }

    #[test]
    fn test_error_reporting_user_friendly_success() {
        let result: Result<i32> = Ok(100);
        let user_result = result.user_friendly();

        assert!(user_result.is_ok());
        assert_eq!(user_result.unwrap(), 100);
    }

    #[test]
    fn test_error_log_config_error() {
        let error = Error::config("Configuration file not found");
        error.log(); // Should not panic

        match error {
            Error::Config { message, .. } => {
                assert_eq!(message, "Configuration file not found");
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_error_log_database_error() {
        let error = Error::database("SELECT", "Query timeout");
        error.log(); // Should not panic

        match error {
            Error::Database {
                operation, message, ..
            } => {
                assert_eq!(operation, "SELECT");
                assert_eq!(message, "Query timeout");
            }
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_error_log_policy_error() {
        let error = Error::Policy {
            rule: "test_rule".to_string(),
            message: "Policy validation failed".to_string(),
            source: None,
        };
        error.log(); // Should not panic

        match error {
            Error::Policy { rule, message, .. } => {
                assert_eq!(rule, "test_rule");
                assert_eq!(message, "Policy validation failed");
            }
            _ => panic!("Expected Policy error"),
        }
    }

    #[test]
    fn test_error_log_network_error() {
        let error = Error::network("192.168.1.1", "Connection refused");
        error.log(); // Should not panic

        match error {
            Error::Network {
                endpoint, message, ..
            } => {
                assert_eq!(endpoint, "192.168.1.1");
                assert_eq!(message, "Connection refused");
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_error_log_validation_error() {
        let error = Error::validation("username", "Required field missing");
        error.log(); // Should not panic

        match error {
            Error::Validation { field, message, .. } => {
                assert_eq!(field, "username");
                assert_eq!(message, "Required field missing");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_error_log_validation_error_with_value() {
        let error = Error::Validation {
            field: "age".to_string(),
            message: "Value out of range".to_string(),
            value: Some("150".to_string()),
        };
        error.log(); // Should not panic

        match error {
            Error::Validation {
                field,
                message,
                value,
            } => {
                assert_eq!(field, "age");
                assert_eq!(message, "Value out of range");
                assert_eq!(value, Some("150".to_string()));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_error_log_io_error() {
        let error = Error::Io {
            path: "/tmp/test.txt".to_string(),
            message: "File not found".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        };
        error.log(); // Should not panic

        match error {
            Error::Io { path, message, .. } => {
                assert_eq!(path, "/tmp/test.txt");
                assert_eq!(message, "File not found");
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_error_log_serialization_error() {
        let error = Error::Serialization {
            format: "JSON".to_string(),
            message: "Invalid JSON syntax".to_string(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid JSON",
            )),
        };
        error.log(); // Should not panic

        match error {
            Error::Serialization {
                format, message, ..
            } => {
                assert_eq!(format, "JSON");
                assert_eq!(message, "Invalid JSON syntax");
            }
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_error_log_other_error() {
        let error = Error::Other {
            context: "test_context".to_string(),
            message: "Unknown error".to_string(),
            source: None,
        };
        error.log(); // Should not panic

        match error {
            Error::Other {
                context, message, ..
            } => {
                assert_eq!(context, "test_context");
                assert_eq!(message, "Unknown error");
            }
            _ => panic!("Expected Other error"),
        }
    }

    #[test]
    fn test_chaining_error_reporting_methods() {
        let result: Result<i32> = Err(Error::config("Test error"));
        let final_result = result
            .log_error()
            .log_error_with_context("Chain test")
            .user_friendly();

        assert!(final_result.is_err());
        let error_msg = final_result.unwrap_err();
        assert!(error_msg.contains("Test error"));
    }
}
