//! Centralized error codes and message templates for μNet Core

/// Error code constants for consistent error identification
pub mod constants {
    // Configuration errors (CONFIG_*)
    /// Error code for invalid configuration file paths
    pub const CONFIG_INVALID_PATH: &str = "CONFIG_INVALID_PATH";
    /// Error code for configuration file parsing failures
    pub const CONFIG_PARSE_FAILED: &str = "CONFIG_PARSE_FAILED";
    /// Error code for configuration validation failures
    pub const CONFIG_VALIDATION_FAILED: &str = "CONFIG_VALIDATION_FAILED";
    /// Error code for environment variable override failures
    pub const CONFIG_ENV_OVERRIDE_FAILED: &str = "CONFIG_ENV_OVERRIDE_FAILED";

    // Database errors (DB_*)
    /// Error code for database connection failures
    pub const DB_CONNECTION_FAILED: &str = "DB_CONNECTION_FAILED";
    /// Error code for database query failures
    pub const DB_QUERY_FAILED: &str = "DB_QUERY_FAILED";
    /// Error code for database transaction failures
    pub const DB_TRANSACTION_FAILED: &str = "DB_TRANSACTION_FAILED";
    /// Error code for database constraint violations
    pub const DB_CONSTRAINT_VIOLATION: &str = "DB_CONSTRAINT_VIOLATION";
    /// Error code for database operation timeouts
    pub const DB_TIMEOUT: &str = "DB_TIMEOUT";

    // Policy errors (POLICY_*)
    /// Error code for policy parsing failures
    pub const POLICY_PARSE_FAILED: &str = "POLICY_PARSE_FAILED";
    /// Error code for policy validation failures
    pub const POLICY_VALIDATION_FAILED: &str = "POLICY_VALIDATION_FAILED";
    /// Error code for policy execution failures
    pub const POLICY_EXECUTION_FAILED: &str = "POLICY_EXECUTION_FAILED";
    /// Error code for policy condition evaluation failures
    pub const POLICY_CONDITION_FAILED: &str = "POLICY_CONDITION_FAILED";

    // Template errors (TEMPLATE_*)
    /// Error code for template parsing failures
    pub const TEMPLATE_PARSE_FAILED: &str = "TEMPLATE_PARSE_FAILED";
    /// Error code for template rendering failures
    pub const TEMPLATE_RENDER_FAILED: &str = "TEMPLATE_RENDER_FAILED";
    /// Error code for template not found errors
    pub const TEMPLATE_NOT_FOUND: &str = "TEMPLATE_NOT_FOUND";

    // SNMP errors (SNMP_*)
    /// Error code for SNMP connection failures
    pub const SNMP_CONNECTION_FAILED: &str = "SNMP_CONNECTION_FAILED";
    /// Error code for SNMP authentication failures
    pub const SNMP_AUTH_FAILED: &str = "SNMP_AUTH_FAILED";
    /// Error code for SNMP request timeouts
    pub const SNMP_TIMEOUT: &str = "SNMP_TIMEOUT";
    /// Error code for invalid SNMP OIDs
    pub const SNMP_INVALID_OID: &str = "SNMP_INVALID_OID";

    // Network errors (NET_*)
    /// Error code for invalid network addresses
    pub const NET_INVALID_ADDRESS: &str = "NET_INVALID_ADDRESS";
    /// Error code for network connection refused errors
    pub const NET_CONNECTION_REFUSED: &str = "NET_CONNECTION_REFUSED";
    /// Error code for DNS resolution failures
    pub const NET_DNS_RESOLUTION_FAILED: &str = "NET_DNS_RESOLUTION_FAILED";

    // Validation errors (VALID_*)
    /// Error code for missing required fields
    pub const VALID_REQUIRED_FIELD_MISSING: &str = "VALID_REQUIRED_FIELD_MISSING";
    /// Error code for invalid field formats
    pub const VALID_INVALID_FORMAT: &str = "VALID_INVALID_FORMAT";
    /// Error code for values out of valid range
    pub const VALID_VALUE_OUT_OF_RANGE: &str = "VALID_VALUE_OUT_OF_RANGE";

    // I/O errors (IO_*)
    /// Error code for file not found errors
    pub const IO_FILE_NOT_FOUND: &str = "IO_FILE_NOT_FOUND";
    /// Error code for permission denied errors
    pub const IO_PERMISSION_DENIED: &str = "IO_PERMISSION_DENIED";
    /// Error code for disk full errors
    pub const IO_DISK_FULL: &str = "IO_DISK_FULL";

    // Serialization errors (SERIAL_*)
    /// Error code for JSON serialization failures
    pub const SERIAL_JSON_FAILED: &str = "SERIAL_JSON_FAILED";
    /// Error code for TOML serialization failures
    pub const SERIAL_TOML_FAILED: &str = "SERIAL_TOML_FAILED";
    /// Error code for YAML serialization failures
    pub const SERIAL_YAML_FAILED: &str = "SERIAL_YAML_FAILED";
}

