// Secure credential storage and management for μNet Core
//
// This module provides encrypted storage and retrieval of sensitive configuration
// data such as passwords, API keys, and other secrets using industry-standard
// encryption algorithms.

use crate::error::{Error, Result};
use crate::secrets::external::{
    ExternalProviderConfig, ExternalSecretProvider, create_external_provider,
};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use argon2::Argon2;
use base64::{Engine as _, engine::general_purpose};
use keyring::Entry;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Maximum size for secrets (1MB)
const MAX_SECRET_SIZE: usize = 1024 * 1024;

/// Service name for system keyring
const KEYRING_SERVICE: &str = "unet-secrets";

/// Default master key name in keyring
const MASTER_KEY_NAME: &str = "master-key";

/// Encrypted secret data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSecret {
    /// Base64-encoded encrypted data
    pub data: String,
    /// Base64-encoded nonce/IV
    pub nonce: String,
    /// Encryption algorithm used
    pub algorithm: String,
    /// Timestamp when encrypted
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Optional expiration time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Secret metadata (tags, description, etc.)
    pub metadata: HashMap<String, String>,
}

/// Master key for encrypting secrets
#[derive(ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; 32],
}

impl MasterKey {
    /// Generate a new random master key
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    /// Create master key from password using Argon2
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self> {
        use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};

        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| Error::config(format!("Invalid salt: {}", e)))?;

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| Error::config(format!("Failed to hash password: {}", e)))?;

        let mut key = [0u8; 32];
        if let Some(hash) = password_hash.hash {
            let hash_bytes = hash.as_bytes();
            let copy_len = hash_bytes.len().min(32);
            key[..copy_len].copy_from_slice(&hash_bytes[..copy_len]);
        } else {
            return Err(Error::config("Password hashing did not produce a hash"));
        }
        Ok(Self { key })
    }

    /// Create master key from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(Error::config("Master key must be exactly 32 bytes"));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    /// Get key bytes (zero-copied on drop)
    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }
}

/// Secret storage backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretBackend {
    /// File-based storage (encrypted)
    File { path: String },
    /// System keyring storage
    Keyring,
    /// Environment variables (for development only)
    Environment,
    /// External secret stores
    External {
        provider: String,
        config: HashMap<String, String>,
    },
}

/// Secure secret manager
pub struct SecretManager {
    backend: SecretBackend,
    master_key: Option<MasterKey>,
    secrets_cache: HashMap<String, EncryptedSecret>,
    external_provider: Option<Box<dyn ExternalSecretProvider>>,
}

impl SecretManager {
    /// Create a new secret manager with specified backend
    pub fn new(backend: SecretBackend) -> Self {
        Self {
            backend,
            master_key: None,
            secrets_cache: HashMap::new(),
            external_provider: None,
        }
    }

    /// Create a new in-memory secret manager for testing
    pub fn new_in_memory() -> Self {
        Self::new(SecretBackend::Environment)
    }

    /// Initialize the secret manager with a master key
    pub async fn initialize(&mut self, master_key: MasterKey) -> Result<()> {
        self.master_key = Some(master_key);

        // Initialize external provider if using external backend
        if let SecretBackend::External { provider, config } = &self.backend {
            let provider_config = ExternalProviderConfig {
                provider_type: match provider.as_str() {
                    "vault" => crate::secrets::external::ExternalProviderType::Vault,
                    "aws-secrets-manager" => {
                        crate::secrets::external::ExternalProviderType::AwsSecretsManager
                    }
                    "azure-key-vault" => {
                        crate::secrets::external::ExternalProviderType::AzureKeyVault
                    }
                    _ => {
                        return Err(Error::config(format!(
                            "Unsupported external provider: {}",
                            provider
                        )));
                    }
                },
                config: config.clone(),
            };

            self.external_provider = Some(create_external_provider(&provider_config).await?);
        }

        self.load_secrets().await?;
        Ok(())
    }

    /// Initialize with password-based master key
    pub async fn initialize_with_password(&mut self, password: &str) -> Result<()> {
        let salt = self.get_or_create_salt().await?;
        let master_key = MasterKey::from_password(password, &salt)?;
        self.initialize(master_key).await
    }

