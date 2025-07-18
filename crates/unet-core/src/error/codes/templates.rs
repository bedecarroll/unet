//! Centralized error message templates for consistent messaging

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
    pub const TIMEOUT: &str = "Database operation '{operation}' timed out after {seconds} seconds";
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
    pub const CONNECTION_FAILED: &str = "Failed to connect to SNMP agent at '{address}': {error}";
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
