//! Error code constants for consistent error identification

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
