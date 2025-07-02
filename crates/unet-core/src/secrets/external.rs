//! External secret store providers for μNet Core
//!
//! This module provides integrations with external secret management systems
//! like HashiCorp Vault, AWS Secrets Manager, and Azure Key Vault.

use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// External secret provider trait
#[async_trait]
pub trait ExternalSecretProvider: Send + Sync {
    /// Store a secret in the external provider
    async fn store_secret(
        &mut self,
        name: &str,
        value: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()>;

    /// Retrieve a secret from the external provider
    async fn get_secret(&mut self, name: &str) -> Result<Option<String>>;

    /// List all secret names in the external provider
    async fn list_secrets(&self) -> Result<Vec<String>>;

    /// Delete a secret from the external provider
    async fn delete_secret(&mut self, name: &str) -> Result<bool>;

    /// Test connectivity to the external provider
    async fn test_connection(&self) -> Result<()>;

    /// Get provider information
    fn provider_info(&self) -> ProviderInfo;
}

/// Information about a secret provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub version: String,
    pub endpoint: Option<String>,
    pub authentication_type: String,
    pub supports_metadata: bool,
    pub supports_versioning: bool,
}

/// HashiCorp Vault secret provider
pub struct VaultProvider {
    client: vaultrs::client::VaultClient,
    mount_path: String,
    namespace: Option<String>,
}

impl VaultProvider {
    /// Create a new Vault provider
    pub async fn new(
        endpoint: &str,
        token: &str,
        mount_path: Option<String>,
        namespace: Option<String>,
    ) -> Result<Self> {
        let client = vaultrs::client::VaultClient::new(
            vaultrs::client::VaultClientSettingsBuilder::default()
                .address(endpoint)
                .token(token)
                .namespace(namespace.clone())
                .build()
                .map_err(|e| Error::config(format!("Failed to configure Vault client: {}", e)))?,
        )
        .map_err(|e| Error::config(format!("Failed to create Vault client: {}", e)))?;

        Ok(Self {
            client,
            mount_path: mount_path.unwrap_or_else(|| "secret".to_string()),
            namespace,
        })
    }

    /// Create provider with certificate authentication (disabled - API not available)
    pub async fn new_with_cert(
        endpoint: &str,
        _cert_path: &str,
        _key_path: &str,
        mount_path: Option<String>,
        namespace: Option<String>,
    ) -> Result<Self> {
        // Certificate authentication not available in current vaultrs version
        Self::new(endpoint, "token", mount_path, namespace).await
    }
}

#[async_trait]
impl ExternalSecretProvider for VaultProvider {
    async fn store_secret(
        &mut self,
        name: &str,
        value: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let mut secret_data = HashMap::new();
        secret_data.insert(
            "value".to_string(),
            serde_json::Value::String(value.to_string()),
        );

        if let Some(meta) = metadata {
            for (key, val) in meta {
                secret_data.insert(format!("meta_{}", key), serde_json::Value::String(val));
            }
        }

        let path = format!("{}/{}", self.mount_path, name);
        vaultrs::kv2::set(&self.client, &self.mount_path, &path, &secret_data)
            .await
            .map_err(|e| Error::config(format!("Failed to store secret in Vault: {}", e)))?;

        Ok(())
    }

    async fn get_secret(&mut self, name: &str) -> Result<Option<String>> {
        let path = format!("{}/{}", self.mount_path, name);
        match vaultrs::kv2::read::<HashMap<String, serde_json::Value>>(
            &self.client,
            &self.mount_path,
            &path,
        )
        .await
        {
            Ok(secret) => {
                if let Some(value) = secret.get("value") {
                    if let Some(value_str) = value.as_str() {
                        Ok(Some(value_str.to_string()))
                    } else {
                        Err(Error::config("Secret value is not a string"))
                    }
                } else {
                    Ok(None)
                }
            }
            Err(vaultrs::error::ClientError::APIError {
                code: 404,
                errors: _,
            }) => Ok(None),
            Err(e) => Err(Error::config(format!(
                "Failed to read secret from Vault: {}",
                e
            ))),
        }
    }

    async fn list_secrets(&self) -> Result<Vec<String>> {
        match vaultrs::kv2::list(&self.client, &self.mount_path, "").await {
            Ok(keys) => Ok(keys),
            Err(vaultrs::error::ClientError::APIError {
                code: 404,
                errors: _,
            }) => Ok(vec![]),
            Err(e) => Err(Error::config(format!(
                "Failed to list secrets from Vault: {}",
                e
            ))),
        }
    }

