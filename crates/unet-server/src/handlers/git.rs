//! Git version control API handlers
//!
//! This module provides REST API endpoints for Git integration functionality,
//! including sync status, change history, version control management, and webhooks.

use crate::error::{ServerError, ServerResult};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::{ApiResponse, PaginatedResponse},
    server::AppState,
};
use unet_core::{
    git::{BranchInfo, CommitInfo, FileChange, GitClient, GitClientConfig, RepositoryInfo},
    models::change_tracking::{
        ChangeAuditLog, ChangeSource, ChangeStatus, ChangeType, ConfigurationChange,
    },
};

/// Request to trigger Git synchronization
#[derive(Debug, Deserialize)]
pub struct GitSyncRequest {
    /// Force sync even if up to date
    #[serde(default)]
    pub force: bool,
    /// Sync only policies
    #[serde(default)]
    pub policies_only: bool,
    /// Sync only templates
    #[serde(default)]
    pub templates_only: bool,
    /// Target branch to sync (default: configured branch)
    pub branch: Option<String>,
}

/// Git synchronization status response
#[derive(Debug, Serialize)]
pub struct GitSyncStatusResponse {
    /// Current sync status
    pub status: String,
    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Next scheduled sync
    pub next_sync: Option<DateTime<Utc>>,
    /// Repository information
    pub repository: Option<RepositoryInfo>,
    /// Current branch
    pub current_branch: Option<String>,
    /// Sync statistics
    pub sync_stats: Option<SyncStatistics>,
    /// Any sync errors
    pub errors: Vec<String>,
}

/// Sync statistics
#[derive(Debug, Serialize)]
pub struct SyncStatistics {
    /// Number of policies synced
    pub policies_synced: u32,
    /// Number of templates synced
    pub templates_synced: u32,
    /// Number of files changed
    pub files_changed: u32,
    /// Sync duration in milliseconds
    pub duration_ms: u64,
}