    /// Store a secret securely
    pub async fn store_secret(
        &mut self,
        name: &str,
        value: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        if value.len() > MAX_SECRET_SIZE {
            return Err(Error::config("Secret too large"));
        }

        // Handle external providers
        if let Some(ref mut provider) = self.external_provider {
            return provider.store_secret(name, value, metadata).await;
        }

        let master_key = self
            .master_key
            .as_ref()
            .ok_or_else(|| Error::config("SecretManager not initialized"))?;

        let encrypted = self.encrypt_secret(value, master_key, metadata)?;
        self.secrets_cache
            .insert(name.to_string(), encrypted.clone());
        self.persist_secret(name, &encrypted).await?;

        Ok(())
    }

    /// Retrieve a secret
    pub async fn get_secret(&mut self, name: &str) -> Result<Option<String>> {
        // Handle external providers
        if let Some(ref mut provider) = self.external_provider {
            return provider.get_secret(name).await;
        }

        let master_key = self
            .master_key
            .as_ref()
            .ok_or_else(|| Error::config("SecretManager not initialized"))?;

        // Check cache first
        if let Some(encrypted) = self.secrets_cache.get(name) {
            // Check expiration
            if let Some(expires_at) = encrypted.expires_at {
                if chrono::Utc::now() > expires_at {
                    self.delete_secret(name).await?;
                    return Ok(None);
                }
            }
            return Ok(Some(self.decrypt_secret(encrypted, master_key)?));
        }

        // Load from backend
        if let Some(encrypted) = self.load_secret(name).await? {
            // Check expiration
            if let Some(expires_at) = encrypted.expires_at {
                if chrono::Utc::now() > expires_at {
                    self.delete_secret(name).await?;
                    return Ok(None);
                }
            }
            let decrypted = self.decrypt_secret(&encrypted, master_key)?;
            self.secrets_cache.insert(name.to_string(), encrypted);
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }

    /// List all secret names
    pub async fn list_secrets(&self) -> Result<Vec<String>> {
        // Handle external providers
        if let Some(ref provider) = self.external_provider {
            return provider.list_secrets().await;
        }

        match &self.backend {
            SecretBackend::File { path } => {
                let secrets_path = Path::new(path);
                if !secrets_path.exists() {
                    return Ok(vec![]);
                }

                let contents = fs::read_to_string(secrets_path)
                    .await
                    .map_err(|e| Error::Io {
                        path: secrets_path.to_string_lossy().to_string(),
                        message: "Failed to read secrets file".to_string(),
                        source: e,
                    })?;

                let secrets: HashMap<String, EncryptedSecret> = serde_json::from_str(&contents)
                    .map_err(|e| Error::config_with_source("Failed to parse secrets file", e))?;

                Ok(secrets.keys().cloned().collect())
            }
            SecretBackend::Keyring => {
                // For keyring, we maintain a separate index
                self.list_keyring_secrets().await
            }
            SecretBackend::Environment => {
                // List environment variables with UNET_SECRET_ prefix
                Ok(std::env::vars()
                    .filter_map(|(key, _)| {
                        if key.starts_with("UNET_SECRET_") {
                            Some(key.strip_prefix("UNET_SECRET_").unwrap().to_lowercase())
                        } else {
                            None
                        }
                    })
                    .collect())
            }
            SecretBackend::External { .. } => {
                // External providers handled above
                Ok(vec![])
            }
        }
    }

    /// Get secret metadata (including timestamps and rotation information)
    pub async fn get_secret_metadata(
        &mut self,
        name: &str,
    ) -> Result<Option<HashMap<String, String>>> {
        // Handle external providers
        if let Some(ref mut provider) = self.external_provider {
            // External providers may have metadata support
            // For now, return basic metadata
            let exists = provider.get_secret(name).await?.is_some();
            if exists {
                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), "external".to_string());
                return Ok(Some(metadata));
            } else {
                return Ok(None);
            }
        }

        // Try cache first
        if let Some(encrypted) = self.secrets_cache.get(name) {
            return Ok(Some(encrypted.metadata.clone()));
        }

        // Load from backend
        if let Some(encrypted) = self.load_secret(name).await? {
            return Ok(Some(encrypted.metadata));
        }

