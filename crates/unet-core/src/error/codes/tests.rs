//! Tests for error codes and message templates

#[cfg(test)]
mod constants_tests {
    use super::super::codes::constants::*;

    #[test]
    fn test_config_error_codes() {
        assert_eq!(CONFIG_INVALID_PATH, "CONFIG_INVALID_PATH");
        assert_eq!(CONFIG_PARSE_FAILED, "CONFIG_PARSE_FAILED");
        assert_eq!(CONFIG_VALIDATION_FAILED, "CONFIG_VALIDATION_FAILED");
        assert_eq!(CONFIG_ENV_OVERRIDE_FAILED, "CONFIG_ENV_OVERRIDE_FAILED");
    }

    #[test]
    fn test_database_error_codes() {
        assert_eq!(DB_CONNECTION_FAILED, "DB_CONNECTION_FAILED");
        assert_eq!(DB_QUERY_FAILED, "DB_QUERY_FAILED");
        assert_eq!(DB_TRANSACTION_FAILED, "DB_TRANSACTION_FAILED");
        assert_eq!(DB_CONSTRAINT_VIOLATION, "DB_CONSTRAINT_VIOLATION");
        assert_eq!(DB_TIMEOUT, "DB_TIMEOUT");
    }

    #[test]
    fn test_policy_error_codes() {
        assert_eq!(POLICY_PARSE_FAILED, "POLICY_PARSE_FAILED");
        assert_eq!(POLICY_VALIDATION_FAILED, "POLICY_VALIDATION_FAILED");
        assert_eq!(POLICY_EXECUTION_FAILED, "POLICY_EXECUTION_FAILED");
        assert_eq!(POLICY_CONDITION_FAILED, "POLICY_CONDITION_FAILED");
    }

    #[test]
    fn test_template_error_codes() {
        assert_eq!(TEMPLATE_PARSE_FAILED, "TEMPLATE_PARSE_FAILED");
        assert_eq!(TEMPLATE_RENDER_FAILED, "TEMPLATE_RENDER_FAILED");
        assert_eq!(TEMPLATE_NOT_FOUND, "TEMPLATE_NOT_FOUND");
    }

    #[test]
    fn test_snmp_error_codes() {
        assert_eq!(SNMP_CONNECTION_FAILED, "SNMP_CONNECTION_FAILED");
        assert_eq!(SNMP_AUTH_FAILED, "SNMP_AUTH_FAILED");
        assert_eq!(SNMP_TIMEOUT, "SNMP_TIMEOUT");
        assert_eq!(SNMP_INVALID_OID, "SNMP_INVALID_OID");
    }

    #[test]
    fn test_network_error_codes() {
        assert_eq!(NET_INVALID_ADDRESS, "NET_INVALID_ADDRESS");
        assert_eq!(NET_CONNECTION_REFUSED, "NET_CONNECTION_REFUSED");
        assert_eq!(NET_DNS_RESOLUTION_FAILED, "NET_DNS_RESOLUTION_FAILED");
    }

    #[test]
    fn test_validation_error_codes() {
        assert_eq!(VALID_REQUIRED_FIELD_MISSING, "VALID_REQUIRED_FIELD_MISSING");
        assert_eq!(VALID_INVALID_FORMAT, "VALID_INVALID_FORMAT");
        assert_eq!(VALID_VALUE_OUT_OF_RANGE, "VALID_VALUE_OUT_OF_RANGE");
    }

    #[test]
    fn test_io_error_codes() {
        assert_eq!(IO_FILE_NOT_FOUND, "IO_FILE_NOT_FOUND");
        assert_eq!(IO_PERMISSION_DENIED, "IO_PERMISSION_DENIED");
        assert_eq!(IO_DISK_FULL, "IO_DISK_FULL");
    }

    #[test]
    fn test_serialization_error_codes() {
        assert_eq!(SERIAL_JSON_FAILED, "SERIAL_JSON_FAILED");
        assert_eq!(SERIAL_TOML_FAILED, "SERIAL_TOML_FAILED");
        assert_eq!(SERIAL_YAML_FAILED, "SERIAL_YAML_FAILED");
    }
}

#[cfg(test)]
mod templates_tests {
    use super::super::codes::templates::*;

    #[test]
    fn test_config_templates() {
        assert!(config::INVALID_PATH.contains("{path}"));
        assert!(config::PARSE_FAILED.contains("{file}"));
        assert!(config::PARSE_FAILED.contains("{error}"));
        assert!(config::VALIDATION_FAILED.contains("{field}"));
        assert!(config::VALIDATION_FAILED.contains("{reason}"));
        assert!(config::ENV_OVERRIDE_FAILED.contains("{var}"));
        assert!(config::ENV_OVERRIDE_FAILED.contains("{error}"));
    }

    #[test]
    fn test_database_templates() {
        assert!(database::CONNECTION_FAILED.contains("{url}"));
        assert!(database::CONNECTION_FAILED.contains("{error}"));
        assert!(database::QUERY_FAILED.contains("{operation}"));
        assert!(database::QUERY_FAILED.contains("{error}"));
        assert!(database::TRANSACTION_FAILED.contains("{operation}"));
        assert!(database::TRANSACTION_FAILED.contains("{error}"));
        assert!(database::CONSTRAINT_VIOLATION.contains("{table}"));
        assert!(database::CONSTRAINT_VIOLATION.contains("{constraint}"));
        assert!(database::TIMEOUT.contains("{operation}"));
        assert!(database::TIMEOUT.contains("{seconds}"));
    }

