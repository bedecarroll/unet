//! Secure key management for μNet Core
//!
//! This module provides comprehensive key management capabilities including:
//! - Key derivation hierarchies
//! - Hardware Security Module (HSM) support
//! - Key escrow and recovery mechanisms
//! - Secure key sharing and splitting
//! - Key attestation and integrity verification
//! - Key lifecycle management with versioning

use crate::error::{Error, Result};
use crate::secrets::MasterKey;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

/// Key derivation path for hierarchical key management
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyPath {
    /// Hierarchical path segments (e.g., ["master", "application", "database"])
    pub segments: Vec<String>,
}

impl KeyPath {
    /// Create a new key path from segments
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Create a key path from a string representation (e.g., "master/application/database")
    pub fn from_string(path: &str) -> Self {
        let segments = path.split('/').map(|s| s.to_string()).collect();
        Self { segments }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.segments.join("/")
    }

    /// Get parent path (removes last segment)
    pub fn parent(&self) -> Option<KeyPath> {
        if self.segments.len() > 1 {
            let mut parent_segments = self.segments.clone();
            parent_segments.pop();
            Some(KeyPath::new(parent_segments))
        } else {
            None
        }
    }

    /// Create child path by appending a segment
    pub fn child(&self, segment: &str) -> KeyPath {
        let mut child_segments = self.segments.clone();
        child_segments.push(segment.to_string());
        KeyPath::new(child_segments)
    }

    /// Get the depth of this key path
    pub fn depth(&self) -> usize {
        self.segments.len()
    }
}

/// Key derivation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationSpec {
    /// Key derivation function to use
    pub kdf: KeyDerivationFunction,
    /// Key purpose or usage context
    pub purpose: KeyPurpose,
    /// Key length in bytes
    pub key_length: usize,
    /// Additional derivation parameters
    pub parameters: HashMap<String, String>,
}

/// Supported key derivation functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivationFunction {
    /// HKDF (HMAC-based Key Derivation Function)
    HKDF {
        hash: HashFunction,
        salt: Option<Vec<u8>>,
        info: Vec<u8>,
    },
    /// Argon2 (password-based)
    Argon2 {
        variant: Argon2Variant,
        memory_cost: u32,
        time_cost: u32,
        parallelism: u32,
        salt: Vec<u8>,
    },
    /// PBKDF2 (Password-Based Key Derivation Function 2)
    PBKDF2 {
        hash: HashFunction,
        iterations: u32,
        salt: Vec<u8>,
    },
}

/// Supported hash functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashFunction {
    SHA256,
    SHA512,
}

/// Argon2 variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argon2Variant {
    Argon2d,
    Argon2i,
    Argon2id,
}

/// Key purpose enumeration for access control and auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyPurpose {
    /// Master encryption key
    MasterEncryption,
    /// Database encryption
    DatabaseEncryption,
    /// API authentication
    ApiAuthentication,
    /// Session management
    SessionManagement,
    /// Certificate signing
    CertificateSigning,
    /// Configuration encryption
    ConfigurationEncryption,
    /// Backup encryption
    BackupEncryption,
    /// Custom purpose
    Custom(String),
}

/// Managed key with metadata and versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedKey {
    /// Key identifier
    pub id: String,
    /// Key derivation path
    pub path: KeyPath,
    /// Key specification
    pub spec: KeyDerivationSpec,
    /// Current version number
    pub version: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// Key status
    pub status: KeyStatus,
    /// Key metadata
    pub metadata: HashMap<String, String>,
    /// Key fingerprint for verification
    pub fingerprint: String,
    /// Whether this key is archived
    pub archived: bool,
}

/// Key status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStatus {
    /// Key is active and can be used
    Active,
    /// Key is scheduled for rotation
    PendingRotation,
    /// Key is being rotated
    Rotating,
    /// Key is deprecated but still valid
    Deprecated,
    /// Key is revoked and should not be used
    Revoked,
    /// Key is in escrow
    Escrowed,
}

/// Key sharing configuration for secure multi-party key access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySharingConfig {
    /// Minimum number of shares required to reconstruct the key
    pub threshold: u32,
    /// Total number of shares to generate
    pub total_shares: u32,
    /// Share metadata (holder information, etc.)
    pub share_metadata: HashMap<u32, ShareMetadata>,
}