        Ok(None)
    }

    /// Delete a secret
    pub async fn delete_secret(&mut self, name: &str) -> Result<bool> {
        // Handle external providers
        if let Some(ref mut provider) = self.external_provider {
            return provider.delete_secret(name).await;
        }

        self.secrets_cache.remove(name);

        match &self.backend {
            SecretBackend::File { path } => {
                let secrets_path = Path::new(path);
                if !secrets_path.exists() {
                    return Ok(false);
                }

                let contents = fs::read_to_string(secrets_path)
                    .await
                    .map_err(|e| Error::Io {
                        path: secrets_path.to_string_lossy().to_string(),
                        message: "Failed to read secrets file".to_string(),
                        source: e,
                    })?;

                let mut secrets: HashMap<String, EncryptedSecret> = serde_json::from_str(&contents)
                    .map_err(|e| Error::config_with_source("Failed to parse secrets file", e))?;

                let existed = secrets.remove(name).is_some();

                let updated_contents = serde_json::to_string_pretty(&secrets)
                    .map_err(|e| Error::config_with_source("Failed to serialize secrets", e))?;

                fs::write(secrets_path, updated_contents)
                    .await
                    .map_err(|e| Error::Io {
                        path: secrets_path.to_string_lossy().to_string(),
                        message: "Failed to write secrets file".to_string(),
                        source: e,
                    })?;

                Ok(existed)
            }
            SecretBackend::Keyring => match Entry::new(KEYRING_SERVICE, name) {
                Ok(entry) => match entry.delete_password() {
                    Ok(()) => Ok(true),
                    Err(keyring::Error::NoEntry) => Ok(false),
                    Err(e) => Err(Error::config_with_source(
                        "Failed to delete from keyring",
                        e,
                    )),
                },
                Err(e) => Err(Error::config_with_source(
                    "Failed to create keyring entry",
                    e,
                )),
            },
            SecretBackend::Environment => {
                // Can't delete environment variables
                Ok(false)
            }
            SecretBackend::External { .. } => {
                // External providers handled above
                Ok(false)
            }
        }
    }

    /// Rotate encryption for all secrets (when master key changes)
    pub async fn rotate_secrets(&mut self, new_master_key: MasterKey) -> Result<()> {
        let old_master_key = self
            .master_key
            .as_ref()
            .ok_or_else(|| Error::config("SecretManager not initialized"))?;

        let secret_names = self.list_secrets().await?;
        let mut rotated_secrets = HashMap::new();

        // Decrypt with old key and re-encrypt with new key
        for name in secret_names {
            if let Some(encrypted) = self.load_secret(&name).await? {
                let decrypted = self.decrypt_secret(&encrypted, old_master_key)?;
                let re_encrypted =
                    self.encrypt_secret(&decrypted, &new_master_key, Some(encrypted.metadata))?;
                rotated_secrets.insert(name, re_encrypted);
            }
        }

        // Update master key
        self.master_key = Some(new_master_key);

        // Persist all rotated secrets
        for (name, encrypted) in rotated_secrets {
            self.secrets_cache.insert(name.clone(), encrypted.clone());
            self.persist_secret(&name, &encrypted).await?;
        }

        Ok(())
    }

    // Private helper methods

    fn encrypt_secret(
        &self,
        value: &str,
        master_key: &MasterKey,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<EncryptedSecret> {
        let cipher = Aes256Gcm::new_from_slice(master_key.as_bytes())
            .map_err(|e| Error::config(format!("Failed to create cipher: {}", e)))?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, value.as_bytes())
            .map_err(|e| Error::config(format!("Failed to encrypt secret: {}", e)))?;

        Ok(EncryptedSecret {
            data: general_purpose::STANDARD.encode(ciphertext),
            nonce: general_purpose::STANDARD.encode(nonce),
            algorithm: "AES256-GCM".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            metadata: metadata.unwrap_or_default(),
        })
    }

    fn decrypt_secret(
        &self,
        encrypted: &EncryptedSecret,
        master_key: &MasterKey,
    ) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(master_key.as_bytes())
            .map_err(|e| Error::config(format!("Failed to create cipher: {}", e)))?;

        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted.data)
            .map_err(|e| Error::config(format!("Failed to decode encrypted data: {}", e)))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted.nonce)
            .map_err(|e| Error::config(format!("Failed to decode nonce: {}", e)))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| Error::config(format!("Failed to decrypt secret: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| Error::config(format!("Invalid UTF-8 in decrypted secret: {}", e)))
    }

    async fn get_or_create_salt(&self) -> Result<Vec<u8>> {
        match &self.backend {
            SecretBackend::Keyring => {
                match Entry::new(KEYRING_SERVICE, "salt") {
                    Ok(entry) => {
                        match entry.get_password() {
                            Ok(salt_b64) => {
                                general_purpose::STANDARD.decode(salt_b64).map_err(|e| {
                                    Error::config(format!("Invalid salt in keyring: {}", e))
                                })
                            }
                            Err(keyring::Error::NoEntry) => {
                                // Generate new salt
                                let mut salt = [0u8; 16];
                                OsRng.fill_bytes(&mut salt);
                                let salt_b64 = general_purpose::STANDARD.encode(salt);
                                entry.set_password(&salt_b64).map_err(|e| {
                                    Error::config_with_source("Failed to store salt in keyring", e)
                                })?;
                                Ok(salt.to_vec())
                            }
                            Err(e) => Err(Error::config_with_source(
                                "Failed to read salt from keyring",
                                e,
                            )),
                        }
                    }
                    Err(e) => Err(Error::config_with_source(
                        "Failed to create keyring entry for salt",
                        e,
                    )),
                }
            }
            _ => {
                // For file backend, use a fixed salt (less secure)
                Ok(b"unet-salt-v1.0.0".to_vec())
            }
        }
    }

    async fn load_secrets(&mut self) -> Result<()> {
        match &self.backend {
            SecretBackend::File { path } => {
                let secrets_path = Path::new(path);
                if secrets_path.exists() {
                    let contents =
                        fs::read_to_string(secrets_path)
                            .await
                            .map_err(|e| Error::Io {
                                path: secrets_path.to_string_lossy().to_string(),
                                message: "Failed to read secrets file".to_string(),
                                source: e,
                            })?;

                    self.secrets_cache = serde_json::from_str(&contents).map_err(|e| {
                        Error::config_with_source("Failed to parse secrets file", e)
                    })?;
                }
            }
            _ => {
                // Other backends load on-demand
            }
        }
        Ok(())
    }

    async fn load_secret(&self, name: &str) -> Result<Option<EncryptedSecret>> {
        match &self.backend {
            SecretBackend::File { .. } => {
                // Already loaded in cache
                Ok(self.secrets_cache.get(name).cloned())
            }
            SecretBackend::Keyring => match Entry::new(KEYRING_SERVICE, name) {
                Ok(entry) => match entry.get_password() {
                    Ok(encrypted_json) => {
                        let encrypted: EncryptedSecret = serde_json::from_str(&encrypted_json)
                            .map_err(|e| {
                                Error::config_with_source("Failed to parse secret from keyring", e)
                            })?;
                        Ok(Some(encrypted))
                    }
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(e) => Err(Error::config_with_source("Failed to read from keyring", e)),
                },
                Err(e) => Err(Error::config_with_source(
                    "Failed to create keyring entry",
                    e,
                )),
            },
            SecretBackend::Environment => {
                let env_key = format!("UNET_SECRET_{}", name.to_uppercase());
                Ok(std::env::var(&env_key).ok().map(|value| EncryptedSecret {
                    data: general_purpose::STANDARD.encode(value),
                    nonce: String::new(),
                    algorithm: "plaintext".to_string(),
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    metadata: HashMap::new(),
                }))
            }
            SecretBackend::External { .. } => {
                // External providers handled directly in get_secret
                Ok(None)
            }
        }
    }

    async fn persist_secret(&mut self, name: &str, encrypted: &EncryptedSecret) -> Result<()> {
        match &self.backend {
            SecretBackend::File { path } => {
                let secrets_path = Path::new(path);

                // Ensure directory exists
                if let Some(dir) = secrets_path.parent() {
                    fs::create_dir_all(dir).await.map_err(|e| Error::Io {
                        path: dir.to_string_lossy().to_string(),
                        message: "Failed to create secrets directory".to_string(),
                        source: e,
                    })?;
                }

                let contents = serde_json::to_string_pretty(&self.secrets_cache)
                    .map_err(|e| Error::config_with_source("Failed to serialize secrets", e))?;

                fs::write(secrets_path, contents)
                    .await
                    .map_err(|e| Error::Io {
                        path: secrets_path.to_string_lossy().to_string(),
                        message: "Failed to write secrets file".to_string(),
                        source: e,
                    })?;
            }
            SecretBackend::Keyring => {
                let encrypted_json = serde_json::to_string(encrypted)
                    .map_err(|e| Error::config_with_source("Failed to serialize secret", e))?;

                match Entry::new(KEYRING_SERVICE, name) {
                    Ok(entry) => {
                        entry.set_password(&encrypted_json).map_err(|e| {
                            Error::config_with_source("Failed to store in keyring", e)
                        })?;
                    }
                    Err(e) => {
                        return Err(Error::config_with_source(
                            "Failed to create keyring entry",
                            e,
                        ));
                    }
                }
            }
            SecretBackend::Environment => {
                // Cannot persist to environment variables
                return Err(Error::config(
                    "Cannot persist secrets to environment backend",
                ));
            }
            SecretBackend::External { .. } => {
                // External providers handled directly in store_secret
                return Err(Error::config("External secret providers handled directly"));
            }
        }
        Ok(())
    }

    async fn list_keyring_secrets(&self) -> Result<Vec<String>> {
        // Keyring doesn't have enumeration, so we maintain an index
        match Entry::new(KEYRING_SERVICE, "secret-index") {
            Ok(entry) => match entry.get_password() {
                Ok(index_json) => {
                    let index: Vec<String> = serde_json::from_str(&index_json).map_err(|e| {
                        Error::config_with_source("Failed to parse secret index", e)
                    })?;
                    Ok(index)
                }
                Err(keyring::Error::NoEntry) => Ok(vec![]),
                Err(e) => Err(Error::config_with_source(
                    "Failed to read secret index from keyring",
                    e,
                )),
            },
            Err(e) => Err(Error::config_with_source(
                "Failed to create keyring entry for index",
                e,
            )),
        }
    }
}

