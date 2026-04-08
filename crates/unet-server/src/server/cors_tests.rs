//! CORS integration tests for the server router.

use axum::{
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode},
};
use tempfile::NamedTempFile;
use tower::ServiceExt as _;
use unet_core::config::Config;

use super::middleware::create_app;

async fn request_headers(
    config: Config,
    method: Method,
    path: &str,
    origin: &str,
    requested_method: Option<&str>,
    requested_headers: Option<&str>,
) -> (StatusCode, HeaderMap) {
    let app = create_app(config, "sqlite::memory:".to_string())
        .await
        .expect("app should build");

    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("Origin", origin);

    if let Some(value) = requested_method {
        builder = builder.header("Access-Control-Request-Method", value);
    }

    if let Some(value) = requested_headers {
        builder = builder.header("Access-Control-Request-Headers", value);
    }

    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should succeed");

    (response.status(), response.headers().clone())
}

fn config_from_file(server_fields: &str) -> Config {
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

[server]
host = "127.0.0.1"
port = 8080
max_request_size = 1048576
{server_fields}

[snmp]
community = "public"
timeout = 5
retries = 3

[git]
branch = "main"
sync_interval = 300

[domain]
search_domains = []

[auth]
enabled = false
"#
        ),
    )
    .expect("config should write");

    Config::from_file(temp_file.path()).expect("config should load")
}

#[tokio::test]
async fn test_default_cors_rejects_non_local_origin() {
    let (status, headers) = request_headers(
        Config::default(),
        Method::GET,
        "/health",
        "https://dashboard.example.com",
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(headers.get("access-control-allow-origin").is_none());
}

#[tokio::test]
async fn test_default_cors_allows_local_development_origin() {
    let (status, headers) = request_headers(
        Config::default(),
        Method::GET,
        "/health",
        "http://localhost:3000",
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers
            .get("access-control-allow-origin")
            .expect("local origin should be allowed"),
        "http://localhost:3000"
    );
}

#[tokio::test]
async fn test_configured_cors_returns_only_explicit_origin_method_and_headers() {
    let config = config_from_file(
        r#"
cors_origins = ["https://dashboard.corp.local"]
cors_methods = ["GET", "POST"]
cors_headers = ["authorization", "content-type"]
"#,
    );

    let (status, headers) = request_headers(
        config,
        Method::OPTIONS,
        "/api/v1/policies/status",
        "https://dashboard.corp.local",
        Some("POST"),
        Some("authorization,content-type"),
    )
    .await;

    assert!(status.is_success());
    assert_eq!(
        headers
            .get("access-control-allow-origin")
            .expect("configured origin should be allowed"),
        "https://dashboard.corp.local"
    );

    let methods = headers
        .get("access-control-allow-methods")
        .expect("allow methods header should exist")
        .to_str()
        .expect("allow methods header should be valid");
    assert!(methods.contains("GET"));
    assert!(methods.contains("POST"));

    let allowed_headers = headers
        .get("access-control-allow-headers")
        .expect("allow headers header should exist")
        .to_str()
        .expect("allow headers header should be valid");
    assert!(allowed_headers.contains("authorization"));
    assert!(allowed_headers.contains("content-type"));
}

#[tokio::test]
async fn test_configured_cors_rejects_origin_not_in_allowlist() {
    let config = config_from_file(
        r#"
cors_origins = ["https://dashboard.corp.local"]
cors_methods = ["GET"]
cors_headers = ["authorization"]
"#,
    );

    let (_status, headers) = request_headers(
        config,
        Method::GET,
        "/health",
        "https://evil.corp.local",
        None,
        None,
    )
    .await;

    assert!(headers.get("access-control-allow-origin").is_none());
}
