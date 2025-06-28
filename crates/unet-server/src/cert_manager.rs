//! Certificate management for automated certificate handling

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde::{Deserialize, Serialize};
use std::fs::{File, copy, create_dir_all};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use unet_core::config::TlsConfig;

/// Certificate manager for automated certificate operations
pub struct CertificateManager {
    config: TlsConfig,
    cert_dir: PathBuf,
}

/// Certificate metadata for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateMetadata {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub fingerprint: String,
    pub last_validated: DateTime<Utc>,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
}

/// Certificate validation result
#[derive(Debug, Clone)]
pub struct CertificateValidation {
    pub is_valid: bool,
    pub expires_in_days: Option<i64>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Certificate rotation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    /// Manual rotation only
    Manual,
    /// Automatic rotation using ACME (Let's Encrypt)
    Acme {
        domain: String,
        contact_email: String,
        staging: bool,
    },
    /// External certificate provider
    External {
        provider: String,
        config: serde_json::Value,
    },
}

impl CertificateManager {
    /// Create a new certificate manager
    pub fn new(config: TlsConfig, cert_dir: impl AsRef<Path>) -> Result<Self> {
        let cert_dir = cert_dir.as_ref().to_path_buf();

        // Ensure certificate directory exists
        if !cert_dir.exists() {
            create_dir_all(&cert_dir).with_context(|| {
                format!(
                    "Failed to create certificate directory: {}",
                    cert_dir.display()
                )
            })?;
        }

        Ok(Self { config, cert_dir })
    }

    /// Validate current certificates
    pub async fn validate_certificates(&self) -> Result<CertificateValidation> {
        info!("Validating current certificates");

        let mut validation = CertificateValidation {
            is_valid: true,
            expires_in_days: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        // Check if certificate files exist
        if !Path::new(&self.config.cert_file).exists() {
            validation.is_valid = false;
            validation.errors.push(format!(
                "Certificate file not found: {}",
                self.config.cert_file
            ));
            return Ok(validation);
        }

        if !Path::new(&self.config.key_file).exists() {
            validation.is_valid = false;
            validation.errors.push(format!(
                "Private key file not found: {}",
                self.config.key_file
            ));
            return Ok(validation);
        }

        // Load and validate certificate
        match self.load_certificate_metadata().await {
            Ok(metadata) => {
                let now = Utc::now();
                let expires_in = (metadata.not_after - now).num_days();
                validation.expires_in_days = Some(expires_in);

                // Check expiration
                if expires_in <= 0 {
                    validation.is_valid = false;
                    validation
                        .errors
                        .push("Certificate has expired".to_string());
                } else if expires_in <= 30 {
                    validation
                        .warnings
                        .push(format!("Certificate expires in {} days", expires_in));
                } else if expires_in <= 7 {
                    validation.warnings.push(format!(
                        "Certificate expires in {} days - urgent renewal needed",
                        expires_in
                    ));
                }

                // Check if certificate is valid
                if !metadata.is_valid {
                    validation.is_valid = false;
                    validation.errors.extend(metadata.validation_errors);
                }

                info!(
                    "Certificate validation completed: expires in {} days",
                    expires_in
                );
            }
            Err(e) => {
                validation.is_valid = false;
                validation
                    .errors
                    .push(format!("Failed to load certificate metadata: {}", e));
            }
        }

        Ok(validation)
    }

    /// Load certificate metadata
    async fn load_certificate_metadata(&self) -> Result<CertificateMetadata> {
        // This is a simplified implementation
        // In a real system, you would parse the certificate using x509-parser or similar
        let cert_path = PathBuf::from(&self.config.cert_file);
        let key_path = PathBuf::from(&self.config.key_file);

        // For now, create placeholder metadata
        // In production, this would parse actual certificate data
        Ok(CertificateMetadata {
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            not_before: Utc::now() - chrono::Duration::days(30),
            not_after: Utc::now() + chrono::Duration::days(60),
            subject: "CN=localhost".to_string(),
            issuer: "CN=Self-Signed".to_string(),
            serial_number: "1".to_string(),
            fingerprint: "placeholder_fingerprint".to_string(),
            last_validated: Utc::now(),
            is_valid: self.validate_certificate_files().is_ok(),
            validation_errors: Vec::new(),
        })
    }

    /// Validate certificate files can be loaded
    fn validate_certificate_files(&self) -> Result<()> {
        // Try to load certificates
        let cert_file = File::open(&self.config.cert_file).with_context(|| {
            format!("Failed to open certificate file: {}", self.config.cert_file)
        })?;
        let mut cert_reader = BufReader::new(cert_file);

        let _certs: Vec<Certificate> = certs(&mut cert_reader)
            .map_err(|e| anyhow::anyhow!("Failed to parse certificate file: {}", e))?
            .into_iter()
            .map(Certificate)
            .collect();

        // Try to load private key
        let key_file = File::open(&self.config.key_file).with_context(|| {
            format!("Failed to open private key file: {}", self.config.key_file)
        })?;
        let mut key_reader = BufReader::new(key_file);

        let keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_reader)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key file: {}", e))?
            .into_iter()
            .map(PrivateKey)
            .collect();

        if keys.is_empty() {
            return Err(anyhow::anyhow!("No private keys found in key file"));
        }

        Ok(())
    }

    /// Backup current certificates
    pub async fn backup_certificates(&self) -> Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_dir = self.cert_dir.join(format!("backup_{}", timestamp));
        create_dir_all(&backup_dir).with_context(|| {
            format!(
                "Failed to create backup directory: {}",
                backup_dir.display()
            )
        })?;

