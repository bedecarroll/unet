//! Helper functions for creating standardized error messages

use crate::Error;

/// Create a configuration error with standardized messaging
#[must_use]
pub fn config_error(template: &str, args: &[(&str, &str)]) -> Error {
    let message = format_template(template, args);
    Error::config(message)
}

/// Create a database error with standardized messaging
#[must_use]
pub fn database_error(operation: &str, template: &str, args: &[(&str, &str)]) -> Error {
    let message = format_template(template, args);
    Error::database(operation, message)
}

/// Create a policy error with standardized messaging  
#[must_use]
pub fn policy_error(rule: &str, template: &str, args: &[(&str, &str)]) -> Error {
    let message = format_template(template, args);
    Error::Policy {
        rule: rule.to_string(),
        message,
        source: None,
    }
}

/// Create a network error with standardized messaging
#[must_use]
pub fn network_error(endpoint: &str, template: &str, args: &[(&str, &str)]) -> Error {
    let message = format_template(template, args);
    Error::network(endpoint, message)
}

/// Create a validation error with standardized messaging
#[must_use]
pub fn validation_error(field: &str, template: &str, args: &[(&str, &str)]) -> Error {
    let message = format_template(template, args);
    Error::validation(field, message)
}

/// Format a template string with named arguments
///
/// This is a simple template formatter that replaces {key} with values
/// from the args slice. For more complex formatting needs, consider using
/// a dedicated template engine.
#[must_use]
pub fn format_template(template: &str, args: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in args {
        let placeholder = format!("{{{key}}}");
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::super::constants;
    use super::super::templates;
    use super::*;
    use crate::Error;

    #[test]
    fn test_config_error_codes() {
        assert_eq!(constants::CONFIG_INVALID_PATH, "CONFIG_INVALID_PATH");
        assert_eq!(constants::CONFIG_PARSE_FAILED, "CONFIG_PARSE_FAILED");
        assert_eq!(
            constants::CONFIG_VALIDATION_FAILED,
            "CONFIG_VALIDATION_FAILED"
        );
        assert_eq!(
            constants::CONFIG_ENV_OVERRIDE_FAILED,
            "CONFIG_ENV_OVERRIDE_FAILED"
        );
    }

    #[test]
    fn test_database_error_codes() {
        assert_eq!(constants::DB_CONNECTION_FAILED, "DB_CONNECTION_FAILED");
        assert_eq!(constants::DB_QUERY_FAILED, "DB_QUERY_FAILED");
        assert_eq!(constants::DB_TRANSACTION_FAILED, "DB_TRANSACTION_FAILED");
        assert_eq!(
            constants::DB_CONSTRAINT_VIOLATION,
            "DB_CONSTRAINT_VIOLATION"
        );
        assert_eq!(constants::DB_TIMEOUT, "DB_TIMEOUT");
    }

    #[test]
    fn test_policy_error_codes() {
        assert_eq!(constants::POLICY_PARSE_FAILED, "POLICY_PARSE_FAILED");
        assert_eq!(
            constants::POLICY_VALIDATION_FAILED,
            "POLICY_VALIDATION_FAILED"
        );
        assert_eq!(
            constants::POLICY_EXECUTION_FAILED,
            "POLICY_EXECUTION_FAILED"
        );
        assert_eq!(
            constants::POLICY_CONDITION_FAILED,
            "POLICY_CONDITION_FAILED"
        );
    }

    #[test]
    fn test_template_error_codes() {
        assert_eq!(constants::TEMPLATE_PARSE_FAILED, "TEMPLATE_PARSE_FAILED");
        assert_eq!(constants::TEMPLATE_RENDER_FAILED, "TEMPLATE_RENDER_FAILED");
        assert_eq!(constants::TEMPLATE_NOT_FOUND, "TEMPLATE_NOT_FOUND");
    }

    #[test]
    fn test_snmp_error_codes() {
        assert_eq!(constants::SNMP_CONNECTION_FAILED, "SNMP_CONNECTION_FAILED");
        assert_eq!(constants::SNMP_AUTH_FAILED, "SNMP_AUTH_FAILED");
        assert_eq!(constants::SNMP_TIMEOUT, "SNMP_TIMEOUT");
        assert_eq!(constants::SNMP_INVALID_OID, "SNMP_INVALID_OID");
    }

    #[test]
    fn test_network_error_codes() {
        assert_eq!(constants::NET_INVALID_ADDRESS, "NET_INVALID_ADDRESS");
        assert_eq!(constants::NET_CONNECTION_REFUSED, "NET_CONNECTION_REFUSED");
        assert_eq!(
            constants::NET_DNS_RESOLUTION_FAILED,
            "NET_DNS_RESOLUTION_FAILED"
        );
    }

    #[test]
    fn test_validation_error_codes() {
        assert_eq!(
            constants::VALID_REQUIRED_FIELD_MISSING,
            "VALID_REQUIRED_FIELD_MISSING"
        );
        assert_eq!(constants::VALID_INVALID_FORMAT, "VALID_INVALID_FORMAT");
        assert_eq!(
            constants::VALID_VALUE_OUT_OF_RANGE,
            "VALID_VALUE_OUT_OF_RANGE"
        );
    }

    #[test]
    fn test_io_error_codes() {
        assert_eq!(constants::IO_FILE_NOT_FOUND, "IO_FILE_NOT_FOUND");
        assert_eq!(constants::IO_PERMISSION_DENIED, "IO_PERMISSION_DENIED");
        assert_eq!(constants::IO_DISK_FULL, "IO_DISK_FULL");
    }

    #[test]
    fn test_serialization_error_codes() {
        assert_eq!(constants::SERIAL_JSON_FAILED, "SERIAL_JSON_FAILED");
        assert_eq!(constants::SERIAL_TOML_FAILED, "SERIAL_TOML_FAILED");
        assert_eq!(constants::SERIAL_YAML_FAILED, "SERIAL_YAML_FAILED");
    }

    #[test]
    fn test_config_templates() {
        assert!(templates::config::INVALID_PATH.contains("{path}"));
        assert!(templates::config::PARSE_FAILED.contains("{file}"));
        assert!(templates::config::PARSE_FAILED.contains("{error}"));
        assert!(templates::config::VALIDATION_FAILED.contains("{field}"));
        assert!(templates::config::VALIDATION_FAILED.contains("{reason}"));
        assert!(templates::config::ENV_OVERRIDE_FAILED.contains("{var}"));
        assert!(templates::config::ENV_OVERRIDE_FAILED.contains("{error}"));
    }

    #[test]
    fn test_database_templates() {
        assert!(templates::database::CONNECTION_FAILED.contains("{url}"));
        assert!(templates::database::CONNECTION_FAILED.contains("{error}"));
        assert!(templates::database::QUERY_FAILED.contains("{operation}"));
        assert!(templates::database::QUERY_FAILED.contains("{error}"));
        assert!(templates::database::TRANSACTION_FAILED.contains("{operation}"));
        assert!(templates::database::TRANSACTION_FAILED.contains("{error}"));
        assert!(templates::database::CONSTRAINT_VIOLATION.contains("{table}"));
        assert!(templates::database::CONSTRAINT_VIOLATION.contains("{constraint}"));
        assert!(templates::database::TIMEOUT.contains("{operation}"));
        assert!(templates::database::TIMEOUT.contains("{seconds}"));
    }

    #[test]
    fn test_policy_templates() {
        assert!(templates::policy::PARSE_FAILED.contains("{rule}"));
        assert!(templates::policy::PARSE_FAILED.contains("{error}"));
        assert!(templates::policy::VALIDATION_FAILED.contains("{rule}"));
        assert!(templates::policy::VALIDATION_FAILED.contains("{reason}"));
        assert!(templates::policy::EXECUTION_FAILED.contains("{rule}"));
        assert!(templates::policy::EXECUTION_FAILED.contains("{error}"));
        assert!(templates::policy::CONDITION_FAILED.contains("{condition}"));
    }

    #[test]
    fn test_template_templates() {
        assert!(templates::template::PARSE_FAILED.contains("{template}"));
        assert!(templates::template::PARSE_FAILED.contains("{error}"));
        assert!(templates::template::RENDER_FAILED.contains("{template}"));
        assert!(templates::template::RENDER_FAILED.contains("{error}"));
        assert!(templates::template::NOT_FOUND.contains("{template}"));
    }

    #[test]
    fn test_snmp_templates() {
        assert!(templates::snmp::CONNECTION_FAILED.contains("{address}"));
        assert!(templates::snmp::CONNECTION_FAILED.contains("{error}"));
        assert!(templates::snmp::AUTH_FAILED.contains("{address}"));
        assert!(templates::snmp::AUTH_FAILED.contains("{error}"));
        assert!(templates::snmp::TIMEOUT.contains("{address}"));
        assert!(templates::snmp::TIMEOUT.contains("{seconds}"));
        assert!(templates::snmp::INVALID_OID.contains("{oid}"));
        assert!(templates::snmp::INVALID_OID.contains("{reason}"));
    }

    #[test]
    fn test_network_templates() {
        assert!(templates::network::INVALID_ADDRESS.contains("{address}"));
        assert!(templates::network::INVALID_ADDRESS.contains("{reason}"));
        assert!(templates::network::CONNECTION_REFUSED.contains("{address}"));
        assert!(templates::network::CONNECTION_REFUSED.contains("{port}"));
        assert!(templates::network::DNS_RESOLUTION_FAILED.contains("{hostname}"));
        assert!(templates::network::DNS_RESOLUTION_FAILED.contains("{error}"));
    }

    #[test]
    fn test_validation_templates() {
        assert!(templates::validation::REQUIRED_FIELD_MISSING.contains("{field}"));
        assert!(templates::validation::INVALID_FORMAT.contains("{field}"));
        assert!(templates::validation::INVALID_FORMAT.contains("{expected}"));
        assert!(templates::validation::VALUE_OUT_OF_RANGE.contains("{field}"));
        assert!(templates::validation::VALUE_OUT_OF_RANGE.contains("{value}"));
        assert!(templates::validation::VALUE_OUT_OF_RANGE.contains("{min}"));
        assert!(templates::validation::VALUE_OUT_OF_RANGE.contains("{max}"));
    }

    #[test]
    fn test_io_templates() {
        assert!(templates::io::FILE_NOT_FOUND.contains("{path}"));
        assert!(templates::io::PERMISSION_DENIED.contains("{path}"));
        assert!(templates::io::DISK_FULL.contains("{path}"));
    }

    #[test]
    fn test_serialization_templates() {
        assert!(templates::serialization::JSON_FAILED.contains("{type}"));
        assert!(templates::serialization::JSON_FAILED.contains("{error}"));
        assert!(templates::serialization::TOML_FAILED.contains("{type}"));
        assert!(templates::serialization::TOML_FAILED.contains("{error}"));
        assert!(templates::serialization::YAML_FAILED.contains("{type}"));
        assert!(templates::serialization::YAML_FAILED.contains("{error}"));
    }

    #[test]
    fn test_config_error_helper() {
        let error = config_error(
            templates::config::INVALID_PATH,
            &[("path", "/invalid/path")],
        );

        match error {
            Error::Config { message, .. } => {
                assert!(message.contains("/invalid/path"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_database_error_helper() {
        let error = database_error(
            "SELECT",
            templates::database::QUERY_FAILED,
            &[("operation", "SELECT"), ("error", "timeout")],
        );

        match error {
            Error::Database {
                operation, message, ..
            } => {
                assert_eq!(operation, "SELECT");
                assert!(message.contains("timeout"));
            }
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_policy_error_helper() {
        let error = policy_error(
            "test_rule",
            templates::policy::PARSE_FAILED,
            &[("rule", "test_rule"), ("error", "syntax error")],
        );

        match error {
            Error::Policy { rule, message, .. } => {
                assert_eq!(rule, "test_rule");
                assert!(message.contains("syntax error"));
            }
            _ => panic!("Expected Policy error"),
        }
    }

    #[test]
    fn test_network_error_helper() {
        let error = network_error(
            "192.168.1.1",
            templates::network::CONNECTION_REFUSED,
            &[("address", "192.168.1.1"), ("port", "80")],
        );

        match error {
            Error::Network {
                endpoint, message, ..
            } => {
                assert_eq!(endpoint, "192.168.1.1");
                assert!(message.contains("192.168.1.1"));
                assert!(message.contains("80"));
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_validation_error_helper() {
        let error = validation_error(
            "username",
            templates::validation::REQUIRED_FIELD_MISSING,
            &[("field", "username")],
        );

        match error {
            Error::Validation { field, message, .. } => {
                assert_eq!(field, "username");
                assert!(message.contains("username"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_format_template_multiple_placeholders() {
        let template = "Error in {operation} for {table}: {error}";
        let args = &[
            ("operation", "INSERT"),
            ("table", "users"),
            ("error", "constraint violation"),
        ];

        let result = format_template(template, args);
        assert_eq!(result, "Error in INSERT for users: constraint violation");
    }

    #[test]
    fn test_format_template_no_placeholders() {
        let template = "Simple error message";
        let args = &[];

        let result = format_template(template, args);
        assert_eq!(result, "Simple error message");
    }

    #[test]
    fn test_format_template_unused_args() {
        let template = "Error: {error}";
        let args = &[("error", "timeout"), ("unused", "value")];

        let result = format_template(template, args);
        assert_eq!(result, "Error: timeout");
    }

    #[test]
    fn test_format_template_missing_args() {
        let template = "Error in {operation}: {error}";
        let args = &[("operation", "SELECT")];

        let result = format_template(template, args);
        assert_eq!(result, "Error in SELECT: {error}");
    }
}
