//! TLS configuration and certificate management

use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};
use unet_core::config::TlsConfig;

/// TLS certificate manager for HTTPS support
pub struct TlsManager {
    config: TlsConfig,
}

impl TlsManager {
    /// Create a new TLS manager with the given configuration
    pub fn new(config: TlsConfig) -> Self {
        Self { config }
    }

    /// Load TLS configuration for axum-server
    pub async fn load_rustls_config(&self) -> Result<RustlsConfig> {
        info!("Loading TLS certificates from disk");

        // Load certificates
        let certs = self
            .load_certificates(&self.config.cert_file)
            .with_context(|| {
                format!("Failed to load certificate file: {}", self.config.cert_file)
            })?;

        // Load private key
        let key = self
            .load_private_key(&self.config.key_file)
            .with_context(|| {
                format!("Failed to load private key file: {}", self.config.key_file)
            })?;

        // Create server config
        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .with_context(|| "Failed to create TLS server configuration")?;

        let rustls_config = RustlsConfig::from_config(Arc::new(server_config));

        info!("TLS configuration loaded successfully");
        Ok(rustls_config)
    }

    /// Load certificates from PEM file
    fn load_certificates(&self, cert_file: &str) -> Result<Vec<Certificate>> {
        let file = File::open(cert_file)
            .with_context(|| format!("Failed to open certificate file: {}", cert_file))?;
        let mut reader = BufReader::new(file);

        let cert_chain: Vec<Certificate> = certs(&mut reader)
            .map_err(|e| anyhow::anyhow!("Failed to parse certificate file: {}", e))?
            .into_iter()
            .map(Certificate)
            .collect();

        info!(
            "Loaded {} certificates from {}",
            cert_chain.len(),
            cert_file
        );

        Ok(cert_chain)
    }

    /// Load private key from PEM file
    fn load_private_key(&self, key_file: &str) -> Result<PrivateKey> {
        let file = File::open(key_file)
            .with_context(|| format!("Failed to open private key file: {}", key_file))?;
        let mut reader = BufReader::new(file);

        let keys = pkcs8_private_keys(&mut reader)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key file: {}", e))?
            .into_iter()
            .map(PrivateKey)
            .collect::<Vec<_>>();

        if keys.is_empty() {
            return Err(anyhow::anyhow!(
                "No private keys found in key file: {}",
                key_file
            ));
        }

        if keys.len() > 1 {
            warn!(
                "Multiple private keys found in {}, using the first one",
                key_file
            );
        }

        info!("Loaded private key from {}", key_file);
        Ok(keys.into_iter().next().unwrap())
    }

    /// Validate certificate and key files
    pub fn validate_certificates(&self) -> Result<()> {
        info!("Validating TLS certificate configuration");

        // Check if files exist
        if !Path::new(&self.config.cert_file).exists() {
            return Err(anyhow::anyhow!(
                "Certificate file does not exist: {}",
                self.config.cert_file
            ));
        }

        if !Path::new(&self.config.key_file).exists() {
            return Err(anyhow::anyhow!(
                "Private key file does not exist: {}",
                self.config.key_file
            ));
        }

        // Try to load certificates to validate format
        let _certs = self
            .load_certificates(&self.config.cert_file)
            .with_context(|| "Certificate file validation failed")?;

        // Try to load private key to validate format
        let _key = self
            .load_private_key(&self.config.key_file)
            .with_context(|| "Private key file validation failed")?;

        info!("TLS certificate validation completed successfully");
        Ok(())
    }

    /// Get certificate information for monitoring/logging
    pub fn get_certificate_info(&self) -> Result<CertificateInfo> {
        let certs = self.load_certificates(&self.config.cert_file)?;

        if certs.is_empty() {
            return Err(anyhow::anyhow!("No certificates found"));
        }

        // Parse the first certificate for basic info
        let cert = &certs[0];

        Ok(CertificateInfo {
            cert_file: self.config.cert_file.clone(),
            key_file: self.config.key_file.clone(),
            cert_count: certs.len(),
            cert_size_bytes: cert.0.len(),
            force_https: self.config.force_https,
            http_redirect_port: self.config.http_redirect_port,
        })
    }
}

/// Certificate information for monitoring and logging
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    pub cert_file: String,
    pub key_file: String,
    pub cert_count: usize,
    pub cert_size_bytes: usize,
    pub force_https: bool,
    pub http_redirect_port: Option<u16>,
}

impl std::fmt::Display for CertificateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TLS Config: cert={}, key={}, certs={}, size={}B, force_https={}, redirect_port={:?}",
            self.cert_file,
            self.key_file,
            self.cert_count,
            self.cert_size_bytes,
            self.force_https,
            self.http_redirect_port
        )
    }
}

/// Generate self-signed certificate for development/testing
#[cfg(feature = "dev-certs")]
pub mod dev_certs {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Generate a self-signed certificate for development use
    pub fn generate_self_signed_cert(cert_path: &str, key_path: &str) -> Result<()> {
        // This would require additional dependencies like rcgen
        // For now, provide instructions for manual generation
        let instructions = format!(
            r#"
To generate a self-signed certificate for development, run:

openssl req -x509 -newkey rsa:4096 -keyout {} -out {} -days 365 -nodes \
    -subj "/C=US/ST=Dev/L=Dev/O=μNet/CN=localhost"

Or use mkcert for local development:
mkcert -install
mkcert -key-file {} -cert-file {} localhost 127.0.0.1 ::1
            "#,
            key_path, cert_path, key_path, cert_path
        );

        if !Path::new(cert_path).exists() || !Path::new(key_path).exists() {
            warn!("TLS certificates not found. {}", instructions);
            return Err(anyhow::anyhow!(
                "TLS certificates not found. {}",
                instructions
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_certificate_info_display() {
        let info = CertificateInfo {
            cert_file: "/path/to/cert.pem".to_string(),
            key_file: "/path/to/key.pem".to_string(),
            cert_count: 1,
            cert_size_bytes: 1024,
            force_https: true,
            http_redirect_port: Some(8080),
        };

        let display = format!("{}", info);
        assert!(display.contains("cert=/path/to/cert.pem"));
        assert!(display.contains("force_https=true"));
        assert!(display.contains("redirect_port=Some(8080)"));
    }

    #[test]
    fn test_tls_config_validation() {
        let config = TlsConfig {
            cert_file: "/nonexistent/cert.pem".to_string(),
            key_file: "/nonexistent/key.pem".to_string(),
            force_https: false,
            http_redirect_port: None,
        };

        let manager = TlsManager::new(config);
        let result = manager.validate_certificates();
        assert!(result.is_err());
    }
}
