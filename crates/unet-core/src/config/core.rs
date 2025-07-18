//! Core configuration structure and loading/saving functionality

use crate::error::{Error, Result};
use config::{Config as ConfigBuilder, File};
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
        Self::from_env_with_source(|key| std::env::var(key))
    }

    /// Loads configuration from environment variables using a custom source function
    ///
    /// This function allows injecting a custom environment variable source for testing
    ///
    /// # Arguments
    /// * `env_source` - Function that takes a variable name and returns Result<String, `VarError`>
    ///
    /// # Returns
    /// Configuration loaded from environment variables with defaults
    ///
    /// # Errors
    /// Returns an error if environment variables contain invalid values
    fn from_env_with_source<F>(env_source: F) -> Result<Self>
    where
        F: for<'a> Fn(&'a str) -> std::result::Result<String, std::env::VarError>,
    {
        // Start with default config and merge in environment variables manually
        let mut config = Self::default();

        // Override with environment variables if they exist
        if let Ok(val) = env_source("UNET_DATABASE__URL") {
            config.database.url = val;
        }
        if let Ok(val) = env_source("UNET_SERVER__HOST") {
            config.server.host = val;
        }
        if let Ok(val) = env_source("UNET_SERVER__PORT") {
            config.server.port = val
                .parse()
                .map_err(|e| Error::config(format!("Invalid port number: {e}")))?;
        }
        if let Ok(val) = env_source("UNET_LOGGING__LEVEL") {
            config.logging.level = val;
        }

        Ok(config)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        let default_config = Config::default();

        assert_eq!(config.database.url, default_config.database.url);
        assert_eq!(config.server.host, default_config.server.host);
        assert_eq!(config.server.port, default_config.server.port);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();

        // Test database defaults
        assert_eq!(
            config.database.url,
            defaults::database::DEFAULT_DATABASE_URL
        );
        assert_eq!(
            config.database.max_connections,
            Some(defaults::database::DEFAULT_DB_MAX_CONNECTIONS)
        );
        assert_eq!(
            config.database.timeout,
            Some(defaults::database::DEFAULT_DB_TIMEOUT_SECONDS)
        );

        // Test server defaults
        assert_eq!(config.server.host, defaults::server::DEFAULT_SERVER_HOST);
        assert_eq!(config.server.port, defaults::network::DEFAULT_SERVER_PORT);
        assert_eq!(
            config.server.max_request_size,
            defaults::server::DEFAULT_MAX_REQUEST_SIZE
        );

        // Test logging defaults
        assert_eq!(config.logging.level, defaults::logging::DEFAULT_LOG_LEVEL);
        assert_eq!(config.logging.format, defaults::logging::DEFAULT_LOG_FORMAT);
        assert_eq!(config.logging.file, None);

        // Test SNMP defaults
        assert_eq!(
            config.snmp.community,
            defaults::snmp::DEFAULT_SNMP_COMMUNITY
        );
        assert_eq!(
            config.snmp.timeout,
            defaults::snmp::DEFAULT_SNMP_TIMEOUT_SECONDS
        );
        assert_eq!(config.snmp.retries, defaults::snmp::DEFAULT_SNMP_RETRIES);

        // Test git defaults
        assert_eq!(config.git.repository_url, None);
        assert_eq!(config.git.local_directory, Some("./policies".to_string()));
        assert_eq!(config.git.branch, "main");
        assert_eq!(config.git.auth_token, None);
        assert_eq!(config.git.sync_interval, 300);

        // Test domain defaults
        assert_eq!(config.domain.default_domain, None);
        assert!(config.domain.search_domains.is_empty());

        // Test auth defaults
        assert!(!config.auth.enabled);
        assert_eq!(config.auth.token_endpoint, None);
        assert_eq!(config.auth.token_expiry, 3600);
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("database"));
        assert!(debug_str.contains("server"));
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::with_suffix(".toml").expect("Failed to create temp file");
        let file_path = temp_file.path();

        // Create a custom config
        let mut original_config = Config::default();
        original_config.server.port = 9999;
        original_config.database.url = "sqlite:test.db".to_string();

        // Save config
        original_config
            .save_to_file(file_path)
            .expect("Failed to save config");

        // Load config
        let loaded_config = Config::from_file(file_path).expect("Failed to load config");

        assert_eq!(loaded_config.server.port, 9999);
        assert_eq!(loaded_config.database.url, "sqlite:test.db");
    }

    #[test]
    fn test_from_file_invalid_path() {
        let result = Config::from_file("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_file_invalid_toml() {
        let temp_file = NamedTempFile::with_suffix(".toml").expect("Failed to create temp file");

        // Write invalid TOML
        std::fs::write(temp_file.path(), "invalid toml content [[[")
            .expect("Failed to write invalid TOML");

        let result = Config::from_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_file_invalid_path() {
        let config = Config::default();
        let result = config.save_to_file("/invalid/path/that/cannot/exist/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_env_empty() {
        // Use a mock environment that has no UNET_ variables
        let empty_env = |_key: &str| -> std::result::Result<String, std::env::VarError> {
            Err(std::env::VarError::NotPresent)
        };

        let result = Config::from_env_with_source(empty_env);
        assert!(result.is_ok());

        // Should get default values when no env vars are set
        let config = result.unwrap();
        assert_eq!(
            config.database.url,
            defaults::database::DEFAULT_DATABASE_URL
        );
    }

    #[test]
    fn test_from_env_with_values() {
        // Use a mock environment with specific values
        let mock_env = |key: &str| -> std::result::Result<String, std::env::VarError> {
            match key {
                "UNET_SERVER__PORT" => Ok("9001".to_string()),
                "UNET_DATABASE__URL" => Ok("postgres://env-test/db".to_string()),
                "UNET_LOGGING__LEVEL" => Ok("debug".to_string()),
                _ => Err(std::env::VarError::NotPresent),
            }
        };

        let result = Config::from_env_with_source(mock_env);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server.port, 9001);
        assert_eq!(config.database.url, "postgres://env-test/db");
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_from_env_invalid_values() {
        // Use a mock environment with invalid port value
        let invalid_env = |key: &str| -> std::result::Result<String, std::env::VarError> {
            match key {
                "UNET_SERVER__PORT" => Ok("not_a_number".to_string()),
                _ => Err(std::env::VarError::NotPresent),
            }
        };

        let result = Config::from_env_with_source(invalid_env);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        // Test TOML serialization
        let toml_str = toml::to_string(&config).expect("Failed to serialize to TOML");
        assert!(toml_str.contains("[database]"));
        assert!(toml_str.contains("[server]"));
        assert!(toml_str.contains("[logging]"));

        // Test deserialization
        let deserialized: Config =
            toml::from_str(&toml_str).expect("Failed to deserialize from TOML");
        assert_eq!(config.database.url, deserialized.database.url);
        assert_eq!(config.server.port, deserialized.server.port);
    }

    #[test]
    fn test_config_structure_completeness() {
        let config = Config::default();

        // Ensure all major sections are present
        assert!(!config.database.url.is_empty());
        assert!(!config.server.host.is_empty());
        assert!(!config.logging.level.is_empty());
        assert!(!config.snmp.community.is_empty());
        assert!(!config.git.branch.is_empty());
        assert!(config.git.sync_interval > 0);
        assert!(config.auth.token_expiry > 0);
    }
}