    async fn delete_secret(&mut self, name: &str) -> Result<bool> {
        let path = format!("{}/{}", self.mount_path, name);
        match vaultrs::kv2::delete_latest(&self.client, &self.mount_path, &path).await {
            Ok(_) => Ok(true),
            Err(vaultrs::error::ClientError::APIError {
                code: 404,
                errors: _,
            }) => Ok(false),
            Err(e) => Err(Error::config(format!(
                "Failed to delete secret from Vault: {}",
                e
            ))),
        }
    }

    async fn test_connection(&self) -> Result<()> {
        vaultrs::sys::health(&self.client)
            .await
            .map_err(|e| Error::config(format!("Vault connection test failed: {}", e)))?;
        Ok(())
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "HashiCorp Vault".to_string(),
            version: "KV v2".to_string(),
            endpoint: Some("vault-endpoint".to_string()), // TODO: Get actual endpoint
            authentication_type: "Token/Certificate".to_string(),
            supports_metadata: true,
            supports_versioning: true,
        }
    }
}

/// AWS Secrets Manager provider
pub struct AwsSecretsManagerProvider {
    client: aws_sdk_secretsmanager::Client,
    region: String,
    prefix: Option<String>,
}

impl AwsSecretsManagerProvider {
    /// Create a new AWS Secrets Manager provider
    pub async fn new(region: Option<String>, prefix: Option<String>) -> Result<Self> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = aws_sdk_secretsmanager::Client::new(&config);

        Ok(Self {
            client,
            region: region.unwrap_or_else(|| {
                config
                    .region()
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "us-east-1".to_string())
            }),
            prefix,
        })
    }

    /// Create provider with explicit credentials
    pub async fn new_with_credentials(
        access_key: &str,
        secret_key: &str,
        region: Option<String>,
        prefix: Option<String>,
    ) -> Result<Self> {
        let credentials = aws_sdk_secretsmanager::config::Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "unet-secrets",
        );

        let region_str = region.unwrap_or_else(|| "us-east-1".to_string());
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region_str.clone()))
            .credentials_provider(credentials)
            .load()
            .await;

        let client = aws_sdk_secretsmanager::Client::new(&config);

        Ok(Self {
            client,
            region: region_str,
            prefix,
        })
    }

    fn get_secret_name(&self, name: &str) -> String {
        if let Some(prefix) = &self.prefix {
            format!("{}/{}", prefix, name)
        } else {
            name.to_string()
        }
    }
}

#[async_trait]
impl ExternalSecretProvider for AwsSecretsManagerProvider {
    async fn store_secret(
        &mut self,
        name: &str,
        value: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let secret_name = self.get_secret_name(name);
        let description = metadata
            .as_ref()
            .and_then(|m| m.get("description"))
            .cloned()
            .unwrap_or_else(|| format!("μNet secret: {}", name));

        // Try to update existing secret first
        match self
            .client
            .update_secret()
            .secret_id(&secret_name)
            .secret_string(value)
            .description(&description)
            .send()
            .await
        {
            Ok(_) => return Ok(()),
            Err(aws_sdk_secretsmanager::error::SdkError::ServiceError(service_err)) => {
                if service_err.err().is_resource_not_found_exception() {
                    // Secret doesn't exist, create it
                } else {
                    return Err(Error::config(format!(
                        "Failed to update secret in AWS Secrets Manager: {:?}",
                        service_err
                    )));
                }
            }
            Err(e) => {
                return Err(Error::config(format!(
                    "Failed to update secret in AWS Secrets Manager: {}",
                    e
                )));
            }
        }

        // Create new secret
        let mut builder = self
            .client
            .create_secret()
            .name(&secret_name)
            .secret_string(value)
            .description(&description);

        // Add tags from metadata
        if let Some(meta) = metadata {
            let tags: Vec<_> = meta
                .iter()
                .map(|(k, v)| {
                    aws_sdk_secretsmanager::types::Tag::builder()
                        .key(k)
                        .value(v)
                        .build()
                })
                .collect();
            builder = builder.set_tags(Some(tags));
        }

        builder.send().await.map_err(|e| {
            Error::config(format!(
                "Failed to create secret in AWS Secrets Manager: {}",
                e
            ))
        })?;

        Ok(())
    }

