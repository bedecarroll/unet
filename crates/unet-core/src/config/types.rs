//! Configuration type definitions

use serde::{Deserialize, Serialize};

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of database connections
    pub max_connections: Option<u32>,
    /// Database connection timeout in seconds
    pub timeout: Option<u64>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (debug, info, warn, error)
    pub level: String,
    /// Log format (json, text)
    pub format: String,
    /// Optional log file path
    pub file: Option<String>,
}

/// SNMP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpConfig {
    /// Default SNMP community string
    pub community: String,
    /// SNMP timeout in seconds
    pub timeout: u64,
    /// Number of retries for SNMP operations
    pub retries: u8,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum request size in bytes
    pub max_request_size: usize,
}

/// Git repository configuration for policy loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Repository URL (if using Git for policy storage)
    pub repository_url: Option<String>,
    /// Local checkout directory
    pub local_directory: Option<String>,
    /// Git branch to use
    pub branch: String,
    /// Authentication token for private repositories
    pub auth_token: Option<String>,
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Policies repository URL (backward compatibility)
    pub policies_repo: Option<String>,
    /// Templates repository URL (backward compatibility)
    pub templates_repo: Option<String>,
}

/// Domain configuration for network naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Default domain suffix
    pub default_domain: Option<String>,
    /// Search domains list
    pub search_domains: Vec<String>,
}

/// Authentication configuration (future extensibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication enabled
    pub enabled: bool,
    /// Token validation endpoint (future)
    pub token_endpoint: Option<String>,
    /// Default token expiry in seconds
    pub token_expiry: u64,
}
