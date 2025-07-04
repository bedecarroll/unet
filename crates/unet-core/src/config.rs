//! Configuration management for μNet Core
//!
//! This module provides TOML-based configuration management with hierarchical
//! configuration support and environment variable overrides.

pub mod migration;
pub mod validation;

use crate::cluster::ClusterConfig;
use crate::error::{Error, Result};
use crate::load_balancer::LoadBalancerConfig;
use crate::resource_management::ResourceConfig;
use crate::secrets::SecretsConfig;
use crate::shared_state::SharedStateConfig;
use crate::stateless::StatelessConfig;
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// μNet Core configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// SNMP configuration
    pub snmp: SnmpConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Git repository configuration
    pub git: GitConfig,
    /// Domain configuration
    pub domain: DomainConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Secrets management configuration
    pub secrets: SecretsConfig,
    /// Network access control configuration
    pub network: NetworkConfig,
    /// Metrics and monitoring configuration
    pub metrics: MetricsConfig,
    /// Load balancer compatibility configuration
    pub load_balancer: LoadBalancerConfig,
    /// Shared state management configuration
    pub shared_state: SharedStateConfig,
    /// Stateless operation configuration
    pub stateless: StatelessConfig,
    /// Cluster coordination configuration
    pub cluster: ClusterConfig,
    /// Resource management configuration
    pub resource_management: ResourceConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL (supports sqlite:// and postgresql://)
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    /// PostgreSQL-specific configuration
    pub postgres: Option<PostgresConfig>,
    /// Connection pool configuration
    pub pool: Option<PoolConfig>,
    /// Migration configuration
    pub migration: Option<MigrationConfig>,
}

/// PostgreSQL-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Enable connection SSL
    pub ssl: bool,
    /// SSL mode (disable, allow, prefer, require, verify-ca, verify-full)
    pub ssl_mode: Option<String>,
    /// SSL certificate file path
    pub ssl_cert: Option<String>,
    /// SSL key file path
    pub ssl_key: Option<String>,
    /// SSL root certificate file path
    pub ssl_root_cert: Option<String>,
    /// Application name for connection identification
    pub application_name: Option<String>,
    /// Schema search path
    pub search_path: Option<Vec<String>>,
    /// Statement timeout in seconds
    pub statement_timeout: Option<u64>,
    /// Lock timeout in seconds
    pub lock_timeout: Option<u64>,
    /// Idle connection timeout in seconds
    pub idle_timeout: Option<u64>,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_connections: Option<u32>,
    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,
    /// Time to wait for a connection before timing out (seconds)
    pub acquire_timeout: Option<u64>,
    /// Maximum lifetime of a connection (seconds)
    pub max_lifetime: Option<u64>,
    /// Idle timeout for connections (seconds)
    pub idle_timeout: Option<u64>,
    /// Test query to validate connections
    pub test_query: Option<String>,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Enable automatic migrations on startup
    pub auto_migrate: bool,
    /// Migration timeout in seconds
    pub timeout: Option<u64>,
    /// Lock timeout for migration operations (seconds)
    pub lock_timeout: Option<u64>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Log to file path (optional)
    pub file: Option<String>,
    /// OpenTelemetry tracing configuration
    pub opentelemetry: Option<OpenTelemetryConfig>,
    /// Log aggregation configuration
    pub aggregation: Option<LogAggregationConfig>,
    /// Log alerting configuration
    pub alerting: Option<LogAlertingConfig>,
}

/// OpenTelemetry configuration for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTelemetryConfig {
    /// Enable OpenTelemetry tracing
    pub enabled: bool,
    /// Jaeger collector endpoint
    pub jaeger_endpoint: Option<String>,
    /// Service name for tracing
    pub service_name: String,
    /// Service version for tracing
    pub service_version: String,
    /// Environment name (development, staging, production)
    pub environment: String,
    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f64,
}

/// Log aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAggregationConfig {
    /// Enable log aggregation
    pub enabled: bool,
    /// Syslog server endpoint (optional)
    pub syslog_endpoint: Option<String>,
    /// Log rotation configuration
    pub rotation: LogRotationConfig,
    /// Log parsing and indexing
    pub parsing: LogParsingConfig,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum file size in MB before rotation
    pub max_size_mb: u64,
    /// Maximum number of rotated files to keep
    pub max_files: u32,
    /// Rotation schedule (daily, hourly, size-based)
    pub schedule: String,
}

/// Log parsing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogParsingConfig {
    /// Enable structured log parsing
    pub enabled: bool,
    /// Extract and index specific fields
    pub indexed_fields: Vec<String>,
    /// Log enrichment with context data
    pub enrichment: bool,
}

