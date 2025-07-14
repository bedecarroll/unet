//! Configuration management for μNet Core
//!
//! This module provides TOML-based configuration management with hierarchical
//! configuration support and environment variable overrides.

use crate::error::{Error, Result};
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

// Re-export submodules
pub mod defaults;
pub mod network;
pub mod types;

// Re-export commonly used items
pub use defaults::*;
pub use network::*;
pub use types::*;

// Re-export specific constants for backward compatibility
pub use defaults::network::LOCALHOST_SNMP;

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

    /// Load configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// * `Ok(Config)` - Successfully loaded configuration
    /// * `Err(Error)` - If the file cannot be read or parsed
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
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

    /// Load configuration from environment variables
    ///
    /// Environment variables should be prefixed with `UNET_` and use double
    /// underscores to separate nested configuration keys.
    ///
    /// # Examples
    /// * `UNET_DATABASE__URL` -> `database.url`
    /// * `UNET_SERVER__PORT` -> `server.port`
    /// * `UNET_LOGGING__LEVEL` -> `logging.level`
    ///
    /// # Returns
    /// * `Ok(Config)` - Successfully loaded configuration
    /// * `Err(Error)` - If environment variables cannot be parsed
    ///
    /// # Errors
    /// Returns an error if environment variables cannot be parsed or loaded
    pub fn from_env() -> Result<Self> {
        let config = ConfigBuilder::builder()
            .add_source(Environment::with_prefix("UNET").separator("__"))
            .build()
            .map_err(|e| {
                Error::config(format!(
                    "Failed to load configuration from environment: {e}"
                ))
            })?;

        config.try_deserialize().map_err(|e| {
            Error::config(format!(
                "Failed to parse configuration from environment: {e}"
            ))
        })
    }

    /// Save configuration to a TOML file
    ///
    /// # Arguments
    /// * `path` - Path where to save the configuration file
    ///
    /// # Returns
    /// * `Ok(())` - Configuration saved successfully
    /// * `Err(Error)` - If the file cannot be written
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be serialized or the file cannot be written
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize configuration: {e}")))?;

        std::fs::write(&path, toml_content).map_err(|e| {
            Error::config(format!(
                "Failed to write configuration to '{}': {e}",
                path.as_ref().display()
            ))
        })
    }

    /// Validate configuration values and return warnings for invalid settings
    ///
    /// This method checks configuration values against reasonable bounds
    /// and adjusts them if necessary, returning a list of warnings.
    ///
    /// # Returns
    /// Vector of warning messages for adjusted configuration values
    pub fn validate_and_adjust(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Validate server configuration
        if self.server.max_request_size < defaults::server::MIN_REQUEST_SIZE {
            warnings.push(format!(
                "Server max_request_size {} is below minimum {}, adjusting to minimum",
                self.server.max_request_size,
                defaults::server::MIN_REQUEST_SIZE
            ));
            self.server.max_request_size = defaults::server::MIN_REQUEST_SIZE;
        }
        if self.server.max_request_size > defaults::server::MAX_REQUEST_SIZE {
            warnings.push(format!(
                "Server max_request_size {} exceeds maximum {}, adjusting to maximum",
                self.server.max_request_size,
                defaults::server::MAX_REQUEST_SIZE
            ));
            self.server.max_request_size = defaults::server::MAX_REQUEST_SIZE;
        }

        // Validate database configuration
        if let Some(max_conn) = self.database.max_connections {
            if max_conn < defaults::database::MIN_DB_CONNECTIONS {
                warnings.push(format!(
                    "Database max_connections {} is below minimum {}, adjusting to minimum",
                    max_conn,
                    defaults::database::MIN_DB_CONNECTIONS
                ));
                self.database.max_connections = Some(defaults::database::MIN_DB_CONNECTIONS);
            }
            if max_conn > defaults::database::MAX_DB_CONNECTIONS {
                warnings.push(format!(
                    "Database max_connections {} exceeds maximum {}, adjusting to maximum",
                    max_conn,
                    defaults::database::MAX_DB_CONNECTIONS
                ));
                self.database.max_connections = Some(defaults::database::MAX_DB_CONNECTIONS);
            }
        }

        // Validate SNMP configuration
        if self.snmp.timeout < defaults::snmp::MIN_SNMP_TIMEOUT_SECONDS {
            warnings.push(format!(
                "SNMP timeout {} is below minimum {}, adjusting to minimum",
                self.snmp.timeout,
                defaults::snmp::MIN_SNMP_TIMEOUT_SECONDS
            ));
            self.snmp.timeout = defaults::snmp::MIN_SNMP_TIMEOUT_SECONDS;
        }
        if self.snmp.timeout > defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS {
            warnings.push(format!(
                "SNMP timeout {} exceeds maximum {}, adjusting to maximum",
                self.snmp.timeout,
                defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS
            ));
            self.snmp.timeout = defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS;
        }
        if self.snmp.retries > defaults::snmp::MAX_SNMP_RETRIES {
            warnings.push(format!(
                "SNMP retries {} exceeds maximum {}, adjusting to maximum",
                self.snmp.retries,
                defaults::snmp::MAX_SNMP_RETRIES
            ));
            self.snmp.retries = defaults::snmp::MAX_SNMP_RETRIES;
        }

        warnings
    }

    /// Get the effective database URL, considering environment variable overrides
    ///
    /// This method first checks for the `DATABASE_URL` environment variable,
    /// falling back to the configured database URL if not found.
    ///
    /// # Returns
    /// The database URL to use for connections
    #[must_use]
    pub fn database_url(&self) -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| self.database.url.clone())
    }

    /// Check if the configuration represents a development environment
    ///
    /// This is determined by checking if the database URL uses `SQLite`
    /// and the server is bound to localhost.
    ///
    /// # Returns
    /// `true` if this appears to be a development configuration
    #[must_use]
    pub fn is_development(&self) -> bool {
        self.database_url().starts_with("sqlite:")
            && (self.server.host == "127.0.0.1" || self.server.host == "localhost")
    }

    /// Check if the configuration represents a production environment
    ///
    /// This is the inverse of `is_development()`.
    ///
    /// # Returns
    /// `true` if this appears to be a production configuration
    #[must_use]
    pub fn is_production(&self) -> bool {
        !self.is_development()
    }

    /// Validate configuration and return errors for invalid settings
    ///
    /// This method checks configuration values against reasonable bounds
    /// and returns validation errors for invalid settings.
    ///
    /// # Returns
    /// * `Ok(())` - Configuration is valid
    /// * `Err(Error)` - Configuration has validation errors
    ///
    /// # Errors
    /// Returns an error if any configuration values are outside acceptable bounds
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        // Validate server configuration
        if self.server.max_request_size < defaults::server::MIN_REQUEST_SIZE {
            errors.push(format!(
                "Server max_request_size {} is below minimum {}",
                self.server.max_request_size,
                defaults::server::MIN_REQUEST_SIZE
            ));
        }
        if self.server.max_request_size > defaults::server::MAX_REQUEST_SIZE {
            errors.push(format!(
                "Server max_request_size {} exceeds maximum {}",
                self.server.max_request_size,
                defaults::server::MAX_REQUEST_SIZE
            ));
        }

        // Validate database configuration
        if let Some(max_conn) = self.database.max_connections {
            if max_conn < defaults::database::MIN_DB_CONNECTIONS {
                errors.push(format!(
                    "Database max_connections {} is below minimum {}",
                    max_conn,
                    defaults::database::MIN_DB_CONNECTIONS
                ));
            }
            if max_conn > defaults::database::MAX_DB_CONNECTIONS {
                errors.push(format!(
                    "Database max_connections {} exceeds maximum {}",
                    max_conn,
                    defaults::database::MAX_DB_CONNECTIONS
                ));
            }
        }

        // Validate SNMP configuration
        if self.snmp.timeout < defaults::snmp::MIN_SNMP_TIMEOUT_SECONDS {
            errors.push(format!(
                "SNMP timeout {} is below minimum {}",
                self.snmp.timeout,
                defaults::snmp::MIN_SNMP_TIMEOUT_SECONDS
            ));
        }
        if self.snmp.timeout > defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS {
            errors.push(format!(
                "SNMP timeout {} exceeds maximum {}",
                self.snmp.timeout,
                defaults::snmp::MAX_SNMP_TIMEOUT_SECONDS
            ));
        }
        if self.snmp.retries > defaults::snmp::MAX_SNMP_RETRIES {
            errors.push(format!(
                "SNMP retries {} exceeds maximum {}",
                self.snmp.retries,
                defaults::snmp::MAX_SNMP_RETRIES
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::config(format!(
                "Configuration validation failed: {}",
                errors.join("; ")
            )))
        }
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
        let toml_path = format!("{path_str}.toml");
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
    fn test_config_database_url_override() {
        unsafe {
            std::env::set_var("DATABASE_URL", "postgresql://test");
        }
        let config = Config::default();
        assert_eq!(config.database_url(), "postgresql://test");
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn test_config_environment_detection() {
        let dev_config = Config::default();
        assert!(dev_config.is_development());
        assert!(!dev_config.is_production());
    }
}
