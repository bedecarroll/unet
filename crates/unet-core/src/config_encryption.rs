//! Encrypted configuration file support for μNet Core
//!
//! This module provides functionality to encrypt entire configuration files
//! and manage encrypted configuration storage with versioning and backup support.

use crate::error::{Error, Result};
use crate::secrets::{MasterKey, SecretManager};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

/// Encrypted configuration file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedConfigMetadata {
    /// Configuration format (toml, yaml, json)
    pub format: String,
    /// Encryption algorithm used
    pub algorithm: String,
    /// Configuration version
    pub version: u32,
    /// Timestamp when encrypted
    pub encrypted_at: DateTime<Utc>,
    /// Optional description
    pub description: Option<String>,
    /// Hash of original content for integrity verification
    pub content_hash: String,
    /// Backup file locations (for rollback)
    pub backups: Vec<String>,
}

/// Encrypted configuration file container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedConfigFile {
    /// Metadata about the encrypted configuration
    pub metadata: EncryptedConfigMetadata,
    /// Base64-encoded encrypted configuration data
    pub encrypted_data: String,
    /// Base64-encoded nonce/IV
    pub nonce: String,
}

/// Configuration encryption manager
pub struct ConfigEncryption {
    secret_manager: SecretManager,
    master_key_name: String,
}

impl ConfigEncryption {
    /// Create a new configuration encryption manager
    pub fn new(secret_manager: SecretManager) -> Self {
        Self {
            secret_manager,
            master_key_name: "config-master-key".to_string(),
        }
    }

    /// Set custom master key name for configuration encryption
    pub fn with_master_key_name(mut self, name: String) -> Self {
        self.master_key_name = name;
        self
    }

    /// Encrypt a configuration file
    pub async fn encrypt_config_file(
        &mut self,
        config_path: &Path,
        output_path: &Path,
        description: Option<String>,
    ) -> Result<()> {
        // Read the original configuration file
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| Error::config(format!("Failed to read config file: {}", e)))?;

        // Detect format from file extension
        let format = self.detect_config_format(config_path)?;

        // Validate configuration format
        self.validate_config_content(&config_content, &format)?;

        // Calculate content hash for integrity
        let content_hash = self.calculate_content_hash(&config_content);

        // Get or create master key
        let master_key = self.get_or_create_master_key().await?;

        // Encrypt the configuration content
        let (encrypted_data, nonce) = self.encrypt_content(&config_content, &master_key)?;

        // Create metadata
        let metadata = EncryptedConfigMetadata {
            format,
            algorithm: "AES-256-GCM".to_string(),
            version: 1,
            encrypted_at: Utc::now(),
            description,
            content_hash,
            backups: Vec::new(),
        };

        // Create encrypted config file
        let encrypted_config = EncryptedConfigFile {
            metadata,
            encrypted_data: general_purpose::STANDARD.encode(encrypted_data),
            nonce: general_purpose::STANDARD.encode(nonce),
        };

        // Write encrypted configuration to output file
        let encrypted_json = serde_json::to_string_pretty(&encrypted_config)
            .map_err(|e| Error::config(format!("Failed to serialize encrypted config: {}", e)))?;

        fs::write(output_path, encrypted_json)
            .map_err(|e| Error::config(format!("Failed to write encrypted config: {}", e)))?;

        tracing::info!(
            "Configuration file encrypted: {} -> {}",
            config_path.display(),
            output_path.display()
        );