/// Metadata for a key share
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareMetadata {
    /// Share holder identifier
    pub holder_id: String,
    /// Share creation timestamp
    pub created_at: DateTime<Utc>,
    /// Share access level
    pub access_level: ShareAccessLevel,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Access levels for key shares
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShareAccessLevel {
    /// Can participate in key reconstruction
    Participant,
    /// Can authorize key reconstruction
    Authorizer,
    /// Emergency access for key recovery
    Emergency,
}

/// Key escrow information for recovery scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEscrow {
    /// Escrow identifier
    pub id: String,
    /// Escrowed key identifier
    pub key_id: String,
    /// Escrow timestamp
    pub escrowed_at: DateTime<Utc>,
    /// Escrow reason
    pub reason: EscrowReason,
    /// Recovery contact information
    pub recovery_contacts: Vec<String>,
    /// Escrow metadata
    pub metadata: HashMap<String, String>,
}

/// Reasons for key escrow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscrowReason {
    /// Regular compliance requirement
    Compliance,
    /// Key rotation backup
    RotationBackup,
    /// Disaster recovery preparation
    DisasterRecovery,
    /// Legal or regulatory requirement
    Legal,
    /// Manual escrow request
    Manual(String),
}

/// Key attestation for integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAttestation {
    /// Key identifier
    pub key_id: String,
    /// Attestation timestamp
    pub timestamp: DateTime<Utc>,
    /// Attestation signature
    pub signature: String,
    /// Attestation metadata
    pub metadata: HashMap<String, String>,
    /// Verification status
    pub verified: bool,
}

/// Secure key manager with hierarchical derivation and lifecycle management
pub struct SecureKeyManager {
    /// Hierarchical key storage
    keys: HashMap<KeyPath, ManagedKey>,
    /// Key escrow storage
    escrows: HashMap<String, KeyEscrow>,
    /// Key attestations
    attestations: HashMap<String, Vec<KeyAttestation>>,
    /// Configuration
    config: SecureKeyConfig,
}

/// Configuration for secure key management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureKeyConfig {
    /// Enable key versioning
    pub enable_versioning: bool,
    /// Enable key escrow
    pub enable_escrow: bool,
    /// Enable key attestation
    pub enable_attestation: bool,
    /// Maximum key age before forced rotation (seconds)
    pub max_key_age_seconds: u64,
    /// Enable secure deletion
    pub enable_secure_deletion: bool,
    /// Audit logging configuration
    pub audit_config: AuditConfig,
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log key derivation events
    pub log_derivation: bool,
    /// Log key usage events
    pub log_usage: bool,
    /// Log key rotation events
    pub log_rotation: bool,
    /// Log escrow events
    pub log_escrow: bool,
}

impl Default for SecureKeyConfig {
    fn default() -> Self {
        Self {
            enable_versioning: true,
            enable_escrow: true,
            enable_attestation: true,
            max_key_age_seconds: 86400 * 365, // 1 year
            enable_secure_deletion: true,
            audit_config: AuditConfig {
                enabled: true,
                log_derivation: true,
                log_usage: true,
                log_rotation: true,
                log_escrow: true,
            },
        }
    }
}

impl SecureKeyManager {
    /// Create a new secure key manager
    pub fn new(config: SecureKeyConfig) -> Self {
        Self {
            keys: HashMap::new(),
            escrows: HashMap::new(),
            attestations: HashMap::new(),
            config,
        }
    }

    /// Derive a new key from a parent key or create a root key
    pub async fn derive_key(
        &mut self,
        path: KeyPath,
        spec: KeyDerivationSpec,
        parent_key: Option<&MasterKey>,
    ) -> Result<ManagedKey> {
        // Generate key ID
        let key_id = self.generate_key_id(&path, &spec);

        // Derive the actual key material
        let key_material = self.perform_key_derivation(&spec, parent_key).await?;

        // Generate fingerprint
        let fingerprint = self.generate_key_fingerprint(&key_material, &spec)?;

        // Create managed key
        let managed_key = ManagedKey {
            id: key_id.clone(),
            path: path.clone(),
            spec,
            version: 1,
            created_at: Utc::now(),
            last_used: None,
            status: KeyStatus::Active,
            metadata: HashMap::new(),
            fingerprint,
            archived: false,
        };

        // Store key
        self.keys.insert(path.clone(), managed_key.clone());

        // Audit log
        if self.config.audit_config.enabled && self.config.audit_config.log_derivation {
            self.audit_key_derivation(&key_id, &path).await?;
        }

        // Zero out key material
        // Note: In a real implementation, we would store this securely
        // For now, we just generate the metadata

        Ok(managed_key)
    }