/// Centralized error message templates for consistent messaging
pub mod templates {
    /// Configuration error message templates
    pub mod config {
        /// Template for invalid configuration file path errors
        pub const INVALID_PATH: &str = "Configuration file path contains invalid UTF-8: {path}";
        /// Template for configuration parsing failures
        pub const PARSE_FAILED: &str = "Failed to parse configuration file '{file}': {error}";
        /// Template for configuration validation failures
        pub const VALIDATION_FAILED: &str =
            "Configuration validation failed for field '{field}': {reason}";
        /// Template for environment variable override failures
        pub const ENV_OVERRIDE_FAILED: &str =
            "Failed to apply environment variable override '{var}': {error}";
    }

    /// Database error message templates
    pub mod database {
        /// Template for database connection failures
        pub const CONNECTION_FAILED: &str = "Failed to connect to database at '{url}': {error}";
        /// Template for database query failures
        pub const QUERY_FAILED: &str = "Database query failed for operation '{operation}': {error}";
        /// Template for database transaction failures
        pub const TRANSACTION_FAILED: &str =
            "Database transaction failed during '{operation}': {error}";
        /// Template for database constraint violations
        pub const CONSTRAINT_VIOLATION: &str =
            "Database constraint violation in '{table}': {constraint}";
        /// Template for database operation timeouts
        pub const TIMEOUT: &str =
            "Database operation '{operation}' timed out after {seconds} seconds";
    }

    /// Policy error message templates
    pub mod policy {
        /// Template for policy parsing failures
        pub const PARSE_FAILED: &str = "Failed to parse policy rule '{rule}': {error}";
        /// Template for policy validation failures
        pub const VALIDATION_FAILED: &str = "Policy validation failed for rule '{rule}': {reason}";
        /// Template for policy execution failures
        pub const EXECUTION_FAILED: &str = "Policy execution failed for rule '{rule}': {error}";
        /// Template for policy condition evaluation failures
        pub const CONDITION_FAILED: &str = "Policy condition evaluation failed: {condition}";
    }

    /// Template error message templates
    pub mod template {
        /// Template for template parsing failures
        pub const PARSE_FAILED: &str = "Failed to parse template '{template}': {error}";
        /// Template for template rendering failures
        pub const RENDER_FAILED: &str = "Failed to render template '{template}': {error}";
        /// Template for template not found errors
        pub const NOT_FOUND: &str = "Template '{template}' not found in search paths";
    }

    /// SNMP error message templates
    pub mod snmp {
        /// Template for SNMP connection failures
        pub const CONNECTION_FAILED: &str =
            "Failed to connect to SNMP agent at '{address}': {error}";
        /// Template for SNMP authentication failures
        pub const AUTH_FAILED: &str = "SNMP authentication failed for '{address}': {error}";
        /// Template for SNMP request timeouts
        pub const TIMEOUT: &str = "SNMP request to '{address}' timed out after {seconds} seconds";
        /// Template for invalid SNMP OID errors
        pub const INVALID_OID: &str = "Invalid SNMP OID '{oid}': {reason}";
    }

    /// Network error message templates
    pub mod network {
        /// Template for invalid network address errors
        pub const INVALID_ADDRESS: &str = "Invalid network address '{address}': {reason}";
        /// Template for network connection refused errors
        pub const CONNECTION_REFUSED: &str = "Connection refused to '{address}:{port}'";
        /// Template for DNS resolution failures
        pub const DNS_RESOLUTION_FAILED: &str = "DNS resolution failed for '{hostname}': {error}";
    }

    /// Validation error message templates
    pub mod validation {
        /// Template for missing required field errors
        pub const REQUIRED_FIELD_MISSING: &str = "Required field '{field}' is missing";
        /// Template for invalid field format errors
        pub const INVALID_FORMAT: &str = "Field '{field}' has invalid format: {expected}";
        /// Template for value out of range errors
        pub const VALUE_OUT_OF_RANGE: &str =
            "Field '{field}' value {value} is out of range [{min}, {max}]";
    }

    /// I/O error message templates
    pub mod io {
        /// Template for file not found errors
        pub const FILE_NOT_FOUND: &str = "File not found: '{path}'";
        /// Template for permission denied errors
        pub const PERMISSION_DENIED: &str = "Permission denied accessing '{path}'";
        /// Template for disk full errors
        pub const DISK_FULL: &str = "Insufficient disk space for operation on '{path}'";
    }

    /// Serialization error message templates
    pub mod serialization {
        /// Template for JSON serialization failures
        pub const JSON_FAILED: &str = "JSON serialization failed for type '{type}': {error}";
        /// Template for TOML serialization failures
        pub const TOML_FAILED: &str = "TOML serialization failed for type '{type}': {error}";
        /// Template for YAML serialization failures
        pub const YAML_FAILED: &str = "YAML serialization failed for type '{type}': {error}";
    }
}

/// Helper functions for creating standardized error messages
pub mod helpers {
    use super::super::Error;

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
    fn format_template(template: &str, args: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in args {
            let placeholder = format!("{{{key}}}");
            result = result.replace(&placeholder, value);
        }
        result
    }
}