        Ok(())
    }

    /// Decrypt a configuration file
    pub async fn decrypt_config_file(
        &mut self,
        encrypted_path: &Path,
        output_path: Option<&Path>,
    ) -> Result<String> {
        // Read encrypted configuration file
        let encrypted_content = fs::read_to_string(encrypted_path)
            .map_err(|e| Error::config(format!("Failed to read encrypted config: {}", e)))?;

        // Parse encrypted configuration
        let encrypted_config: EncryptedConfigFile = serde_json::from_str(&encrypted_content)
            .map_err(|e| Error::config(format!("Failed to parse encrypted config: {}", e)))?;

        // Get master key
        let master_key = self.get_master_key().await?;

        // Decrypt the configuration content
        let decrypted_content = self.decrypt_content(
            &encrypted_config.encrypted_data,
            &encrypted_config.nonce,
            &master_key,
        )?;

        // Verify content integrity
        let calculated_hash = self.calculate_content_hash(&decrypted_content);
        if calculated_hash != encrypted_config.metadata.content_hash {
            return Err(Error::config(
                "Configuration content integrity check failed".to_string(),
            ));
        }

        // Validate decrypted configuration
        self.validate_config_content(&decrypted_content, &encrypted_config.metadata.format)?;

        // Write to output file if specified
        if let Some(output_path) = output_path {
            fs::write(output_path, &decrypted_content)
                .map_err(|e| Error::config(format!("Failed to write decrypted config: {}", e)))?;

            tracing::info!(
                "Configuration file decrypted: {} -> {}",
                encrypted_path.display(),
                output_path.display()
            );
        }

        Ok(decrypted_content)
    }

    /// Create a backup of an encrypted configuration file
    pub async fn backup_encrypted_config(
        &self,
        encrypted_path: &Path,
        backup_dir: &Path,
    ) -> Result<String> {
        // Ensure backup directory exists
        fs::create_dir_all(backup_dir)
            .map_err(|e| Error::config(format!("Failed to create backup directory: {}", e)))?;

        // Generate backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!(
            "{}_{}.encrypted",
            encrypted_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy(),
            timestamp
        );
        let backup_path = backup_dir.join(backup_filename);

        // Copy encrypted file to backup location
        fs::copy(encrypted_path, &backup_path)
            .map_err(|e| Error::config(format!("Failed to create backup: {}", e)))?;

        // Update metadata to include backup reference
        self.update_backup_metadata(encrypted_path, &backup_path)
            .await?;

        tracing::info!("Configuration backup created: {}", backup_path.display());

        Ok(backup_path.to_string_lossy().to_string())
    }

    /// List available backups for an encrypted configuration
    pub async fn list_backups(&self, encrypted_path: &Path) -> Result<Vec<String>> {
        let encrypted_content = fs::read_to_string(encrypted_path)
            .map_err(|e| Error::config(format!("Failed to read encrypted config: {}", e)))?;

        let encrypted_config: EncryptedConfigFile = serde_json::from_str(&encrypted_content)
            .map_err(|e| Error::config(format!("Failed to parse encrypted config: {}", e)))?;

        Ok(encrypted_config.metadata.backups)
    }

    /// Restore from backup
    pub async fn restore_from_backup(&self, backup_path: &Path, target_path: &Path) -> Result<()> {
        fs::copy(backup_path, target_path)
            .map_err(|e| Error::config(format!("Failed to restore from backup: {}", e)))?;

        tracing::info!(
            "Configuration restored from backup: {} -> {}",
            backup_path.display(),
            target_path.display()
        );

        Ok(())
    }

    /// Rotate master key and re-encrypt configuration
    pub async fn rotate_master_key(&mut self, encrypted_path: &Path) -> Result<()> {
        // Create backup before rotation
        let backup_dir = encrypted_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("backups");
        self.backup_encrypted_config(encrypted_path, &backup_dir)
            .await?;

        // Decrypt with old key
        let decrypted_content = self.decrypt_config_file(encrypted_path, None).await?;

        // Create new master key
        let new_master_key = MasterKey::generate();
        let key_b64 = general_purpose::STANDARD.encode(new_master_key.as_bytes());
        self.secret_manager
            .store_secret(&self.master_key_name, &key_b64, None)
            .await?;

        // Re-encrypt with new key
        let content_hash = self.calculate_content_hash(&decrypted_content);
        let (encrypted_data, nonce) = self.encrypt_content(&decrypted_content, &new_master_key)?;

        // Read current metadata
        let encrypted_content = fs::read_to_string(encrypted_path)
            .map_err(|e| Error::config(format!("Failed to read encrypted config: {}", e)))?;
        let mut encrypted_config: EncryptedConfigFile = serde_json::from_str(&encrypted_content)
            .map_err(|e| Error::config(format!("Failed to parse encrypted config: {}", e)))?;

        // Update with new encryption
        encrypted_config.metadata.version += 1;
        encrypted_config.metadata.encrypted_at = Utc::now();
        encrypted_config.metadata.content_hash = content_hash;
        encrypted_config.encrypted_data = general_purpose::STANDARD.encode(encrypted_data);
        encrypted_config.nonce = general_purpose::STANDARD.encode(nonce);

        // Write updated encrypted configuration
        let encrypted_json = serde_json::to_string_pretty(&encrypted_config)
            .map_err(|e| Error::config(format!("Failed to serialize encrypted config: {}", e)))?;

        fs::write(encrypted_path, encrypted_json)
            .map_err(|e| Error::config(format!("Failed to write encrypted config: {}", e)))?;

        tracing::info!(
            "Master key rotated for configuration: {}",
            encrypted_path.display()
        );

        Ok(())
    }

    /// Get configuration metadata without decrypting
    pub async fn get_config_metadata(
        &self,
        encrypted_path: &Path,
    ) -> Result<EncryptedConfigMetadata> {
        let encrypted_content = fs::read_to_string(encrypted_path)
            .map_err(|e| Error::config(format!("Failed to read encrypted config: {}", e)))?;

        let encrypted_config: EncryptedConfigFile = serde_json::from_str(&encrypted_content)
            .map_err(|e| Error::config(format!("Failed to parse encrypted config: {}", e)))?;

        Ok(encrypted_config.metadata)
    }

    // Private helper methods

    /// Detect configuration format from file extension
    fn detect_config_format(&self, path: &Path) -> Result<String> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => Ok("toml".to_string()),
            Some("yaml" | "yml") => Ok("yaml".to_string()),
            Some("json") => Ok("json".to_string()),
            _ => Err(Error::config(
                "Unsupported configuration format".to_string(),
            )),
        }
    }

    /// Validate configuration content based on format
    fn validate_config_content(&self, content: &str, format: &str) -> Result<()> {
        match format {
            "toml" => {
                toml::from_str::<toml::Value>(content)
                    .map_err(|e| Error::config(format!("Invalid TOML format: {}", e)))?;
            }
            "yaml" => {
                serde_yaml::from_str::<serde_yaml::Value>(content)
                    .map_err(|e| Error::config(format!("Invalid YAML format: {}", e)))?;
            }
            "json" => {
                serde_json::from_str::<serde_json::Value>(content)
                    .map_err(|e| Error::config(format!("Invalid JSON format: {}", e)))?;
            }
            _ => {
                return Err(Error::config(
                    "Unsupported format for validation".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Calculate SHA-256 hash of content
    fn calculate_content_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get or create master key for configuration encryption
    async fn get_or_create_master_key(&mut self) -> Result<MasterKey> {
        match self.secret_manager.get_secret(&self.master_key_name).await {
            Ok(Some(key_b64)) => {
                let key_bytes = general_purpose::STANDARD
                    .decode(&key_b64)
                    .map_err(|e| Error::config(format!("Failed to decode master key: {}", e)))?;
                MasterKey::from_bytes(&key_bytes)
                    .map_err(|_| Error::config("Invalid master key data".to_string()))
            }
            Ok(None) => {
                // Create new master key
                let master_key = MasterKey::generate();
                let key_b64 = general_purpose::STANDARD.encode(master_key.as_bytes());
                self.secret_manager
                    .store_secret(&self.master_key_name, &key_b64, None)
                    .await?;
                Ok(master_key)
            }
            Err(e) => Err(e),
        }
    }

    /// Get existing master key
    async fn get_master_key(&mut self) -> Result<MasterKey> {
        let key_b64 = self
            .secret_manager
            .get_secret(&self.master_key_name)
            .await?
            .ok_or_else(|| Error::config("Master key not found".to_string()))?;

        let key_bytes = general_purpose::STANDARD
            .decode(&key_b64)
            .map_err(|e| Error::config(format!("Failed to decode master key: {}", e)))?;

        MasterKey::from_bytes(&key_bytes)
            .map_err(|_| Error::config("Invalid master key data".to_string()))
    }

    /// Encrypt content using AES-256-GCM
    fn encrypt_content(&self, content: &str, master_key: &MasterKey) -> Result<(Vec<u8>, Vec<u8>)> {
        let cipher = Aes256Gcm::new_from_slice(master_key.as_bytes())
            .map_err(|e| Error::config(format!("Failed to create cipher: {}", e)))?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let encrypted_data = cipher
            .encrypt(&nonce, content.as_bytes())
            .map_err(|e| Error::config(format!("Encryption failed: {}", e)))?;

        Ok((encrypted_data, nonce.to_vec()))
    }

    /// Decrypt content using AES-256-GCM
    fn decrypt_content(
        &self,
        encrypted_data_b64: &str,
        nonce_b64: &str,
        master_key: &MasterKey,
    ) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(master_key.as_bytes())
            .map_err(|e| Error::config(format!("Failed to create cipher: {}", e)))?;

        let encrypted_data = general_purpose::STANDARD
            .decode(encrypted_data_b64)
            .map_err(|e| Error::config(format!("Failed to decode encrypted data: {}", e)))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce_b64)
            .map_err(|e| Error::config(format!("Failed to decode nonce: {}", e)))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let decrypted_data = cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| Error::config(format!("Decryption failed: {}", e)))?;

        String::from_utf8(decrypted_data)
            .map_err(|e| Error::config(format!("Invalid UTF-8 in decrypted data: {}", e)))
    }

    /// Update backup metadata
    async fn update_backup_metadata(
        &self,
        encrypted_path: &Path,
        backup_path: &Path,
    ) -> Result<()> {
        let encrypted_content = fs::read_to_string(encrypted_path)
            .map_err(|e| Error::config(format!("Failed to read encrypted config: {}", e)))?;

        let mut encrypted_config: EncryptedConfigFile = serde_json::from_str(&encrypted_content)
            .map_err(|e| Error::config(format!("Failed to parse encrypted config: {}", e)))?;

        // Add backup to metadata
        encrypted_config
            .metadata
            .backups
            .push(backup_path.to_string_lossy().to_string());

        // Write updated metadata
        let encrypted_json = serde_json::to_string_pretty(&encrypted_config)
            .map_err(|e| Error::config(format!("Failed to serialize encrypted config: {}", e)))?;

        fs::write(encrypted_path, encrypted_json)
            .map_err(|e| Error::config(format!("Failed to update encrypted config: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::SecretBackend;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_encrypt_decrypt_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        let encrypted_path = temp_dir.path().join("test.encrypted");

        // Create test configuration file
        let test_config = r#"
[database]
url = "sqlite:test.db"
max_connections = 10

[logging]
level = "info"
"#;
        fs::write(&config_path, test_config).unwrap();

        // Create configuration encryption manager
        let temp_secret = tempdir().unwrap();
        let mut secret_manager = SecretManager::new(SecretBackend::File {
            path: temp_secret
                .path()
                .join("secrets.json")
                .to_str()
                .unwrap()
                .to_string(),
        });
        secret_manager
            .initialize_with_password("test")
            .await
            .unwrap();
        let mut config_encryption = ConfigEncryption::new(secret_manager);

        // Encrypt configuration
        config_encryption
            .encrypt_config_file(
                &config_path,
                &encrypted_path,
                Some("Test config".to_string()),
            )
            .await
            .unwrap();

        // Verify encrypted file exists
        assert!(encrypted_path.exists());

        // Decrypt configuration
        let decrypted_content = config_encryption
            .decrypt_config_file(&encrypted_path, None)
            .await
            .unwrap();

        // Verify content matches
        assert_eq!(decrypted_content.trim(), test_config.trim());
    }

    #[tokio::test]
    async fn test_backup_and_restore() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        let encrypted_path = temp_dir.path().join("test.encrypted");
        let backup_dir = temp_dir.path().join("backups");

        // Create and encrypt test configuration
        let test_config = r#"[test]
value = "original"
"#;
        fs::write(&config_path, test_config).unwrap();

        let temp_secret = tempdir().unwrap();
        let mut secret_manager = SecretManager::new(SecretBackend::File {
            path: temp_secret
                .path()
                .join("secrets.json")
                .to_str()
                .unwrap()
                .to_string(),
        });
        secret_manager
            .initialize_with_password("test")
            .await
            .unwrap();
        let mut config_encryption = ConfigEncryption::new(secret_manager);

        config_encryption
            .encrypt_config_file(&config_path, &encrypted_path, None)
            .await
            .unwrap();

        // Create backup
        let backup_path = config_encryption
            .backup_encrypted_config(&encrypted_path, &backup_dir)
            .await
            .unwrap();

        assert!(Path::new(&backup_path).exists());

        // Restore from backup
        let restore_path = temp_dir.path().join("restored.encrypted");
        config_encryption
            .restore_from_backup(Path::new(&backup_path), &restore_path)
            .await
            .unwrap();

        // Verify restored content
        let restored_content = config_encryption
            .decrypt_config_file(&restore_path, None)
            .await
            .unwrap();

        assert_eq!(restored_content.trim(), test_config.trim());
    }
}