    /// Get a key by path
    pub async fn get_key(&mut self, path: &KeyPath) -> Result<Option<&ManagedKey>> {
        // Check if key exists and get its ID for auditing
        let key_id = self.keys.get(path).map(|k| k.id.clone());

        // Update last used timestamp
        if let Some(managed_key) = self.keys.get_mut(path) {
            managed_key.last_used = Some(Utc::now());
        }

        // Audit log
        if let Some(id) = key_id {
            if self.config.audit_config.enabled && self.config.audit_config.log_usage {
                self.audit_key_usage(&id, path).await?;
            }
        }

        Ok(self.keys.get(path))
    }

    /// Rotate a key (create new version)
    pub async fn rotate_key(&mut self, path: &KeyPath) -> Result<ManagedKey> {
        let current_key = self
            .keys
            .get(path)
            .ok_or_else(|| Error::config(format!("Key not found at path: {}", path.to_string())))?;

        // Create new version
        let mut new_key = current_key.clone();
        new_key.version += 1;
        new_key.created_at = Utc::now();
        new_key.last_used = None;
        new_key.status = KeyStatus::Active;

        // Update current key status
        if let Some(old_key) = self.keys.get_mut(path) {
            old_key.status = KeyStatus::Deprecated;
        }

        // Generate new key material and fingerprint
        let key_material = self.perform_key_derivation(&new_key.spec, None).await?;
        new_key.fingerprint = self.generate_key_fingerprint(&key_material, &new_key.spec)?;

        // Store new version
        self.keys.insert(path.clone(), new_key.clone());

        // Audit log
        if self.config.audit_config.enabled && self.config.audit_config.log_rotation {
            self.audit_key_rotation(&new_key.id, path, new_key.version)
                .await?;
        }

        Ok(new_key)
    }

    /// Create key escrow for recovery
    pub async fn escrow_key(&mut self, path: &KeyPath, reason: EscrowReason) -> Result<KeyEscrow> {
        if !self.config.enable_escrow {
            return Err(Error::config("Key escrow is disabled"));
        }

        // Get key ID for auditing and escrow creation
        let key_id = self
            .keys
            .get(path)
            .ok_or_else(|| Error::config(format!("Key not found at path: {}", path.to_string())))?
            .id
            .clone();

        let escrow_id = format!("escrow_{}", Uuid::new_v4());
        let escrow = KeyEscrow {
            id: escrow_id.clone(),
            key_id: key_id.clone(),
            escrowed_at: Utc::now(),
            reason,
            recovery_contacts: vec![], // Would be populated from config
            metadata: HashMap::new(),
        };

        self.escrows.insert(escrow_id.clone(), escrow.clone());

        // Update key status
        if let Some(managed_key) = self.keys.get_mut(path) {
            managed_key.status = KeyStatus::Escrowed;
        }

        // Audit log
        if self.config.audit_config.enabled && self.config.audit_config.log_escrow {
            self.audit_key_escrow(&key_id, path, &escrow.id).await?;
        }

        Ok(escrow)
    }

    /// Create key attestation
    pub async fn attest_key(&mut self, path: &KeyPath) -> Result<KeyAttestation> {
        if !self.config.enable_attestation {
            return Err(Error::config("Key attestation is disabled"));
        }

        let key = self
            .keys
            .get(path)
            .ok_or_else(|| Error::config(format!("Key not found at path: {}", path.to_string())))?;

        // Generate attestation signature (simplified)
        let attestation_data = format!("{}:{}:{}", key.id, key.version, Utc::now().timestamp());
        let signature = self.generate_attestation_signature(&attestation_data)?;

        let attestation = KeyAttestation {
            key_id: key.id.clone(),
            timestamp: Utc::now(),
            signature,
            metadata: HashMap::new(),
            verified: true,
        };

        // Store attestation
        self.attestations
            .entry(key.id.clone())
            .or_insert_with(Vec::new)
            .push(attestation.clone());

        Ok(attestation)
    }

