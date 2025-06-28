//! Certificate management API handlers

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};
use unet_core::config::TlsConfig;

use crate::{
    cert_manager::{CertificateManager, RotationStrategy},
    server::AppState,
};

/// Certificate status response
#[derive(Serialize)]
pub struct CertificateStatusResponse {
    pub is_valid: bool,
    pub expires_in_days: Option<i64>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub last_validated: String,
}

/// Certificate rotation request
#[derive(Deserialize)]
pub struct CertificateRotationRequest {
    pub strategy: RotationStrategy,
    pub force: Option<bool>,
}

/// Certificate backup request query parameters
#[derive(Deserialize)]
pub struct BackupQuery {
    pub cleanup_days: Option<u64>,
}

/// Get certificate status and validation information
pub async fn get_certificate_status(
    State(app_state): State<AppState>,
) -> Result<Json<CertificateStatusResponse>, StatusCode> {
    info!("Getting certificate status");

    // Check if TLS is configured
    let tls_config = match &app_state.config.server.tls {
        Some(config) => config.clone(),
        None => {
            return Ok(Json(CertificateStatusResponse {
                is_valid: false,
                expires_in_days: None,
                warnings: vec!["TLS not configured".to_string()],
                errors: vec!["No TLS configuration found".to_string()],
                last_validated: chrono::Utc::now().to_rfc3339(),
            }));
        }
    };

    // Create certificate manager
    let cert_manager = match CertificateManager::new(tls_config, "./certs") {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to create certificate manager: {}", e);
            return Ok(Json(CertificateStatusResponse {
                is_valid: false,
                expires_in_days: None,
                warnings: Vec::new(),
                errors: vec![format!("Failed to create certificate manager: {}", e)],
                last_validated: chrono::Utc::now().to_rfc3339(),
            }));
        }
    };

    // Validate certificates
    match cert_manager.validate_certificates().await {
        Ok(validation) => {
            info!(
                "Certificate validation completed: valid={}",
                validation.is_valid
            );
            Ok(Json(CertificateStatusResponse {
                is_valid: validation.is_valid,
                expires_in_days: validation.expires_in_days,
                warnings: validation.warnings,
                errors: validation.errors,
                last_validated: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            warn!("Certificate validation failed: {}", e);
            Ok(Json(CertificateStatusResponse {
                is_valid: false,
                expires_in_days: None,
                warnings: Vec::new(),
                errors: vec![format!("Validation failed: {}", e)],
                last_validated: chrono::Utc::now().to_rfc3339(),
            }))
        }
    }
}

/// Rotate certificates using specified strategy
pub async fn rotate_certificates(
    State(app_state): State<AppState>,
    Json(request): Json<CertificateRotationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!(
        "Starting certificate rotation with strategy: {:?}",
        request.strategy
    );

    // Check if TLS is configured
    let tls_config = match &app_state.config.server.tls {
        Some(config) => config.clone(),
        None => {
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create certificate manager
    let cert_manager = match CertificateManager::new(tls_config, "./certs") {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to create certificate manager: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Perform rotation
    match cert_manager.rotate_certificates(&request.strategy).await {
        Ok(()) => {
            info!("Certificate rotation completed successfully");
            Ok(Json(serde_json::json!({
                "status": "success",
                "message": "Certificate rotation completed successfully",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(e) => {
            warn!("Certificate rotation failed: {}", e);
            Ok(Json(serde_json::json!({
                "status": "error",
                "message": format!("Certificate rotation failed: {}", e),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}

/// Backup current certificates
pub async fn backup_certificates(
    State(app_state): State<AppState>,
    Query(query): Query<BackupQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Starting certificate backup");

    // Check if TLS is configured
    let tls_config = match &app_state.config.server.tls {
        Some(config) => config.clone(),
        None => {
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create certificate manager
    let cert_manager = match CertificateManager::new(tls_config, "./certs") {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to create certificate manager: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Perform backup
    let backup_result = match cert_manager.backup_certificates().await {
        Ok(backup_dir) => {
            info!("Certificate backup completed: {}", backup_dir.display());

            let mut response = serde_json::Map::new();
            response.insert(
                "status".to_string(),
                serde_json::Value::String("success".to_string()),
            );
            response.insert(
                "message".to_string(),
                serde_json::Value::String("Certificate backup completed successfully".to_string()),
            );
            response.insert(
                "backup_path".to_string(),
                serde_json::Value::String(backup_dir.display().to_string()),
            );
            response.insert(
                "timestamp".to_string(),
                serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
            );

            serde_json::Value::Object(response)
        }
        Err(e) => {
            warn!("Certificate backup failed: {}", e);
            serde_json::json!({
                "status": "error",
                "message": format!("Certificate backup failed: {}", e),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        }
    };

    // Cleanup old backups if requested
    if let Some(cleanup_days) = query.cleanup_days {
        match cert_manager.cleanup_old_backups(cleanup_days).await {
            Ok(cleaned_dirs) => {
                info!("Cleaned up {} old backup directories", cleaned_dirs.len());
                // Note: We cannot modify backup_result here since it was already created
                // This would require restructuring the response creation
            }
            Err(e) => {
                warn!("Failed to cleanup old backups: {}", e);
            }
        }
    }

    Ok(Json(backup_result))
}

/// Get certificate expiration information
pub async fn get_certificate_expiration(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting certificate expiration information");

    // Check if TLS is configured
    let tls_config = match &app_state.config.server.tls {
        Some(config) => config.clone(),
        None => {
            return Ok(Json(serde_json::json!({
                "status": "error",
                "message": "TLS not configured",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })));
        }
    };

    // Create certificate manager
    let cert_manager = match CertificateManager::new(tls_config, "./certs") {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to create certificate manager: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get expiration info
    match cert_manager.get_expiration_info().await {
        Ok(expiration_info) => {
            info!(
                "Certificate expires in {} days",
                expiration_info.expires_in_days
            );
            Ok(Json(serde_json::json!({
                "status": "success",
                "expiration_info": expiration_info,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(e) => {
            warn!("Failed to get certificate expiration info: {}", e);
            Ok(Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to get expiration info: {}", e),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}

/// Health check for certificate management system
pub async fn certificate_health_check(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut health_data = HashMap::new();

    // Check if TLS is configured
    let tls_configured = app_state.config.server.tls.is_some();
    health_data.insert("tls_configured", serde_json::Value::Bool(tls_configured));

    if tls_configured {
        let tls_config = app_state.config.server.tls.as_ref().unwrap();

        // Check if certificate files exist
        let cert_exists = std::path::Path::new(&tls_config.cert_file).exists();
        let key_exists = std::path::Path::new(&tls_config.key_file).exists();

        health_data.insert("cert_file_exists", serde_json::Value::Bool(cert_exists));
        health_data.insert("key_file_exists", serde_json::Value::Bool(key_exists));
        health_data.insert(
            "cert_file_path",
            serde_json::Value::String(tls_config.cert_file.clone()),
        );
        health_data.insert(
            "key_file_path",
            serde_json::Value::String(tls_config.key_file.clone()),
        );
        health_data.insert(
            "force_https",
            serde_json::Value::Bool(tls_config.force_https),
        );

        if let Some(redirect_port) = tls_config.http_redirect_port {
            health_data.insert(
                "http_redirect_port",
                serde_json::Value::Number(redirect_port.into()),
            );
        }
    }

    health_data.insert(
        "timestamp",
        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
    );

    let health_map: serde_json::Map<String, serde_json::Value> = health_data
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    Ok(Json(serde_json::Value::Object(health_map)))
}