    #[test]
    fn test_policy_templates() {
        assert!(policy::PARSE_FAILED.contains("{rule}"));
        assert!(policy::PARSE_FAILED.contains("{error}"));
        assert!(policy::VALIDATION_FAILED.contains("{rule}"));
        assert!(policy::VALIDATION_FAILED.contains("{reason}"));
        assert!(policy::EXECUTION_FAILED.contains("{rule}"));
        assert!(policy::EXECUTION_FAILED.contains("{error}"));
        assert!(policy::CONDITION_FAILED.contains("{condition}"));
    }

    #[test]
    fn test_template_templates() {
        assert!(template::PARSE_FAILED.contains("{template}"));
        assert!(template::PARSE_FAILED.contains("{error}"));
        assert!(template::RENDER_FAILED.contains("{template}"));
        assert!(template::RENDER_FAILED.contains("{error}"));
        assert!(template::NOT_FOUND.contains("{template}"));
    }

    #[test]
    fn test_snmp_templates() {
        assert!(snmp::CONNECTION_FAILED.contains("{address}"));
        assert!(snmp::CONNECTION_FAILED.contains("{error}"));
        assert!(snmp::AUTH_FAILED.contains("{address}"));
        assert!(snmp::AUTH_FAILED.contains("{error}"));
        assert!(snmp::TIMEOUT.contains("{address}"));
        assert!(snmp::TIMEOUT.contains("{seconds}"));
        assert!(snmp::INVALID_OID.contains("{oid}"));
        assert!(snmp::INVALID_OID.contains("{reason}"));
    }

    #[test]
    fn test_network_templates() {
        assert!(network::INVALID_ADDRESS.contains("{address}"));
        assert!(network::INVALID_ADDRESS.contains("{reason}"));
        assert!(network::CONNECTION_REFUSED.contains("{address}"));
        assert!(network::CONNECTION_REFUSED.contains("{port}"));
        assert!(network::DNS_RESOLUTION_FAILED.contains("{hostname}"));
        assert!(network::DNS_RESOLUTION_FAILED.contains("{error}"));
    }

    #[test]
    fn test_validation_templates() {
        assert!(validation::REQUIRED_FIELD_MISSING.contains("{field}"));
        assert!(validation::INVALID_FORMAT.contains("{field}"));
        assert!(validation::INVALID_FORMAT.contains("{expected}"));
        assert!(validation::VALUE_OUT_OF_RANGE.contains("{field}"));
        assert!(validation::VALUE_OUT_OF_RANGE.contains("{value}"));
        assert!(validation::VALUE_OUT_OF_RANGE.contains("{min}"));
        assert!(validation::VALUE_OUT_OF_RANGE.contains("{max}"));
    }

    #[test]
    fn test_io_templates() {
        assert!(io::FILE_NOT_FOUND.contains("{path}"));
        assert!(io::PERMISSION_DENIED.contains("{path}"));
        assert!(io::DISK_FULL.contains("{path}"));
    }

    #[test]
    fn test_serialization_templates() {
        assert!(serialization::JSON_FAILED.contains("{type}"));
        assert!(serialization::JSON_FAILED.contains("{error}"));
        assert!(serialization::TOML_FAILED.contains("{type}"));
        assert!(serialization::TOML_FAILED.contains("{error}"));
        assert!(serialization::YAML_FAILED.contains("{type}"));
        assert!(serialization::YAML_FAILED.contains("{error}"));
    }
}

#[cfg(test)]
mod helpers_tests {
    use super::super::codes::helpers::*;
    use super::super::codes::templates;

    #[test]
    fn test_config_error_helper() {
        let error = config_error(
            templates::config::INVALID_PATH,
            &[("path", "/invalid/path")],
        );
        
        match error {
            super::super::Error::Config { message, .. } => {
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
            super::super::Error::Database { operation, message, .. } => {
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
            super::super::Error::Policy { rule, message, .. } => {
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
            super::super::Error::Network { endpoint, message, .. } => {
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
            super::super::Error::Validation { field, message, .. } => {
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
        
        let result = super::super::codes::helpers::format_template(template, args);
        assert_eq!(result, "Error in INSERT for users: constraint violation");
    }

    #[test]
    fn test_format_template_no_placeholders() {
        let template = "Simple error message";
        let args = &[];
        
        let result = super::super::codes::helpers::format_template(template, args);
        assert_eq!(result, "Simple error message");
    }

    #[test]
    fn test_format_template_unused_args() {
        let template = "Error: {error}";
        let args = &[("error", "timeout"), ("unused", "value")];
        
        let result = super::super::codes::helpers::format_template(template, args);
        assert_eq!(result, "Error: timeout");
    }

    #[test]
    fn test_format_template_missing_args() {
        let template = "Error in {operation}: {error}";
        let args = &[("operation", "SELECT")];
        
        let result = super::super::codes::helpers::format_template(template, args);
        assert_eq!(result, "Error in SELECT: {error}");
    }
}