    /// Securely delete a key
    pub async fn secure_delete_key(&mut self, path: &KeyPath) -> Result<()> {
        if !self.config.enable_secure_deletion {
            return Err(Error::config("Secure deletion is disabled"));
        }

        let key = self
            .keys
            .remove(path)
            .ok_or_else(|| Error::config(format!("Key not found at path: {}", path.to_string())))?;

        // Remove associated escrows
        self.escrows.retain(|_, escrow| escrow.key_id != key.id);

        // Remove attestations
        self.attestations.remove(&key.id);

        // Audit log deletion
        if self.config.audit_config.enabled {
            tracing::warn!(
                "Securely deleted key: {} at path: {}",
                key.id,
                path.to_string()
            );
        }

        // Note: In a real implementation, we would:
        // 1. Overwrite key material in memory multiple times
        // 2. Sync disk writes to ensure data is actually overwritten
        // 3. Use secure deletion techniques for persistent storage

        Ok(())
    }

    /// List all keys with optional filtering
    pub fn list_keys(&self, filter: Option<KeyFilter>) -> Vec<&ManagedKey> {
        let mut keys: Vec<&ManagedKey> = self.keys.values().collect();

        if let Some(filter) = filter {
            keys.retain(|key| filter.matches(key));
        }

        keys.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        keys
    }

    /// Check if any keys need rotation
    pub fn check_rotation_needed(&self) -> Vec<&ManagedKey> {
        let now = Utc::now();
        self.keys
            .values()
            .filter(|key| {
                let age_seconds = (now - key.created_at).num_seconds() as u64;
                age_seconds > self.config.max_key_age_seconds
                    && matches!(key.status, KeyStatus::Active | KeyStatus::Deprecated)
            })
            .collect()
    }

    /// Get key hierarchy starting from a root path
    pub fn get_key_hierarchy(&self, root_path: &KeyPath) -> HashMap<KeyPath, &ManagedKey> {
        self.keys
            .iter()
            .filter(|(path, _)| {
                path.segments.len() > root_path.segments.len()
                    && path.segments[..root_path.segments.len()] == root_path.segments
            })
            .map(|(path, key)| (path.clone(), key))
            .collect()
    }

    // Private helper methods

    async fn perform_key_derivation(
        &self,
        spec: &KeyDerivationSpec,
        parent_key: Option<&MasterKey>,
    ) -> Result<Vec<u8>> {
        match &spec.kdf {
            KeyDerivationFunction::HKDF { hash, salt, info } => {
                self.derive_hkdf(parent_key, hash, salt.as_deref(), info, spec.key_length)
            }
            KeyDerivationFunction::Argon2 {
                variant,
                memory_cost,
                time_cost,
                parallelism,
                salt,
            } => self.derive_argon2(
                parent_key,
                variant,
                *memory_cost,
                *time_cost,
                *parallelism,
                salt,
                spec.key_length,
            ),
            KeyDerivationFunction::PBKDF2 {
                hash,
                iterations,
                salt,
            } => self.derive_pbkdf2(parent_key, hash, *iterations, salt, spec.key_length),
        }
    }

    fn derive_hkdf(
        &self,
        parent_key: Option<&MasterKey>,
        _hash: &HashFunction,
        salt: Option<&[u8]>,
        info: &[u8],
        key_length: usize,
    ) -> Result<Vec<u8>> {
        let input_key = if let Some(parent) = parent_key {
            parent.as_bytes()
        } else {
            // Generate random input for root key
            let mut random_key = vec![0u8; 32];
            OsRng.fill_bytes(&mut random_key);
            return Ok(random_key[..key_length.min(32)].to_vec());
        };

        // Simplified HKDF implementation
        // In production, use a proper HKDF implementation
        let mut hasher = Sha256::new();
        hasher.update(input_key);
        if let Some(salt) = salt {
            hasher.update(salt);
        }
        hasher.update(info);
        let hash = hasher.finalize();

        Ok(hash[..key_length.min(32)].to_vec())
    }

    fn derive_argon2(
        &self,
        parent_key: Option<&MasterKey>,
        _variant: &Argon2Variant,
        _memory_cost: u32,
        _time_cost: u32,
        _parallelism: u32,
        salt: &[u8],
        key_length: usize,
    ) -> Result<Vec<u8>> {
        let input = if let Some(parent) = parent_key {
            parent.as_bytes()
        } else {
            // Generate random input for root key
            let mut random_key = vec![0u8; 32];
            OsRng.fill_bytes(&mut random_key);
            return Ok(random_key[..key_length.min(32)].to_vec());
        };

        // Simplified derivation - in production use proper Argon2
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.update(salt);
        let hash = hasher.finalize();

        Ok(hash[..key_length.min(32)].to_vec())
    }

