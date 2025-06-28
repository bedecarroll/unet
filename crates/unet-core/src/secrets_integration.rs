//! Integration layer between configuration system and secure secrets storage
//!
//! This module provides helpers to automatically load sensitive configuration
//! values from secure storage and replace plain-text configuration with
//! encrypted secret references.

use crate::config::{AuthConfig, Config, SnmpConfig};
use crate::error::{Error, Result};
use crate::secrets::{SecretBackend, SecretManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration value that can be either plain text or a secret reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SecureValue {
    /// Plain text value (for development/testing only)
    Plain(String),
    /// Reference to a secret stored securely
    Secret {
        /// Name of the secret in the secret store
        secret_name: String,
        /// Optional fallback value if secret is not found
        fallback: Option<String>,
    },
}

impl SecureValue {
    /// Create a new secret reference
    pub fn secret(name: &str) -> Self {
        Self::Secret {
            secret_name: name.to_string(),
            fallback: None,
        }
    }

    /// Create a new secret reference with fallback
    pub fn secret_with_fallback(name: &str, fallback: &str) -> Self {
        Self::Secret {
            secret_name: name.to_string(),
            fallback: Some(fallback.to_string()),
        }
    }

    /// Create a plain text value (not recommended for production)
    pub fn plain(value: &str) -> Self {
        Self::Plain(value.to_string())
    }

    /// Resolve the actual value using the secret manager
    pub async fn resolve(&self, secret_manager: &mut SecretManager) -> Result<String> {
        match self {
            SecureValue::Plain(value) => Ok(value.clone()),
            SecureValue::Secret {
                secret_name,
                fallback,
            } => match secret_manager.get_secret(secret_name).await? {
                Some(value) => Ok(value),
                None => {
                    if let Some(fallback_value) = fallback {
                        Ok(fallback_value.clone())
                    } else {
                        Err(Error::config(format!(
                            "Secret '{}' not found and no fallback provided",
                            secret_name
                        )))
                    }
                }
            },
        }
    }
}

/// Secure SNMP configuration with secret references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureSnmpConfig {
    /// Community string (can be a secret reference)
    pub community: SecureValue,
    /// Default timeout in seconds
    pub timeout: u64,
    /// Default retry count
    pub retries: u32,
}

impl SecureSnmpConfig {
    /// Convert to regular SNMP config by resolving secrets
    pub async fn resolve(&self, secret_manager: &mut SecretManager) -> Result<SnmpConfig> {
        Ok(SnmpConfig {
            community: self.community.resolve(secret_manager).await?,
            timeout: self.timeout,
            retries: self.retries,
        })
    }
}

impl From<SnmpConfig> for SecureSnmpConfig {
    fn from(config: SnmpConfig) -> Self {
        Self {
            community: SecureValue::plain(&config.community),
            timeout: config.timeout,
            retries: config.retries,
        }
    }
}

/// Secure authentication configuration with secret references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureAuthConfig {
    /// Authentication enabled
    pub enabled: bool,
    /// JWT secret key (can be a secret reference)
    pub jwt_secret: SecureValue,
    /// Token validation endpoint (future)
    pub token_endpoint: Option<String>,
    /// Default token expiry in seconds
    pub token_expiry: u64,
}

impl SecureAuthConfig {
    /// Convert to regular auth config by resolving secrets
    pub async fn resolve(&self, secret_manager: &mut SecretManager) -> Result<AuthConfig> {
        Ok(AuthConfig {
            enabled: self.enabled,
            jwt_secret: self.jwt_secret.resolve(secret_manager).await?,
            token_endpoint: self.token_endpoint.clone(),
            token_expiry: self.token_expiry,
        })
    }
}

impl From<AuthConfig> for SecureAuthConfig {
    fn from(config: AuthConfig) -> Self {
        Self {
            enabled: config.enabled,
            jwt_secret: SecureValue::plain(&config.jwt_secret),
            token_endpoint: config.token_endpoint,
            token_expiry: config.token_expiry,
        }
    }
}

/// Configuration migration helper
pub struct ConfigMigrator {
    secret_manager: SecretManager,
}

impl ConfigMigrator {
    /// Create a new configuration migrator
    pub fn new(secret_manager: SecretManager) -> Self {
        Self { secret_manager }
    }

