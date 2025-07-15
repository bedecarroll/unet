//! Core configuration structure and loading/saving functionality

use crate::error::{Error, Result};
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::defaults;
use super::types::{
    AuthConfig, DatabaseConfig, DomainConfig, GitConfig, LoggingConfig, ServerConfig, SnmpConfig,
};

/// Main configuration structure for μNet
///
/// This structure contains all configuration sections for the application.
/// It supports loading from TOML files, environment variables, and provides
/// sensible defaults for all settings.
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
    /// Git configuration for policy loading
    pub git: GitConfig,
    /// Domain configuration for network naming
    pub domain: DomainConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
}

impl Config {
    /// Creates a new configuration with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// Loaded configuration or error if file cannot be read/parsed
    ///
    /// # Errors
    /// Returns an error if:
    /// - File cannot be read
    /// - TOML syntax is invalid
    /// - Configuration structure is invalid
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_str().ok_or_else(|| {
            Error::config(format!(
                "Configuration file path contains invalid UTF-8: {}",
                path.as_ref().display()
            ))
        })?;

        let config = ConfigBuilder::builder()
            .add_source(File::with_name(path_str))
            .build()
            .map_err(|e| {
                Error::config(format!(
                    "Failed to load configuration from '{path_str}': {e}"
                ))
            })?;

        config.try_deserialize().map_err(|e| {
            Error::config(format!(
                "Failed to parse configuration from '{path_str}': {e}"
            ))
        })
    }

    /// Loads configuration from environment variables
    ///
    /// Uses the `UNET_` prefix for environment variables.
    /// For example: `UNET_DATABASE__URL` maps to `database.url`
    ///
    /// # Returns
    /// Configuration loaded from environment variables with defaults
    ///
    /// # Errors
    /// Returns an error if environment variables contain invalid values
    pub fn from_env() -> Result<Self> {
        let builder = ConfigBuilder::builder()
            .add_source(
                Environment::with_prefix("UNET")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()
            .map_err(|e| Error::config(format!("Failed to read environment: {e}")))?;

        builder
            .try_deserialize()
            .map_err(|e| Error::config(format!("Failed to parse environment configuration: {e}")))
    }

    /// Saves the configuration to a TOML file
    ///
    /// # Arguments
    /// * `path` - Path where to save the configuration file
    ///
    /// # Returns
    /// Success or error if file cannot be written
    ///
    /// # Errors
    /// Returns an error if the file cannot be written or serialization fails
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize config: {e}")))?;

        std::fs::write(path.as_ref(), toml_content).map_err(|e| {
            Error::config(format!(
                "Failed to write config to {}: {e}",
                path.as_ref().display()
            ))
        })?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: defaults::database::DEFAULT_DATABASE_URL.to_string(),
                max_connections: Some(defaults::database::DEFAULT_DB_MAX_CONNECTIONS),
                timeout: Some(defaults::database::DEFAULT_DB_TIMEOUT_SECONDS),
            },
            logging: LoggingConfig {
                level: defaults::logging::DEFAULT_LOG_LEVEL.to_string(),
                format: defaults::logging::DEFAULT_LOG_FORMAT.to_string(),
                file: None,
            },
            snmp: SnmpConfig {
                community: defaults::snmp::DEFAULT_SNMP_COMMUNITY.to_string(),
                timeout: defaults::snmp::DEFAULT_SNMP_TIMEOUT_SECONDS,
                retries: defaults::snmp::DEFAULT_SNMP_RETRIES,
            },
            server: ServerConfig {
                host: defaults::server::DEFAULT_SERVER_HOST.to_string(),
                port: defaults::network::DEFAULT_SERVER_PORT,
                max_request_size: defaults::server::DEFAULT_MAX_REQUEST_SIZE,
            },
            git: GitConfig {
                repository_url: None,
                local_directory: Some("./policies".to_string()),
                branch: "main".to_string(),
                auth_token: None,
                sync_interval: 300, // 5 minutes
                policies_repo: None,
                templates_repo: None,
            },
            domain: DomainConfig {
                default_domain: None,
                search_domains: Vec::new(),
            },
            auth: AuthConfig {
                enabled: false,
                token_endpoint: None,
                token_expiry: 3600, // 1 hour
            },
        }
    }
}
