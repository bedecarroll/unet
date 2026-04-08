//! Authentication integration tests for the server router.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tempfile::NamedTempFile;
use tower::ServiceExt as _;
use unet_core::config::Config;

use super::middleware::create_app;

const PROTECTED_PATH: &str = "/api/v1/policies/results";

async fn request_status(
    config: Config,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, Option<Value>) {
    let app = create_app(config, "sqlite::memory:".to_string())
        .await
        .expect("app should build");

    let mut builder = Request::builder().uri(path);
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should succeed");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");

    let json = if bytes.is_empty() {
        None
    } else {
        Some(serde_json::from_slice(&bytes).expect("body should be valid json"))
    };

    (status, json)
}

fn auth_config(enabled: bool) -> Config {
    let temp_file = NamedTempFile::with_suffix(".toml").expect("temp file should exist");
    std::fs::write(
        temp_file.path(),
        format!(
            r#"
[database]
url = "sqlite://test.db"
max_connections = 10
timeout = 30

[logging]
level = "info"
format = "text"

[snmp]
community = "public"
timeout = 5
retries = 3

[server]
host = "127.0.0.1"
port = 8080
max_request_size = 1048576

[git]
branch = "main"
sync_interval = 300

[domain]
search_domains = []

[auth]
enabled = {enabled}
token = "bed-24-secret"
token_expiry = 3600
"#
        ),
    )
    .expect("config should write");

    Config::from_file(temp_file.path()).expect("config should load")
}

#[tokio::test]
async fn test_health_route_remains_public_when_auth_enabled() {
    let (status, _) = request_status(auth_config(true), "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_protected_route_rejects_missing_bearer_token() {
    let (status, body) = request_status(auth_config(true), PROTECTED_PATH, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let body = body.expect("unauthorized response should be json");
    assert_eq!(body["success"], false);
    assert_eq!(body["code"], "AUTH_REQUIRED");
}

#[tokio::test]
async fn test_protected_route_rejects_invalid_bearer_token() {
    let (status, body) =
        request_status(auth_config(true), PROTECTED_PATH, Some("wrong-token")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let body = body.expect("unauthorized response should be json");
    assert_eq!(body["success"], false);
    assert_eq!(body["code"], "INVALID_AUTH_TOKEN");
}

#[tokio::test]
async fn test_protected_route_accepts_valid_bearer_token() {
    let (status, _) =
        request_status(auth_config(true), PROTECTED_PATH, Some("bed-24-secret")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_protected_route_remains_open_when_auth_disabled() {
    let (status, _) = request_status(auth_config(false), PROTECTED_PATH, None).await;
    assert_eq!(status, StatusCode::OK);
}