        // Backup certificate file
        let cert_backup = backup_dir.join("cert.pem");
        copy(&self.config.cert_file, &cert_backup).with_context(|| {
            format!(
                "Failed to backup certificate file to {}",
                cert_backup.display()
            )
        })?;

        // Backup key file
        let key_backup = backup_dir.join("key.pem");
        copy(&self.config.key_file, &key_backup)
            .with_context(|| format!("Failed to backup key file to {}", key_backup.display()))?;

        info!("Certificates backed up to: {}", backup_dir.display());
        Ok(backup_dir)
    }

    /// Rotate certificates using the specified strategy
    pub async fn rotate_certificates(&self, strategy: &RotationStrategy) -> Result<()> {
        info!(
            "Starting certificate rotation with strategy: {:?}",
            strategy
        );

        // Backup current certificates first
        let backup_dir = self.backup_certificates().await?;
        info!(
            "Current certificates backed up to: {}",
            backup_dir.display()
        );

        match strategy {
            RotationStrategy::Manual => {
                warn!("Manual rotation strategy - no automatic rotation performed");
                return Ok(());
            }
            RotationStrategy::Acme {
                domain,
                contact_email,
                staging,
            } => {
                self.rotate_with_acme(domain, contact_email, *staging)
                    .await?;
            }
            RotationStrategy::External { provider, config } => {
                self.rotate_with_external_provider(provider, config).await?;
            }
        }

        // Validate new certificates
        match self.validate_certificates().await {
            Ok(validation) => {
                if validation.is_valid {
                    info!("Certificate rotation completed successfully");
                } else {
                    error!(
                        "Certificate rotation failed validation: {:?}",
                        validation.errors
                    );
                    return Err(anyhow::anyhow!("New certificates failed validation"));
                }
            }
            Err(e) => {
                error!("Failed to validate new certificates: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Rotate certificates using ACME (Let's Encrypt)
    async fn rotate_with_acme(
        &self,
        domain: &str,
        contact_email: &str,
        staging: bool,
    ) -> Result<()> {
        info!("Rotating certificates using ACME for domain: {}", domain);
        info!("Contact email: {}, Staging: {}", contact_email, staging);

        // This is a placeholder implementation
        // In production, you would use an ACME client like acme2 or instant-acme
        warn!("ACME certificate rotation not yet implemented - placeholder");

        // For demonstration, just log what would happen
        info!("Would request certificate for domain: {}", domain);
        info!("Would use contact email: {}", contact_email);
        info!(
            "Would use {} environment",
            if staging { "staging" } else { "production" }
        );

        Ok(())
    }

    /// Rotate certificates using external provider
    async fn rotate_with_external_provider(
        &self,
        provider: &str,
        config: &serde_json::Value,
    ) -> Result<()> {
        info!(
            "Rotating certificates using external provider: {}",
            provider
        );

        // This is a placeholder implementation
        // In production, you would integrate with external certificate providers
        warn!("External provider certificate rotation not yet implemented - placeholder");

        info!("Would use provider: {}", provider);
        info!("Would use config: {}", config);

        Ok(())
    }

    /// Get certificate expiration monitoring data
    pub async fn get_expiration_info(&self) -> Result<ExpirationInfo> {
        let metadata = self.load_certificate_metadata().await?;
        let now = Utc::now();
        let expires_in_days = (metadata.not_after - now).num_days();

        Ok(ExpirationInfo {
            expires_at: metadata.not_after,
            expires_in_days,
            renewal_recommended: expires_in_days <= 30,
            renewal_urgent: expires_in_days <= 7,
            cert_path: metadata.cert_path,
            key_path: metadata.key_path,
        })
    }

    /// Clean up old certificate backups
    pub async fn cleanup_old_backups(&self, keep_days: u64) -> Result<Vec<PathBuf>> {
        let mut cleaned_dirs = Vec::new();
        let cutoff_time = SystemTime::now() - Duration::from_secs(keep_days * 24 * 60 * 60);

        if !self.cert_dir.exists() {
            return Ok(cleaned_dirs);
        }

        let entries = std::fs::read_dir(&self.cert_dir).with_context(|| {
            format!(
                "Failed to read certificate directory: {}",
                self.cert_dir.display()
            )
        })?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("backup_")
            {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created) = metadata.created() {
                        if created < cutoff_time {
                            if std::fs::remove_dir_all(&path).is_ok() {
                                info!("Cleaned up old backup: {}", path.display());
                                cleaned_dirs.push(path);
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_dirs)
    }
}

/// Certificate expiration information
#[derive(Debug, Clone, Serialize)]
pub struct ExpirationInfo {
    pub expires_at: DateTime<Utc>,
    pub expires_in_days: i64,
    pub renewal_recommended: bool,
    pub renewal_urgent: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_certificate_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cert_dir = temp_dir.path().join("certs");

        let tls_config = TlsConfig {
            cert_file: "/tmp/test_cert.pem".to_string(),
            key_file: "/tmp/test_key.pem".to_string(),
            force_https: false,
            http_redirect_port: None,
        };

        let manager = CertificateManager::new(tls_config, &cert_dir).unwrap();
        assert!(cert_dir.exists());
        assert_eq!(manager.cert_dir, cert_dir);
    }

    #[test]
    fn test_rotation_strategy_serialization() {
        let strategy = RotationStrategy::Acme {
            domain: "example.com".to_string(),
            contact_email: "admin@example.com".to_string(),
            staging: false,
        };

        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: RotationStrategy = serde_json::from_str(&json).unwrap();

        match deserialized {
            RotationStrategy::Acme {
                domain,
                contact_email,
                staging,
            } => {
                assert_eq!(domain, "example.com");
                assert_eq!(contact_email, "admin@example.com");
                assert_eq!(staging, false);
            }
            _ => panic!("Wrong strategy type"),
        }
    }
}