/// Log alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertingConfig {
    /// Enable log-based alerting
    pub enabled: bool,
    /// Alert rules configuration
    pub rules: Vec<LogAlertRule>,
    /// Notification channels
    pub channels: Vec<AlertChannel>,
}

/// Log alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRule {
    /// Rule name
    pub name: String,
    /// Log level threshold (warn, error)
    pub level: String,
    /// Pattern to match in log messages
    pub pattern: Option<String>,
    /// Time window in minutes for evaluation
    pub window_minutes: u32,
    /// Threshold count within window
    pub threshold: u32,
    /// Alert severity (low, medium, high, critical)
    pub severity: String,
}

/// Alert notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannel {
    /// Channel type (email, slack, webhook, pagerduty)
    pub channel_type: String,
    /// Channel configuration (endpoint, token, etc.)
    pub config: std::collections::HashMap<String, String>,
}

/// SNMP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpConfig {
    /// Default community string
    pub community: String,
    /// Default timeout in seconds
    pub timeout: u64,
    /// Default retry count
    pub retries: u32,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
}

/// TLS configuration for HTTPS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// TLS certificate file path (PEM format)
    pub cert_file: String,
    /// TLS private key file path (PEM format)
    pub key_file: String,
    /// Force HTTPS redirects (redirect HTTP to HTTPS)
    pub force_https: bool,
    /// HTTP redirect port (for redirecting HTTP to HTTPS)
    pub http_redirect_port: Option<u16>,
}

/// Git repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Policies repository URL
    pub policies_repo: Option<String>,
    /// Templates repository URL
    pub templates_repo: Option<String>,
    /// Git branch to use
    pub branch: String,
    /// Sync interval in seconds
    pub sync_interval: u64,
}

/// Domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Default domain suffix
    pub default_domain: Option<String>,
    /// Search domains list
    pub search_domains: Vec<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication enabled
    pub enabled: bool,
    /// JWT secret key for token signing
    pub jwt_secret: String,
    /// Token validation endpoint (future)
    pub token_endpoint: Option<String>,
    /// Default token expiry in seconds
    pub token_expiry: u64,
}

/// Network access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable network access controls
    pub enabled: bool,
    /// Allowed IP addresses (whitelist)
    pub allowed_ips: Vec<String>,
    /// Blocked IP addresses (blacklist)
    pub blocked_ips: Vec<String>,
    /// Allowed IP ranges in CIDR notation
    pub allowed_ranges: Vec<String>,
    /// Blocked IP ranges in CIDR notation
    pub blocked_ranges: Vec<String>,
    /// Blocked country codes (ISO 3166-1 alpha-2)
    pub blocked_countries: Vec<String>,
    /// Allowed country codes (if specified, only these are allowed)
    pub allowed_countries: Option<Vec<String>>,
    /// Enable geolocation-based access control
    pub enable_geolocation: bool,
    /// Maximum request size for untrusted networks (bytes)
    pub untrusted_max_request_size: usize,
    /// Network interface binding restrictions
    pub bind_interfaces: Option<Vec<String>>,
    /// Enable network-based rate limiting
    pub enable_network_rate_limits: bool,
}

/// Metrics and monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable Prometheus metrics collection
    pub enabled: bool,
    /// Prometheus metrics endpoint path
    pub endpoint: String,
    /// Metrics server bind address (if different from main server)
    pub bind_address: Option<String>,
    /// Metrics collection interval in seconds
    pub collection_interval: u64,
    /// Enable detailed performance metrics
    pub enable_performance_metrics: bool,
    /// Enable business metrics (requests, users, nodes, etc.)
    pub enable_business_metrics: bool,
    /// Enable system health metrics (CPU, memory, disk)
    pub enable_system_metrics: bool,
    /// Custom metrics labels
    pub labels: std::collections::HashMap<String, String>,
    /// Metrics retention period in days
    pub retention_days: u32,
}

impl Config {
    /// Creates a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a TOML file with environment overrides
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let builder = ConfigBuilder::builder()
            .add_source(File::with_name(path.as_ref().to_str().unwrap_or("config")))
            .add_source(Environment::with_prefix("UNET").separator("_"));

        let config = builder
            .build()
            .map_err(|e| Error::config_with_source("Failed to build configuration", e))?;