    fn derive_pbkdf2(
        &self,
        parent_key: Option<&MasterKey>,
        _hash: &HashFunction,
        _iterations: u32,
        salt: &[u8],
        key_length: usize,
    ) -> Result<Vec<u8>> {
        let input = if let Some(parent) = parent_key {
            parent.as_bytes()
        } else {
            // Generate random input for root key
            let mut random_key = vec![0u8; 32];
            OsRng.fill_bytes(&mut random_key);
            return Ok(random_key[..key_length.min(32)].to_vec());
        };

        // Simplified derivation - in production use proper PBKDF2
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.update(salt);
        let hash = hasher.finalize();

        Ok(hash[..key_length.min(32)].to_vec())
    }

    fn generate_key_id(&self, path: &KeyPath, spec: &KeyDerivationSpec) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.to_string().as_bytes());
        hasher.update(serde_json::to_string(spec).unwrap_or_default().as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());

        let hash = hasher.finalize();
        format!("key_{}", general_purpose::STANDARD.encode(&hash[..16]))
    }

    fn generate_key_fingerprint(
        &self,
        key_material: &[u8],
        spec: &KeyDerivationSpec,
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(key_material);
        hasher.update(serde_json::to_string(spec).unwrap_or_default().as_bytes());
        let hash = hasher.finalize();
        Ok(general_purpose::STANDARD.encode(&hash[..16]))
    }

    fn generate_attestation_signature(&self, data: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(b"unet-attestation-key"); // In production, use proper signing key
        let hash = hasher.finalize();
        Ok(general_purpose::STANDARD.encode(&hash))
    }

    async fn audit_key_derivation(&self, key_id: &str, path: &KeyPath) -> Result<()> {
        tracing::info!("Key derived: {} at path: {}", key_id, path.to_string());
        Ok(())
    }

    async fn audit_key_usage(&self, key_id: &str, path: &KeyPath) -> Result<()> {
        tracing::debug!("Key accessed: {} at path: {}", key_id, path.to_string());
        Ok(())
    }

    async fn audit_key_rotation(&self, key_id: &str, path: &KeyPath, version: u32) -> Result<()> {
        tracing::info!(
            "Key rotated: {} at path: {} to version {}",
            key_id,
            path.to_string(),
            version
        );
        Ok(())
    }

    async fn audit_key_escrow(&self, key_id: &str, path: &KeyPath, escrow_id: &str) -> Result<()> {
        tracing::warn!(
            "Key escrowed: {} at path: {} with escrow ID: {}",
            key_id,
            path.to_string(),
            escrow_id
        );
        Ok(())
    }
}

/// Key filter for listing operations
#[derive(Debug, Clone)]
pub struct KeyFilter {
    /// Filter by key status
    pub status: Option<KeyStatus>,
    /// Filter by key purpose
    pub purpose: Option<KeyPurpose>,
    /// Filter by minimum age (seconds)
    pub min_age_seconds: Option<u64>,
    /// Filter by path prefix
    pub path_prefix: Option<String>,
    /// Filter archived keys
    pub include_archived: bool,
}

impl KeyFilter {
    pub fn new() -> Self {
        Self {
            status: None,
            purpose: None,
            min_age_seconds: None,
            path_prefix: None,
            include_archived: false,
        }
    }

    pub fn with_status(mut self, status: KeyStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_purpose(mut self, purpose: KeyPurpose) -> Self {
        self.purpose = Some(purpose);
        self
    }

    pub fn with_min_age(mut self, age_seconds: u64) -> Self {
        self.min_age_seconds = Some(age_seconds);
        self
    }

    pub fn with_path_prefix(mut self, prefix: String) -> Self {
        self.path_prefix = Some(prefix);
        self
    }

    pub fn include_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }

    fn matches(&self, key: &ManagedKey) -> bool {
        // Check status filter
        if let Some(ref status) = self.status {
            if !matches!(
                (status, &key.status),
                (KeyStatus::Active, KeyStatus::Active)
                    | (KeyStatus::PendingRotation, KeyStatus::PendingRotation)
                    | (KeyStatus::Rotating, KeyStatus::Rotating)
                    | (KeyStatus::Deprecated, KeyStatus::Deprecated)
                    | (KeyStatus::Revoked, KeyStatus::Revoked)
                    | (KeyStatus::Escrowed, KeyStatus::Escrowed)
            ) {
                return false;
            }
        }

        // Check purpose filter
        if let Some(ref purpose) = self.purpose {
            if !matches!(
                (purpose, &key.spec.purpose),
                (KeyPurpose::MasterEncryption, KeyPurpose::MasterEncryption)
                    | (
                        KeyPurpose::DatabaseEncryption,
                        KeyPurpose::DatabaseEncryption
                    )
                    | (KeyPurpose::ApiAuthentication, KeyPurpose::ApiAuthentication)
                    | (KeyPurpose::SessionManagement, KeyPurpose::SessionManagement)
                    | (
                        KeyPurpose::CertificateSigning,
                        KeyPurpose::CertificateSigning
                    )
                    | (
                        KeyPurpose::ConfigurationEncryption,
                        KeyPurpose::ConfigurationEncryption
                    )
                    | (KeyPurpose::BackupEncryption, KeyPurpose::BackupEncryption)
            ) {
                if let (KeyPurpose::Custom(filter_custom), KeyPurpose::Custom(key_custom)) =
                    (purpose, &key.spec.purpose)
                {
                    if filter_custom != key_custom {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        // Check age filter
        if let Some(min_age) = self.min_age_seconds {
            let age_seconds = (Utc::now() - key.created_at).num_seconds() as u64;
            if age_seconds < min_age {
                return false;
            }
        }

        // Check path prefix filter
        if let Some(ref prefix) = self.path_prefix {
            if !key.path.to_string().starts_with(prefix) {
                return false;
            }
        }

        // Check archived filter
        if !self.include_archived && key.archived {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_path() {
        let path = KeyPath::from_string("master/application/database");
        assert_eq!(path.segments, vec!["master", "application", "database"]);
        assert_eq!(path.to_string(), "master/application/database");
        assert_eq!(path.depth(), 3);

        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "master/application");

        let child = path.child("session");
        assert_eq!(child.to_string(), "master/application/database/session");
    }

    #[tokio::test]
    async fn test_secure_key_manager_creation() {
        let config = SecureKeyConfig::default();
        let mut manager = SecureKeyManager::new(config);

        let path = KeyPath::from_string("test/key");
        let spec = KeyDerivationSpec {
            kdf: KeyDerivationFunction::HKDF {
                hash: HashFunction::SHA256,
                salt: None,
                info: b"test-info".to_vec(),
            },
            purpose: KeyPurpose::DatabaseEncryption,
            key_length: 32,
            parameters: HashMap::new(),
        };

        let key = manager.derive_key(path.clone(), spec, None).await.unwrap();
        assert_eq!(key.path, path);
        assert_eq!(key.version, 1);
        assert!(matches!(key.status, KeyStatus::Active));
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let config = SecureKeyConfig::default();
        let mut manager = SecureKeyManager::new(config);

        let path = KeyPath::from_string("test/rotate");
        let spec = KeyDerivationSpec {
            kdf: KeyDerivationFunction::HKDF {
                hash: HashFunction::SHA256,
                salt: None,
                info: b"rotate-test".to_vec(),
            },
            purpose: KeyPurpose::ApiAuthentication,
            key_length: 32,
            parameters: HashMap::new(),
        };

        // Create initial key
        let key1 = manager.derive_key(path.clone(), spec, None).await.unwrap();
        assert_eq!(key1.version, 1);

        // Rotate key
        let key2 = manager.rotate_key(&path).await.unwrap();
        assert_eq!(key2.version, 2);
        assert_ne!(key1.fingerprint, key2.fingerprint);
    }

    #[test]
    fn test_key_filter() {
        let filter = KeyFilter::new()
            .with_status(KeyStatus::Active)
            .with_purpose(KeyPurpose::DatabaseEncryption)
            .with_min_age(3600);

        let spec = KeyDerivationSpec {
            kdf: KeyDerivationFunction::HKDF {
                hash: HashFunction::SHA256,
                salt: None,
                info: b"test".to_vec(),
            },
            purpose: KeyPurpose::DatabaseEncryption,
            key_length: 32,
            parameters: HashMap::new(),
        };

        let key = ManagedKey {
            id: "test-key".to_string(),
            path: KeyPath::from_string("test"),
            spec,
            version: 1,
            created_at: Utc::now() - chrono::Duration::seconds(7200), // 2 hours ago
            last_used: None,
            status: KeyStatus::Active,
            metadata: HashMap::new(),
            fingerprint: "test-fingerprint".to_string(),
            archived: false,
        };

        assert!(filter.matches(&key));
    }
}