/// Change history query parameters
#[derive(Debug, Deserialize)]
pub struct ChangeHistoryQuery {
    /// Entity type filter
    pub entity_type: Option<String>,
    /// Entity ID filter
    pub entity_id: Option<String>,
    /// Change type filter
    pub change_type: Option<ChangeType>,
    /// Change source filter
    pub source: Option<ChangeSource>,
    /// User ID filter
    pub user_id: Option<String>,
    /// Start date filter
    pub since: Option<DateTime<Utc>>,
    /// End date filter
    pub until: Option<DateTime<Utc>>,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

const fn default_page() -> u32 {
    1
}
const fn default_per_page() -> u32 {
    20
}

/// Change history response
#[derive(Debug, Serialize)]
pub struct ChangeHistoryResponse {
    /// Configuration change details
    #[serde(flatten)]
    pub change: ConfigurationChange,
    /// Audit trail for this change
    pub audit_trail: Vec<ChangeAuditLog>,
    /// Related changes
    pub related_changes: Vec<String>,
}

/// Git webhook payload
#[derive(Debug, Deserialize)]
pub struct GitWebhookPayload {
    /// Event type (push, pull_request, etc.)
    pub event_type: String,
    /// Repository information
    pub repository: WebhookRepository,
    /// Commits (for push events)
    pub commits: Option<Vec<WebhookCommit>>,
    /// Branch reference
    pub ref_name: Option<String>,
    /// Additional payload data
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookRepository {
    pub name: String,
    pub url: String,
    pub default_branch: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookCommit {
    pub id: String,
    pub message: String,
    pub author: WebhookAuthor,
    pub timestamp: DateTime<Utc>,
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookAuthor {
    pub name: String,
    pub email: String,
}

/// Repository status information for API responses
#[derive(Debug, Serialize)]
pub struct ApiRepositoryStatus {
    /// Current branch name
    pub current_branch: String,
    /// Whether the working directory is clean
    pub is_clean: bool,
    /// Number of commits ahead of remote
    pub commits_ahead: usize,
    /// Number of commits behind remote
    pub commits_behind: usize,
}

/// Helper function to create a GitClient instance
fn create_git_client(app_state: &AppState) -> GitClient {
    let git_config = GitClientConfig {
        base_directory: std::path::PathBuf::from("./git-repos"),
        default_sync_interval: app_state.config.git.sync_interval / 60, // Convert seconds to minutes
        max_state_age: 5,
        auto_fetch: true,
        auto_cleanup: false,
    };
    GitClient::with_config(git_config)
}

/// Version control repository information
#[derive(Debug, Serialize)]
pub struct RepositoryInfoResponse {
    /// Repository details
    #[serde(flatten)]
    pub info: RepositoryInfo,
    /// Current status
    pub status: ApiRepositoryStatus,
    /// Available branches
    pub branches: Vec<BranchInfo>,
    /// Recent commits
    pub recent_commits: Vec<CommitInfo>,
    /// Pending changes
    pub pending_changes: Vec<FileChange>,
}

/// Get Git synchronization status
pub async fn get_git_sync_status(
    State(_app_state): State<AppState>,
) -> ServerResult<Json<ApiResponse<GitSyncStatusResponse>>> {
    let response = GitSyncStatusResponse {
        status: "in_development".to_string(),
        last_sync: None,
        next_sync: Some(Utc::now() + chrono::Duration::minutes(30)),
        repository: None,
        current_branch: None,
        sync_stats: None,
        errors: vec!["Git integration in development".to_string()],
    };
    Ok(Json(ApiResponse::success(response)))
}

/// Trigger Git synchronization
pub async fn trigger_git_sync(
    State(_app_state): State<AppState>,
    Json(_request): Json<GitSyncRequest>,
) -> ServerResult<Json<ApiResponse<GitSyncStatusResponse>>> {
    let response = GitSyncStatusResponse {
        status: "in_development".to_string(),
        last_sync: None,
        next_sync: Some(Utc::now() + chrono::Duration::minutes(30)),
        repository: None,
        current_branch: None,
        sync_stats: None,
        errors: vec!["Git sync functionality in development".to_string()],
    };
    Ok(Json(ApiResponse::success(response)))
}

/// Get change history with filtering and pagination
pub async fn get_change_history(
    State(app_state): State<AppState>,
    Query(query): Query<ChangeHistoryQuery>,
) -> ServerResult<Json<ApiResponse<PaginatedResponse<ChangeHistoryResponse>>>> {
    // Use datastore to get actual change history
    let offset = (query.page - 1) * query.per_page;
    let limit = query.per_page;

    // Try to get changes from datastore
    let query_options = unet_core::datastore::QueryOptions {
        filters: vec![],
        sort: vec![],
        pagination: Some(
            unet_core::datastore::Pagination::new(limit as usize, offset as usize)
                .unwrap_or_else(|_| unet_core::datastore::Pagination::new(20, 0).unwrap()),
        ),
    };

    match app_state
        .datastore
        .list_configuration_changes(&query_options)
        .await
    {
        Ok(paged_result) => {
            let change_responses: Vec<ChangeHistoryResponse> = paged_result
                .items
                .into_iter()
                .map(|change| ChangeHistoryResponse {
                    change,
                    audit_trail: vec![],     // TODO: Get actual audit trail
                    related_changes: vec![], // TODO: Get related changes
                })
                .collect();

            let response =
                PaginatedResponse::from_paged_result(unet_core::datastore::PagedResult {
                    items: change_responses,
                    total_count: paged_result.total_count,
                    page: paged_result.page,
                    page_size: paged_result.page_size,
                    total_pages: paged_result.total_pages,
                    has_next: paged_result.has_next,
                    has_previous: paged_result.has_previous,
                });

            Ok(Json(ApiResponse::success(response)))
        }
        Err(_) => {
            // Fallback to placeholder response
            let changes = vec![ChangeHistoryResponse {
                change: ConfigurationChange {
                    id: Uuid::new_v4().to_string(),
                    change_type: ChangeType::Update,
                    entity_type: "git_sync".to_string(),
                    entity_id: "repository".to_string(),
                    user_id: Some("system".to_string()),
                    source: ChangeSource::GitSync,
                    description: Some("Git repository synchronization".to_string()),
                    old_value: None,
                    new_value: None,
                    diff_content: None,
                    git_commit: Some("latest".to_string()),
                    git_branch: Some("main".to_string()),
                    status: ChangeStatus::Applied,
                    approval_required: false,
                    approved_by: None,
                    approved_at: None,
                    applied_at: Some(Utc::now()),
                    rolled_back_at: None,
                    custom_data: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                audit_trail: vec![],
                related_changes: vec![],
            }];

            let response = PaginatedResponse {
                data: changes,
                total: 1,
                page: query.page as u64,
                per_page: query.per_page as u64,
                total_pages: 1,
                has_next: false,
                has_prev: false,
            };

            Ok(Json(ApiResponse::success(response)))
        }
    }
}

/// Get specific change details by ID
pub async fn get_change_details(
    State(app_state): State<AppState>,
    Path(change_id): Path<String>,
) -> ServerResult<Json<ApiResponse<ChangeHistoryResponse>>> {
    // Try to get change from datastore
    match app_state
        .datastore
        .get_configuration_change(&change_id)
        .await
    {
        Ok(Some(change)) => {
            let response = ChangeHistoryResponse {
                change,
                audit_trail: vec![],     // TODO: Get actual audit trail
                related_changes: vec![], // TODO: Get related changes
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Ok(None) => Err(ServerError::NotFound(format!("Change {}", change_id))),
        Err(_) => {
            // Fallback to placeholder response
            let response = ChangeHistoryResponse {
                change: ConfigurationChange {
                    id: change_id.clone(),
                    change_type: ChangeType::Update,
                    entity_type: "git_sync".to_string(),
                    entity_id: "repository".to_string(),
                    user_id: Some("system".to_string()),
                    source: ChangeSource::GitSync,
                    description: Some("Git repository synchronization".to_string()),
                    old_value: None,
                    new_value: None,
                    diff_content: None,
                    git_commit: Some("latest".to_string()),
                    git_branch: Some("main".to_string()),
                    status: ChangeStatus::Applied,
                    approval_required: false,
                    approved_by: None,
                    approved_at: None,
                    applied_at: Some(Utc::now()),
                    rolled_back_at: None,
                    custom_data: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                audit_trail: vec![],
                related_changes: vec![],
            };

            Ok(Json(ApiResponse::success(response)))
        }
    }
}

/// Get repository information and status
pub async fn get_repository_info(
    State(_app_state): State<AppState>,
) -> ServerResult<Json<ApiResponse<RepositoryInfoResponse>>> {
    // Placeholder response
    let response = RepositoryInfoResponse {
        info: RepositoryInfo {
            url: "https://example.com/repo.git".to_string(),
            local_path: "/tmp/repo".into(),
            current_branch: "main".to_string(),
            remote_name: "origin".to_string(),
            last_sync: None,
            description: Some("Development placeholder".to_string()),
        },
        status: ApiRepositoryStatus {
            current_branch: "main".to_string(),
            is_clean: true,
            commits_ahead: 0,
            commits_behind: 0,
        },
        branches: vec![],
        recent_commits: vec![],
        pending_changes: vec![],
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Handle Git webhook events
pub async fn handle_git_webhook(
    State(_state): State<AppState>,
    Json(payload): Json<GitWebhookPayload>,
) -> ServerResult<Json<ApiResponse<String>>> {
    // TODO: Implement actual webhook handling
    // - Validate webhook signature
    // - Process different event types
    // - Trigger appropriate sync operations
    // - Log webhook events

    let message = match payload.event_type.as_str() {
        "push" => payload.commits.as_ref().map_or_else(
            || "Processed push event".to_string(),
            |commits| format!("Processed push event with {} commits", commits.len()),
        ),
        "pull_request" => "Processed pull request event".to_string(),
        event => format!("Processed {} event", event),
    };

    Ok(Json(ApiResponse::success_with_message(
        "webhook_processed".to_string(),
        message,
    )))
}

/// Get Git webhook configuration
pub async fn get_webhook_config(
    State(_state): State<AppState>,
) -> ServerResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement actual webhook config retrieval
    let config = serde_json::json!({
        "enabled": true,
        "url": "/api/v1/git/webhooks",
        "events": ["push", "pull_request"],
        "secret_configured": true
    });

    Ok(Json(ApiResponse::success(config)))
}

/// Update Git webhook configuration
pub async fn update_webhook_config(
    State(_state): State<AppState>,
    Json(config): Json<serde_json::Value>,
) -> ServerResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement actual webhook config update
    // - Validate configuration
    // - Update webhook settings
    // - Return updated configuration

    Ok(Json(ApiResponse::success_with_message(
        config,
        "Webhook configuration updated".to_string(),
    )))
}
