//! Configuration validation and adjustment logic

use super::core::Config;
use super::defaults;
use crate::error::{Error, Result};

impl Config {
    /// Validates and adjusts configuration values to be within acceptable ranges
    ///
    /// This method checks all configuration values against defined limits and
    /// automatically adjusts values that are outside acceptable ranges. It
    /// returns a list of warning messages for any adjustments made.
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

    /// Validates the configuration for consistency and required values
    ///
    /// This method performs additional validation checks that cannot be
    /// automatically adjusted, such as checking for required fields and
    /// logical consistency between configuration sections.
    ///
    /// # Returns
    /// Success or error if validation fails
    ///
    /// # Errors
    /// Returns an error if:
    /// - Required configuration values are missing
    /// - Configuration values are logically inconsistent
    /// - Values cannot be used as specified
    pub fn validate(&self) -> Result<()> {
        // Validate database URL is not empty
        if self.database.url.trim().is_empty() {
            return Err(Error::config("Database URL cannot be empty"));
        }

        // Validate server configuration
        if self.server.host.trim().is_empty() {
            return Err(Error::config("Server host cannot be empty"));
        }

        if self.server.port == 0 {
            return Err(Error::config("Server port must be greater than 0"));
        }

        // Validate Git configuration if repository URL is provided
        if let Some(ref repo_url) = self.git.repository_url {
            if !repo_url.trim().is_empty() {
                // Basic URL format validation
                if !repo_url.starts_with("http://")
                    && !repo_url.starts_with("https://")
                    && !repo_url.starts_with("git://")
                    && !repo_url.starts_with("ssh://")
                    && !repo_url.starts_with("git@")
                {
                    return Err(Error::config(format!(
                        "Invalid Git repository URL format: {repo_url}"
                    )));
                }
            }
        }

        // Validate domain configuration
        if let Some(ref default_domain) = self.domain.default_domain {
            if !default_domain.trim().is_empty() {
                // Basic domain format validation
                if default_domain.contains(' ') || default_domain.starts_with('.') {
                    return Err(Error::config(format!(
                        "Invalid domain format: {default_domain}"
                    )));
                }
            }
        }

        Ok(())
    }
}