    async fn get_secret(&mut self, name: &str) -> Result<Option<String>> {
        let secret_name = self.get_secret_name(name);
        match self
            .client
            .get_secret_value()
            .secret_id(&secret_name)
            .send()
            .await
        {
            Ok(response) => Ok(response.secret_string),
            Err(aws_sdk_secretsmanager::error::SdkError::ServiceError(service_err)) => {
                if service_err.err().is_resource_not_found_exception() {
                    Ok(None)
                } else {
                    Err(Error::config(format!(
                        "Failed to get secret from AWS Secrets Manager: {:?}",
                        service_err
                    )))
                }
            }
            Err(e) => Err(Error::config(format!(
                "Failed to get secret from AWS Secrets Manager: {}",
                e
            ))),
        }
    }

    async fn list_secrets(&self) -> Result<Vec<String>> {
        let mut secrets = Vec::new();
        let mut next_token = None;

        loop {
            let mut builder = self.client.list_secrets().max_results(100);
            if let Some(token) = next_token {
                builder = builder.next_token(token);
            }

            let response = builder.send().await.map_err(|e| {
                Error::config(format!(
                    "Failed to list secrets from AWS Secrets Manager: {}",
                    e
                ))
            })?;

            if let Some(secret_list) = response.secret_list {
                for secret in secret_list {
                    if let Some(name) = secret.name {
                        // Remove prefix if it exists
                        let clean_name = if let Some(prefix) = &self.prefix {
                            name.strip_prefix(&format!("{}/", prefix))
                                .unwrap_or(&name)
                                .to_string()
                        } else {
                            name
                        };
                        secrets.push(clean_name);
                    }
                }
            }

            next_token = response.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(secrets)
    }

    async fn delete_secret(&mut self, name: &str) -> Result<bool> {
        let secret_name = self.get_secret_name(name);
        match self
            .client
            .delete_secret()
            .secret_id(&secret_name)
            .force_delete_without_recovery(true)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(aws_sdk_secretsmanager::error::SdkError::ServiceError(service_err)) => {
                if service_err.err().is_resource_not_found_exception() {
                    Ok(false)
                } else {
                    Err(Error::config(format!(
                        "Failed to delete secret from AWS Secrets Manager: {:?}",
                        service_err
                    )))
                }
            }
            Err(e) => Err(Error::config(format!(
                "Failed to delete secret from AWS Secrets Manager: {}",
                e
            ))),
        }
    }

    async fn test_connection(&self) -> Result<()> {
        // Test by attempting to list secrets with a limit of 1
        self.client
            .list_secrets()
            .max_results(1)
            .send()
            .await
            .map_err(|e| {
                Error::config(format!("AWS Secrets Manager connection test failed: {}", e))
            })?;
        Ok(())
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "AWS Secrets Manager".to_string(),
            version: "2017-10-17".to_string(),
            endpoint: Some(format!("secretsmanager.{}.amazonaws.com", self.region)),
            authentication_type: "AWS IAM".to_string(),
            supports_metadata: true,
            supports_versioning: true,
        }
    }
}

/// Azure Key Vault provider (placeholder - Azure SDK has API changes)
pub struct AzureKeyVaultProvider {
    vault_url: String,
}

impl AzureKeyVaultProvider {
    /// Create a new Azure Key Vault provider with default credential
    pub async fn new(vault_url: &str) -> Result<Self> {
        Ok(Self {
            vault_url: vault_url.to_string(),
        })
    }

    /// Create provider with service principal authentication
    pub async fn new_with_service_principal(
        vault_url: &str,
        _tenant_id: &str,
        _client_id: &str,
        _client_secret: &str,
    ) -> Result<Self> {
        Ok(Self {
            vault_url: vault_url.to_string(),
        })
    }
}

#[async_trait]
impl ExternalSecretProvider for AzureKeyVaultProvider {
    async fn store_secret(
        &mut self,
        _name: &str,
        _value: &str,
        _metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        Err(Error::config(
            "Azure Key Vault support is temporarily disabled due to SDK changes",
        ))
    }

    async fn get_secret(&mut self, _name: &str) -> Result<Option<String>> {
        Err(Error::config(
            "Azure Key Vault support is temporarily disabled due to SDK changes",
        ))
    }

