//! Configuration management for μNet Core
//!
//! This module provides TOML-based configuration management with hierarchical
//! configuration support and environment variable overrides.

use crate::error::{Error, Result};
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
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty)
    pub format: String,
    /// Log to file path (optional)
    pub file: Option<String>,
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "sqlite:./unet.db?mode=rwc".to_string(),
                max_connections: Some(10),
                timeout: Some(30),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                file: None,
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
                token_endpoint: None,
                token_expiry: 3600, // 1 hour
            },
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
        assert_eq!(config.database.url, "sqlite:./unet.db?mode=rwc");
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