/// Configuration for secret management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Secret storage backend
    pub backend: SecretBackend,
    /// Auto-initialize with empty password (for development only)
    pub auto_init: bool,
    /// Master password source (env var name or literal for dev)
    pub master_password_source: Option<String>,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            backend: SecretBackend::File {
                path: "secrets.json".to_string(),
            },
            auto_init: false,
            master_password_source: Some("UNET_MASTER_PASSWORD".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_secret_encryption_decryption() {
        let master_key = MasterKey::generate();
        let mut manager = SecretManager::new(SecretBackend::File {
            path: "test.json".to_string(),
        });

        let test_secret = "super-secret-password";
        let encrypted = manager
            .encrypt_secret(test_secret, &master_key, None)
            .unwrap();
        let decrypted = manager.decrypt_secret(&encrypted, &master_key).unwrap();

        assert_eq!(test_secret, decrypted);
    }

    #[tokio::test]
    async fn test_secret_storage_file_backend() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_path = temp_dir.path().join("secrets.json");

        let mut manager = SecretManager::new(SecretBackend::File {
            path: secrets_path.to_string_lossy().to_string(),
        });

        let master_key = MasterKey::generate();
        manager.initialize(master_key).await.unwrap();

        // Store secret
        manager
            .store_secret("test-key", "test-value", None)
            .await
            .unwrap();

        // Retrieve secret
        let retrieved = manager.get_secret("test-key").await.unwrap();
        assert_eq!(Some("test-value".to_string()), retrieved);

        // List secrets
        let secrets = manager.list_secrets().await.unwrap();
        assert!(secrets.contains(&"test-key".to_string()));

        // Delete secret
        assert!(manager.delete_secret("test-key").await.unwrap());
        let deleted = manager.get_secret("test-key").await.unwrap();
        assert_eq!(None, deleted);
    }

    #[tokio::test]
    async fn test_master_key_from_password() {
        let password = "test-password";
        let salt = b"test-salt-16-b--";
        let key1 = MasterKey::from_password(password, salt).unwrap();
        let key2 = MasterKey::from_password(password, salt).unwrap();

        // Same password and salt should generate same key
        assert_eq!(key1.as_bytes(), key2.as_bytes());

        // Different salt should generate different key
        let salt2 = b"different-salt--";
        let key3 = MasterKey::from_password(password, salt2).unwrap();
        assert_ne!(key1.as_bytes(), key3.as_bytes());
    }
}