    /// Migrate sensitive configuration values to secure storage
    pub async fn migrate_config_to_secrets(&mut self, config: &Config) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "config-migration".to_string());
        metadata.insert("created_by".to_string(), "unet-core".to_string());

        // Migrate SNMP community string if not default
        if config.snmp.community != "public" {
            self.secret_manager
                .store_secret(
                    "snmp-community",
                    &config.snmp.community,
                    Some(metadata.clone()),
                )
                .await?;
        }

        // Migrate JWT secret if not default
        if config.auth.jwt_secret != "your-secret-key-change-in-production" {
            self.secret_manager
                .store_secret(
                    "jwt-secret",
                    &config.auth.jwt_secret,
                    Some(metadata.clone()),
                )
                .await?;
        }

        Ok(())
    }

    /// Bootstrap secrets for a new installation
    pub async fn bootstrap_secrets(&mut self) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "bootstrap".to_string());
        metadata.insert("created_by".to_string(), "unet-core".to_string());

        // Generate secure JWT secret
        let jwt_secret = generate_secure_secret(32);
        self.secret_manager
            .store_secret("jwt-secret", &jwt_secret, Some(metadata.clone()))
            .await?;

        // Store default SNMP community (can be changed later)
        self.secret_manager
            .store_secret("snmp-community", "public", Some(metadata.clone()))
            .await?;

        Ok(())
    }

    /// Get the internal secret manager
    pub fn secret_manager(&mut self) -> &mut SecretManager {
        &mut self.secret_manager
    }
}

/// Generate a cryptographically secure random secret
fn generate_secure_secret(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Production configuration loader that resolves secrets
pub struct SecureConfigLoader {
    secret_manager: SecretManager,
}

impl SecureConfigLoader {
    /// Create a new secure config loader
    pub async fn new(secret_backend: SecretBackend, master_password: &str) -> Result<Self> {
        let mut secret_manager = SecretManager::new(secret_backend);
        secret_manager
            .initialize_with_password(master_password)
            .await?;

        Ok(Self { secret_manager })
    }

    /// Load configuration and resolve all secret references
    pub async fn load_config(&mut self, config_path: Option<&str>) -> Result<Config> {
        // Load base configuration
        let mut config = if let Some(path) = config_path {
            Config::from_file(path)?
        } else {
            Config::from_env()?
        };

        // Resolve secrets for sensitive configuration
        if let Some(jwt_secret) = self.secret_manager.get_secret("jwt-secret").await? {
            config.auth.jwt_secret = jwt_secret;
        }

        if let Some(snmp_community) = self.secret_manager.get_secret("snmp-community").await? {
            config.snmp.community = snmp_community;
        }

        Ok(config)
    }

    /// Get the internal secret manager
    pub fn secret_manager(&mut self) -> &mut SecretManager {
        &mut self.secret_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::{MasterKey, SecretBackend};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_secure_value_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_path = temp_dir.path().join("secrets.json");

        let mut secret_manager = SecretManager::new(SecretBackend::File {
            path: secrets_path.to_string_lossy().to_string(),
        });

        let master_key = MasterKey::generate();
        secret_manager.initialize(master_key).await.unwrap();

        // Store a test secret
        secret_manager
            .store_secret("test-secret", "secret-value", None)
            .await
            .unwrap();

        // Test secret resolution
        let secret_ref = SecureValue::secret("test-secret");
        let resolved = secret_ref.resolve(&mut secret_manager).await.unwrap();
        assert_eq!(resolved, "secret-value");

        // Test plain value
        let plain_value = SecureValue::plain("plain-text");
        let resolved = plain_value.resolve(&mut secret_manager).await.unwrap();
        assert_eq!(resolved, "plain-text");

        // Test missing secret with fallback
        let missing_with_fallback = SecureValue::secret_with_fallback("missing", "fallback");
        let resolved = missing_with_fallback
            .resolve(&mut secret_manager)
            .await
            .unwrap();
        assert_eq!(resolved, "fallback");
    }

    #[tokio::test]
    async fn test_config_migration() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_path = temp_dir.path().join("secrets.json");

        let mut secret_manager = SecretManager::new(SecretBackend::File {
            path: secrets_path.to_string_lossy().to_string(),
        });

        let master_key = MasterKey::generate();
        secret_manager.initialize(master_key).await.unwrap();

        let mut migrator = ConfigMigrator::new(secret_manager);

        // Create config with sensitive values
        let mut config = Config::default();
        config.snmp.community = "private-community".to_string();
        config.auth.jwt_secret = "super-secret-jwt-key".to_string();

        // Migrate to secrets
        migrator.migrate_config_to_secrets(&config).await.unwrap();

        // Verify secrets were stored
        let snmp_community = migrator
            .secret_manager()
            .get_secret("snmp-community")
            .await
            .unwrap();
        assert_eq!(snmp_community, Some("private-community".to_string()));

        let jwt_secret = migrator
            .secret_manager()
            .get_secret("jwt-secret")
            .await
            .unwrap();
        assert_eq!(jwt_secret, Some("super-secret-jwt-key".to_string()));
    }

    #[test]
    fn test_generate_secure_secret() {
        let secret = generate_secure_secret(32);
        assert_eq!(secret.len(), 32);

        // Should be different each time
        let secret2 = generate_secure_secret(32);
        assert_ne!(secret, secret2);
    }
}
