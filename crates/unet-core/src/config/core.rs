//! Core configuration structure and implementations

use crate::error::{Error, Result};
use config::{Config as ConfigBuilder, File};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;

use super::defaults;
use super::types::{
    AuthConfig, DatabaseConfig, DomainConfig, GitConfig, LoggingConfig, ServerConfig, SnmpConfig,
};

/// Main configuration structure for μNet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration settings
    pub database: DatabaseConfig,
    /// Logging configuration settings
    pub logging: LoggingConfig,
    /// SNMP configuration settings
    pub snmp: SnmpConfig,
    /// Server configuration settings
    pub server: ServerConfig,
    /// Git repository configuration settings
    pub git: GitConfig,
    /// Domain configuration settings
    pub domain: DomainConfig,
    /// Authentication configuration settings
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
    /// # Errors
    ///
    /// Returns an error if the file path contains invalid UTF-8, the file cannot be read,
    /// or the configuration cannot be parsed as valid TOML.
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
    /// # Errors
    ///
    /// Returns an error if environment variables cannot be parsed into valid configuration values
    /// or if the resulting configuration cannot be deserialized.
    pub fn from_env() -> Result<Self> {
        Self::from_env_with_source(|key| std::env::var(key))
    }

    /// Loads configuration from environment variables using a custom source function
    ///
    /// # Errors
    ///
    /// Returns an error if configuration overrides cannot be set, the configuration cannot be built,
    /// or the resulting configuration cannot be deserialized.
    pub fn from_env_with_source<F>(env_source: F) -> Result<Self>
    where
        F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
    {
        let mut builder = ConfigBuilder::builder();

        for (key, value) in collect_env_vars(&env_source) {
            builder = builder.set_override(&key, value).map_err(|e| {
                Error::config(format!("Failed to set config override for {key}: {e}"))
            })?;
        }

        builder
            .build()
            .map_err(|e| Error::config(format!("Failed to build config from environment: {e}")))?
            .try_deserialize()
            .map_err(|e| {
                Error::config(format!(
                    "Failed to deserialize config from environment: {e}"
                ))
            })
    }

    /// Saves configuration to a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be serialized to TOML
    /// or if the file cannot be written to the specified path.
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

    /// Validates the configuration
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration values are invalid, such as empty required fields,
    /// zero values where positive values are required, or invalid network addresses.
    pub fn validate(&self) -> Result<()> {
        self.validate_database()?;
        self.validate_server()?;
        self.validate_git()?;
        self.validate_auth()?;
        Ok(())
    }

    /// Returns the complete database URL
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Returns the server socket address
    ///
    /// # Errors
    ///
    /// Returns an error if the host and port combination cannot be parsed as a valid socket address.
    pub fn socket_addr(&self) -> Result<SocketAddr> {
        let addr = format!("{}:{}", self.server.host, self.server.port);
        addr.parse()
            .map_err(|e| Error::config(format!("Invalid server address '{addr}': {e}")))
    }

    fn validate_database(&self) -> Result<()> {
        if self.database.url.is_empty() {
            return Err(Error::config("Database URL cannot be empty"));
        }
        if let Some(max_conn) = self.database.max_connections {
            if max_conn == 0 {
                return Err(Error::config(
                    "Database max_connections must be greater than 0",
                ));
            }
        }
        if let Some(timeout) = self.database.timeout {
            if timeout == 0 {
                return Err(Error::config("Database timeout must be greater than 0"));
            }
        }
        Ok(())
    }

    fn validate_server(&self) -> Result<()> {
        if self.server.host.is_empty() {
            return Err(Error::config("Server host cannot be empty"));
        }
        if self.server.port == 0 {
            return Err(Error::config("Server port must be greater than 0"));
        }
        if self.server.max_request_size == 0 {
            return Err(Error::config(
                "Server max_request_size must be greater than 0",
            ));
        }
        self.socket_addr()?;
        Ok(())
    }

    fn validate_git(&self) -> Result<()> {
        if self.git.branch.is_empty() {
            return Err(Error::config("Git branch cannot be empty"));
        }
        if self.git.sync_interval == 0 {
            return Err(Error::config("Git sync_interval must be greater than 0"));
        }
        Ok(())
    }

    fn validate_auth(&self) -> Result<()> {
        if self.auth.enabled && self.auth.token_endpoint.is_none() {
            return Err(Error::config(
                "Auth token_endpoint must be set when auth is enabled",
            ));
        }
        if self.auth.token_expiry == 0 {
            return Err(Error::config("Auth token_expiry must be greater than 0"));
        }
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
                sync_interval: 300,
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
                token_expiry: 3_600,
            },
        }
    }
}

fn collect_env_vars<F>(env_source: &F) -> Vec<(String, String)>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    let env_vars = [
        ("UNET_DATABASE__URL", "database.url"),
        ("UNET_DATABASE__MAX_CONNECTIONS", "database.max_connections"),
        ("UNET_DATABASE__TIMEOUT", "database.timeout"),
        ("UNET_LOGGING__LEVEL", "logging.level"),
        ("UNET_LOGGING__FORMAT", "logging.format"),
        ("UNET_LOGGING__FILE", "logging.file"),
        ("UNET_SNMP__COMMUNITY", "snmp.community"),
        ("UNET_SNMP__TIMEOUT", "snmp.timeout"),
        ("UNET_SNMP__RETRIES", "snmp.retries"),
        ("UNET_SERVER__HOST", "server.host"),
        ("UNET_SERVER__PORT", "server.port"),
        ("UNET_SERVER__MAX_REQUEST_SIZE", "server.max_request_size"),
        ("UNET_GIT__REPOSITORY_URL", "git.repository_url"),
        ("UNET_GIT__LOCAL_DIRECTORY", "git.local_directory"),
        ("UNET_GIT__BRANCH", "git.branch"),
        ("UNET_GIT__AUTH_TOKEN", "git.auth_token"),
        ("UNET_GIT__SYNC_INTERVAL", "git.sync_interval"),
        ("UNET_DOMAIN__DEFAULT_DOMAIN", "domain.default_domain"),
        ("UNET_AUTH__ENABLED", "auth.enabled"),
        ("UNET_AUTH__TOKEN_ENDPOINT", "auth.token_endpoint"),
        ("UNET_AUTH__TOKEN_EXPIRY", "auth.token_expiry"),
    ];

    env_vars
        .iter()
        .filter_map(|(env_key, config_key)| {
            env_source(env_key)
                .ok()
                .map(|value| ((*config_key).to_string(), value))
        })
        .collect()
}