    async fn list_secrets(&self) -> Result<Vec<String>> {
        Err(Error::config(
            "Azure Key Vault support is temporarily disabled due to SDK changes",
        ))
    }

    async fn delete_secret(&mut self, _name: &str) -> Result<bool> {
        Err(Error::config(
            "Azure Key Vault support is temporarily disabled due to SDK changes",
        ))
    }

    async fn test_connection(&self) -> Result<()> {
        Err(Error::config(
            "Azure Key Vault support is temporarily disabled due to SDK changes",
        ))
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "Azure Key Vault".to_string(),
            version: "7.4 (disabled)".to_string(),
            endpoint: Some(self.vault_url.clone()),
            authentication_type: "Azure AD".to_string(),
            supports_metadata: true,
            supports_versioning: true,
        }
    }
}

/// External provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalProviderConfig {
    pub provider_type: ExternalProviderType,
    pub config: HashMap<String, String>,
}

/// Supported external provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalProviderType {
    Vault,
    AwsSecretsManager,
    AzureKeyVault,
}

/// Create an external secret provider from configuration
pub async fn create_external_provider(
    provider_config: &ExternalProviderConfig,
) -> Result<Box<dyn ExternalSecretProvider>> {
    match provider_config.provider_type {
        ExternalProviderType::Vault => {
            let endpoint = provider_config
                .config
                .get("endpoint")
                .ok_or_else(|| Error::config("Vault endpoint is required"))?;
            let token = provider_config
                .config
                .get("token")
                .ok_or_else(|| Error::config("Vault token is required"))?;
            let mount_path = provider_config.config.get("mount_path").cloned();
            let namespace = provider_config.config.get("namespace").cloned();

            let provider = VaultProvider::new(endpoint, token, mount_path, namespace).await?;
            Ok(Box::new(provider))
        }
        ExternalProviderType::AwsSecretsManager => {
            let region = provider_config.config.get("region").cloned();
            let prefix = provider_config.config.get("prefix").cloned();

            let provider = if let (Some(access_key), Some(secret_key)) = (
                provider_config.config.get("access_key"),
                provider_config.config.get("secret_key"),
            ) {
                AwsSecretsManagerProvider::new_with_credentials(
                    access_key, secret_key, region, prefix,
                )
                .await?
            } else {
                AwsSecretsManagerProvider::new(region, prefix).await?
            };

            Ok(Box::new(provider))
        }
        ExternalProviderType::AzureKeyVault => {
            let vault_url = provider_config
                .config
                .get("vault_url")
                .ok_or_else(|| Error::config("Azure Key Vault URL is required"))?;

            let provider = if let (Some(tenant_id), Some(client_id), Some(client_secret)) = (
                provider_config.config.get("tenant_id"),
                provider_config.config.get("client_id"),
                provider_config.config.get("client_secret"),
            ) {
                AzureKeyVaultProvider::new_with_service_principal(
                    vault_url,
                    tenant_id,
                    client_id,
                    client_secret,
                )
                .await?
            } else {
                AzureKeyVaultProvider::new(vault_url).await?
            };

            Ok(Box::new(provider))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_provider_config_serialization() {
        let mut config = HashMap::new();
        config.insert(
            "endpoint".to_string(),
            "https://vault.example.com".to_string(),
        );
        config.insert("token".to_string(), "hvs.test-token".to_string());

        let provider_config = ExternalProviderConfig {
            provider_type: ExternalProviderType::Vault,
            config,
        };

        let serialized = serde_json::to_string(&provider_config).unwrap();
        let deserialized: ExternalProviderConfig = serde_json::from_str(&serialized).unwrap();

        assert!(matches!(
            deserialized.provider_type,
            ExternalProviderType::Vault
        ));
        assert_eq!(
            deserialized.config.get("endpoint").unwrap(),
            "https://vault.example.com"
        );
    }

    #[test]
    fn test_provider_info() {
        let info = ProviderInfo {
            name: "Test Provider".to_string(),
            version: "1.0".to_string(),
            endpoint: Some("https://test.example.com".to_string()),
            authentication_type: "Token".to_string(),
            supports_metadata: true,
            supports_versioning: false,
        };

        assert_eq!(info.name, "Test Provider");
        assert!(info.supports_metadata);
        assert!(!info.supports_versioning);
    }
}
