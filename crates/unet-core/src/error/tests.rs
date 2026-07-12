//! Tests for error constructors and methods

#[cfg(test)]
mod error_tests {
    use super::super::Error;
    use std::error::Error as StdError;
    use std::io;

    #[test]
    fn test_config_error() {
        let err = Error::config("Invalid config file");
        assert_eq!(err.error_code(), "CONFIG_ERROR");
        assert_eq!(
            err.user_message(),
            "Configuration problem: Invalid config file"
        );
        assert!(err.to_string().contains("Invalid config file"));
    }

    #[test]
    fn test_config_error_with_source() {
        let source = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let err = Error::config_with_source("Config file missing", source);
        assert_eq!(err.error_code(), "CONFIG_ERROR");
        assert!(
            err.user_message()
                .contains("Configuration problem: Config file missing")
        );
        assert!(StdError::source(&err).is_some());
    }

    #[test]
    fn test_database_error() {
        let err = Error::database("SELECT", "Connection failed");
        assert_eq!(err.error_code(), "DATABASE_ERROR");
        assert_eq!(
            err.user_message(),
            "Database problem during SELECT: Connection failed"
        );
        assert!(err.to_string().contains("SELECT"));
        assert!(err.to_string().contains("Connection failed"));
    }

    #[test]
    fn test_database_error_with_source() {
        let source = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
        let err = Error::database_with_source("INSERT", "Failed to insert", source);
        assert_eq!(err.error_code(), "DATABASE_ERROR");
        assert!(
            err.user_message()
                .contains("Database problem during INSERT: Failed to insert")
        );
        assert!(StdError::source(&err).is_some());
    }

    #[test]
    fn test_validation_error() {
        let err = Error::validation("ip_address", "Invalid IP format");
        assert_eq!(err.error_code(), "VALIDATION_ERROR");
        assert_eq!(err.user_message(), "Invalid ip_address: Invalid IP format");
        assert!(err.to_string().contains("ip_address"));
        assert!(err.to_string().contains("Invalid IP format"));
    }

    #[test]
    fn test_validation_error_with_value() {
        let err = Error::validation_with_value("port", "Port out of range", "99999");
        assert_eq!(err.error_code(), "VALIDATION_ERROR");
        assert_eq!(
            err.user_message(),
            "Invalid value '99999' for port: Port out of range"
        );
        assert!(err.to_string().contains("port"));
        assert!(err.to_string().contains("Port out of range"));
    }

    #[test]
    fn test_snmp_error() {
        let err = Error::snmp("192.168.1.1", "Timeout");
        assert_eq!(err.error_code(), "SNMP_ERROR");
        assert_eq!(err.user_message(), "SNMP error for 192.168.1.1: Timeout");
        assert!(err.to_string().contains("192.168.1.1"));
        assert!(err.to_string().contains("Timeout"));
    }

    #[test]
    fn test_snmp_error_with_source() {
        let source = io::Error::new(io::ErrorKind::TimedOut, "Request timeout");
        let err = Error::snmp_with_source("switch-01", "SNMP timeout", source);
        assert_eq!(err.error_code(), "SNMP_ERROR");
        assert!(
            err.user_message()
                .contains("SNMP error for switch-01: SNMP timeout")
        );
        assert!(StdError::source(&err).is_some());
    }

    #[test]
    fn test_network_error() {
        let err = Error::network("api.example.com", "Connection refused");
        assert_eq!(err.error_code(), "NETWORK_ERROR");
        assert_eq!(
            err.user_message(),
            "Network error connecting to api.example.com: Connection refused"
        );
        assert!(err.to_string().contains("api.example.com"));
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_network_error_with_source() {
        let source = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
        let err = Error::network_with_source("127.0.0.1:8080", "Cannot connect", source);
        assert_eq!(err.error_code(), "NETWORK_ERROR");
        assert!(
            err.user_message()
                .contains("Network error connecting to 127.0.0.1:8080: Cannot connect")
        );
        assert!(StdError::source(&err).is_some());
    }

    #[test]
    fn test_error_code_coverage() {
        // Test all error code variants
        let config_err = Error::config("test");
        let db_err = Error::database("test", "test");
        let validation_err = Error::validation("test", "test");
        let snmp_err = Error::snmp("test", "test");
        let network_err = Error::network("test", "test");

        // Test Policy error variant
        let policy_err = Error::Policy {
            rule: "test_rule".to_string(),
            message: "test message".to_string(),
            source: None,
        };

        // Test Template error variant
        let template_err = Error::Template {
            template: "test_template".to_string(),
            message: "test message".to_string(),
            source: None,
        };

        // Test IO error variant
        let io_err = Error::Io {
            path: "/test/path".to_string(),
            message: "test message".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "not found"),
        };

        // Test Serialization error variant
        let serialization_err = Error::Serialization {
            format: "JSON".to_string(),
            message: "test message".to_string(),
            source: Box::new(io::Error::new(io::ErrorKind::InvalidData, "invalid data")),
        };

        // Test Other error variant
        let other_err = Error::Other {
            context: "test_context".to_string(),
            message: "test message".to_string(),
            source: None,
        };

        // Verify error codes
        assert_eq!(config_err.error_code(), "CONFIG_ERROR");
        assert_eq!(db_err.error_code(), "DATABASE_ERROR");
        assert_eq!(policy_err.error_code(), "POLICY_ERROR");
        assert_eq!(template_err.error_code(), "TEMPLATE_ERROR");
        assert_eq!(snmp_err.error_code(), "SNMP_ERROR");
        assert_eq!(validation_err.error_code(), "VALIDATION_ERROR");
        assert_eq!(network_err.error_code(), "NETWORK_ERROR");
        assert_eq!(io_err.error_code(), "IO_ERROR");
        assert_eq!(serialization_err.error_code(), "SERIALIZATION_ERROR");
        assert_eq!(other_err.error_code(), "OTHER_ERROR");
    }

    #[test]
    fn test_user_message_coverage() {
        // Test all user message variants
        let policy_err = Error::Policy {
            rule: "test_rule".to_string(),
            message: "test message".to_string(),
            source: None,
        };

        let template_err = Error::Template {
            template: "test_template".to_string(),
            message: "test message".to_string(),
            source: None,
        };

        let io_err = Error::Io {
            path: "/test/path".to_string(),
            message: "test message".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "not found"),
        };

        let serialization_err = Error::Serialization {
            format: "JSON".to_string(),
            message: "test message".to_string(),
            source: Box::new(io::Error::new(io::ErrorKind::InvalidData, "invalid data")),
        };

        let other_err = Error::Other {
            context: "test_context".to_string(),
            message: "test message".to_string(),
            source: None,
        };

        // Verify user messages
        assert_eq!(
            policy_err.user_message(),
            "Policy rule 'test_rule' failed: test message"
        );
        assert_eq!(
            template_err.user_message(),
            "Template 'test_template' failed: test message"
        );
        assert_eq!(
            io_err.user_message(),
            "File error with '/test/path': test message"
        );
        assert_eq!(
            serialization_err.user_message(),
            "JSON format error: test message"
        );
        assert_eq!(
            other_err.user_message(),
            "Error in test_context: test message"
        );
    }
}