        config
            .try_deserialize()
            .map_err(|e| Error::config_with_source("Failed to deserialize configuration", e))
    }

    /// Loads configuration from environment variables only
    pub fn from_env() -> Result<Self> {
        let builder =
            ConfigBuilder::builder().add_source(Environment::with_prefix("UNET").separator("_"));

        let config = builder.build().map_err(|e| {
            Error::config_with_source("Failed to build configuration from environment", e)
        })?;

        config
            .try_deserialize()
            .map_err(|e| Error::config_with_source("Failed to deserialize configuration", e))
    }

    /// Saves configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| Error::config_with_source("Failed to serialize configuration", e))?;

        std::fs::write(path, toml_string)
            .map_err(|e| Error::config_with_source("Failed to write configuration file", e))
    }

    /// Gets the database URL with environment variable override
    pub fn database_url(&self) -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| self.database.url.clone())
    }

    /// Validates the configuration for correctness
    pub fn validate(&self) -> Result<()> {
        // Validate database URL
        if self.database.url.is_empty() {
            return Err(Error::validation(
                "database.url",
                "Database URL cannot be empty",
            ));
        }

        // Validate database URL format
        if !Self::is_valid_database_url(&self.database.url) {
            return Err(Error::validation_with_value(
                "database.url",
                "Invalid database URL format. Must start with sqlite:// or postgresql://",
                &self.database.url,
            ));
        }

        // Validate PostgreSQL-specific configuration if URL is PostgreSQL
        if self.database.url.starts_with("postgresql://")
            || self.database.url.starts_with("postgres://")
        {
            if let Some(ref postgres_config) = self.database.postgres {
                Self::validate_postgres_config(postgres_config)?;
            }
        }

        // Validate pool configuration
        if let Some(ref pool) = self.database.pool {
            Self::validate_pool_config(pool)?;
        }

        // Validate migration configuration
        if let Some(ref migration) = self.database.migration {
            Self::validate_migration_config(migration)?;
        }

        // Validate server configuration
        if self.server.host.is_empty() {
            return Err(Error::validation(
                "server.host",
                "Server host cannot be empty",
            ));
        }
        if self.server.port == 0 {
            return Err(Error::validation(
                "server.port",
                "Server port must be greater than 0",
            ));
        }

        // Validate TLS configuration if present
        if let Some(ref tls) = self.server.tls {
            if tls.cert_file.is_empty() {
                return Err(Error::validation(
                    "server.tls.cert_file",
                    "TLS certificate file path cannot be empty",
                ));
            }
            if tls.key_file.is_empty() {
                return Err(Error::validation(
                    "server.tls.key_file",
                    "TLS private key file path cannot be empty",
                ));
            }
            // Validate certificate file exists
            if !std::path::Path::new(&tls.cert_file).exists() {
                return Err(Error::validation_with_value(
                    "server.tls.cert_file",
                    "TLS certificate file does not exist",
                    &tls.cert_file,
                ));
            }
            // Validate key file exists
            if !std::path::Path::new(&tls.key_file).exists() {
                return Err(Error::validation_with_value(
                    "server.tls.key_file",
                    "TLS private key file does not exist",
                    &tls.key_file,
                ));
            }
        }

        // Validate Git repositories if specified
        if let Some(ref repo) = self.git.policies_repo {
            if !repo.is_empty() && !Self::is_valid_git_url(repo) {
                return Err(Error::validation_with_value(
                    "git.policies_repo",
                    "Invalid Git repository URL format",
                    repo,
                ));
            }
        }
        if let Some(ref repo) = self.git.templates_repo {
            if !repo.is_empty() && !Self::is_valid_git_url(repo) {
                return Err(Error::validation_with_value(
                    "git.templates_repo",
                    "Invalid Git repository URL format",
                    repo,
                ));
            }
        }

        // Validate domain names
        if let Some(ref domain) = self.domain.default_domain {
            if !Self::is_valid_domain(domain) {
                return Err(Error::validation_with_value(
                    "domain.default_domain",
                    "Invalid domain name format",
                    domain,
                ));
            }
        }
        for domain in &self.domain.search_domains {
            if !Self::is_valid_domain(domain) {
                return Err(Error::validation_with_value(
                    "domain.search_domains",
                    "Invalid domain name format",
                    domain,
                ));
            }
        }

        // Validate logging configuration using utility functions
        crate::logging::validate_log_level(&self.logging.level)?;
        crate::logging::validate_log_format(&self.logging.format)?;

        Ok(())
    }

    /// Validates a Git URL
    fn is_valid_git_url(url: &str) -> bool {
        url.starts_with("https://") || url.starts_with("git@") || url.starts_with("ssh://")
    }

    /// Validates a domain name
    fn is_valid_domain(domain: &str) -> bool {
        !domain.is_empty()
            && domain.len() <= 253
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && domain
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    }

    /// Validates a database URL format
    fn is_valid_database_url(url: &str) -> bool {
        url.starts_with("sqlite://")
            || url.starts_with("postgresql://")
            || url.starts_with("postgres://")
    }

    /// Validates PostgreSQL-specific configuration
    fn validate_postgres_config(postgres: &PostgresConfig) -> Result<()> {
        // Validate SSL mode if provided
        if let Some(ref ssl_mode) = postgres.ssl_mode {
            let valid_modes = [
                "disable",
                "allow",
                "prefer",
                "require",
                "verify-ca",
                "verify-full",
            ];
            if !valid_modes.contains(&ssl_mode.as_str()) {
                return Err(Error::validation_with_value(
                    "database.postgres.ssl_mode",
                    "Invalid SSL mode. Must be one of: disable, allow, prefer, require, verify-ca, verify-full",
                    ssl_mode,
                ));
            }
        }

        // Validate SSL certificate files exist if provided
        if let Some(ref cert_file) = postgres.ssl_cert {
            if !cert_file.is_empty() && !std::path::Path::new(cert_file).exists() {
                return Err(Error::validation_with_value(
                    "database.postgres.ssl_cert",
                    "SSL certificate file does not exist",
                    cert_file,
                ));
            }
        }

        if let Some(ref key_file) = postgres.ssl_key {
            if !key_file.is_empty() && !std::path::Path::new(key_file).exists() {
                return Err(Error::validation_with_value(
                    "database.postgres.ssl_key",
                    "SSL key file does not exist",
                    key_file,
                ));
            }
        }

        if let Some(ref root_cert) = postgres.ssl_root_cert {
            if !root_cert.is_empty() && !std::path::Path::new(root_cert).exists() {
                return Err(Error::validation_with_value(
                    "database.postgres.ssl_root_cert",
                    "SSL root certificate file does not exist",
                    root_cert,
                ));
            }
        }

        // Validate timeout values
        if let Some(timeout) = postgres.statement_timeout {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.postgres.statement_timeout",
                    "Statement timeout must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = postgres.lock_timeout {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.postgres.lock_timeout",
                    "Lock timeout must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = postgres.idle_timeout {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.postgres.idle_timeout",
                    "Idle timeout must be greater than 0",
                ));
            }
        }

        Ok(())
    }

    /// Validates connection pool configuration
    fn validate_pool_config(pool: &PoolConfig) -> Result<()> {
        // Validate connection counts
        if let (Some(min), Some(max)) = (pool.min_connections, pool.max_connections) {
            if min > max {
                return Err(Error::validation(
                    "database.pool",
                    "Minimum connections cannot be greater than maximum connections",
                ));
            }
        }

        if let Some(min) = pool.min_connections {
            if min == 0 {
                return Err(Error::validation(
                    "database.pool.min_connections",
                    "Minimum connections must be greater than 0",
                ));
            }
        }

        if let Some(max) = pool.max_connections {
            if max == 0 {
                return Err(Error::validation(
                    "database.pool.max_connections",
                    "Maximum connections must be greater than 0",
                ));
            }
        }

        // Validate timeout values
        if let Some(timeout) = pool.acquire_timeout {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.pool.acquire_timeout",
                    "Acquire timeout must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = pool.max_lifetime {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.pool.max_lifetime",
                    "Maximum lifetime must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = pool.idle_timeout {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.pool.idle_timeout",
                    "Idle timeout must be greater than 0",
                ));
            }
        }

        Ok(())
    }

    /// Validates migration configuration
    fn validate_migration_config(migration: &MigrationConfig) -> Result<()> {
        if let Some(timeout) = migration.timeout {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.migration.timeout",
                    "Migration timeout must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = migration.lock_timeout {
            if timeout == 0 {
                return Err(Error::validation(
                    "database.migration.lock_timeout",
                    "Migration lock timeout must be greater than 0",
                ));
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "sqlite:///var/lib/unet/unet.db?mode=rwc".to_string(),
                max_connections: Some(10),
                timeout: Some(30),
                postgres: None,
                pool: Some(PoolConfig {
                    min_connections: Some(1),
                    max_connections: Some(10),
                    acquire_timeout: Some(30),
                    max_lifetime: Some(3600), // 1 hour
                    idle_timeout: Some(600),  // 10 minutes
                    test_query: None,
                }),
                migration: Some(MigrationConfig {
                    auto_migrate: true,
                    timeout: Some(300),     // 5 minutes
                    lock_timeout: Some(60), // 1 minute
                }),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                file: None,
                opentelemetry: Some(OpenTelemetryConfig {
                    enabled: false,
                    jaeger_endpoint: None,
                    service_name: "unet".to_string(),
                    service_version: "0.1.0".to_string(),
                    environment: "development".to_string(),
                    sample_rate: 0.1,
                }),
                aggregation: Some(LogAggregationConfig {
                    enabled: false,
                    syslog_endpoint: None,
                    rotation: LogRotationConfig {
                        max_size_mb: 100,
                        max_files: 10,
                        schedule: "daily".to_string(),
                    },
                    parsing: LogParsingConfig {
                        enabled: true,
                        indexed_fields: vec![
                            "timestamp".to_string(),
                            "level".to_string(),
                            "target".to_string(),
                            "message".to_string(),
                            "span_id".to_string(),
                            "trace_id".to_string(),
                        ],
                        enrichment: true,
                    },
                }),
                alerting: Some(LogAlertingConfig {
                    enabled: false,
                    rules: vec![
                        LogAlertRule {
                            name: "error_rate".to_string(),
                            level: "error".to_string(),
                            pattern: None,
                            window_minutes: 5,
                            threshold: 10,
                            severity: "high".to_string(),
                        },
                        LogAlertRule {
                            name: "critical_errors".to_string(),
                            level: "error".to_string(),
                            pattern: Some("CRITICAL".to_string()),
                            window_minutes: 1,
                            threshold: 1,
                            severity: "critical".to_string(),
                        },
                    ],
                    channels: vec![],
                }),
            },
            snmp: SnmpConfig {
                community: "public".to_string(),
                timeout: 5,
                retries: 3,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_request_size: 1_048_576, // 1MB
                tls: None,                   // TLS disabled by default
            },
            git: GitConfig {
                policies_repo: None,
                templates_repo: None,
                branch: "main".to_string(),
                sync_interval: 300, // 5 minutes
            },
            domain: DomainConfig {
                default_domain: None,
                search_domains: vec![],
            },
            auth: AuthConfig {
                enabled: false,
                jwt_secret: "your-secret-key-change-in-production".to_string(),
                token_endpoint: None,
                token_expiry: 3600, // 1 hour
            },
            secrets: SecretsConfig::default(),
            network: NetworkConfig {
                enabled: true,
                allowed_ips: vec!["127.0.0.1".to_string(), "::1".to_string()],
                blocked_ips: vec![],
                allowed_ranges: vec![
                    "192.168.0.0/16".to_string(),
                    "10.0.0.0/8".to_string(),
                    "172.16.0.0/12".to_string(),
                ],
                blocked_ranges: vec![],
                blocked_countries: vec![],
                allowed_countries: None,
                enable_geolocation: false,
                untrusted_max_request_size: 65536, // 64KB
                bind_interfaces: None,
                enable_network_rate_limits: true,
            },
            metrics: MetricsConfig {
                enabled: true,
                endpoint: "/metrics".to_string(),
                bind_address: None,      // Use same address as main server
                collection_interval: 15, // 15 seconds
                enable_performance_metrics: true,
                enable_business_metrics: true,
                enable_system_metrics: true,
                labels: {
                    let mut labels = std::collections::HashMap::new();
                    labels.insert("service".to_string(), "unet".to_string());
                    labels.insert("version".to_string(), "0.1.0".to_string());
                    labels
                },
                retention_days: 7,
            },
            load_balancer: LoadBalancerConfig::default(),
            shared_state: SharedStateConfig::default(),
            stateless: StatelessConfig::default(),
            cluster: ClusterConfig::default(),
            resource_management: ResourceConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(
            config.database.url,
            "sqlite:///var/lib/unet/unet.db?mode=rwc"
        );
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.snmp.community, "public");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_config_save_and_load() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();

        // Save configuration
        config.save_to_file(temp_file.path()).unwrap();

        // Load configuration with explicit file extension
        let path_str = temp_file.path().to_str().unwrap();
        let toml_path = format!("{}.toml", path_str);
        std::fs::copy(temp_file.path(), &toml_path).unwrap();

        let loaded_config = Config::from_file(&toml_path).unwrap();

        // Clean up
        std::fs::remove_file(&toml_path).ok();

        assert_eq!(config.database.url, loaded_config.database.url);
        assert_eq!(config.logging.level, loaded_config.logging.level);
        assert_eq!(config.snmp.community, loaded_config.snmp.community);
        assert_eq!(config.server.port, loaded_config.server.port);
    }

    #[test]
    fn test_database_url_override() {
        unsafe {
            std::env::set_var("DATABASE_URL", "postgresql://test");
        }
        let config = Config::default();
        assert_eq!(config.database_url(), "postgresql://test");
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }
}
