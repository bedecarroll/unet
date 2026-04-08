//! Remote CLI error-path coverage tests.

mod support;

use clap::Parser;
use support::{remote_test_context, spawn_test_server, text_response};
use unet_cli::Cli;
use uuid::Uuid;

async fn run_remote(server_url: &str, args: &[String]) -> anyhow::Result<()> {
    let mut cli_args = vec![
        "unet".to_string(),
        "--server".to_string(),
        server_url.to_string(),
        "--output".to_string(),
        "json".to_string(),
    ];
    cli_args.extend_from_slice(args);

    let cli = Cli::parse_from(cli_args);
    unet_cli::run_with(remote_test_context(), cli).await
}

#[tokio::test]
async fn test_run_with_remote_plain_text_api_error_is_reported() {
    let (server_url, _requests_rx) =
        spawn_test_server(1, |_, _| text_response(500, "server exploded")).await;
    let list_args = vec!["nodes".to_string(), "list".to_string()];
    let error = run_remote(&server_url, &list_args)
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("HTTP_500"));
    assert!(error.contains("server exploded"));
}

#[tokio::test]
async fn test_run_with_remote_rejects_unsupported_node_commands() {
    let (server_url, _requests_rx) = spawn_test_server(0, |_, _| unreachable!()).await;
    let compare_args = vec![
        "nodes",
        "compare",
        "--node-a",
        &Uuid::new_v4().to_string(),
        "--node-b",
        &Uuid::new_v4().to_string(),
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    let compare_error = run_remote(&server_url, &compare_args)
        .await
        .unwrap_err()
        .to_string();
    assert!(compare_error.contains("does not support compare"));
}

#[tokio::test]
async fn test_run_with_remote_rejects_non_node_commands() {
    let (server_url, _requests_rx) = spawn_test_server(0, |_, _| unreachable!()).await;
    let vendors_args = vec!["vendors".to_string(), "list".to_string()];
    let vendors_error = run_remote(&server_url, &vendors_args)
        .await
        .unwrap_err()
        .to_string();
    assert!(vendors_error.contains("currently supports node commands"));
}

#[tokio::test]
async fn test_run_with_remote_add_rejects_invalid_vendor() {
    let (server_url, _requests_rx) = spawn_test_server(0, |_, _| unreachable!()).await;
    let invalid_vendor_args = vec![
        "nodes",
        "add",
        "--name",
        "edge-1",
        "--domain",
        "example.com",
        "--vendor",
        "unknown-vendor",
        "--model",
        "asr-1001",
        "--role",
        "router",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    let vendor_error = run_remote(&server_url, &invalid_vendor_args)
        .await
        .unwrap_err()
        .to_string();
    assert!(vendor_error.contains("Invalid vendor"));
}

#[tokio::test]
async fn test_run_with_remote_add_rejects_invalid_json() {
    let (server_url, _requests_rx) = spawn_test_server(0, |_, _| unreachable!()).await;
    let invalid_json_args = vec![
        "nodes",
        "add",
        "--name",
        "edge-1",
        "--domain",
        "example.com",
        "--vendor",
        "cisco",
        "--model",
        "asr-1001",
        "--role",
        "router",
        "--custom-data",
        "{not-json}",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    let json_error = run_remote(&server_url, &invalid_json_args)
        .await
        .unwrap_err()
        .to_string();
    assert!(json_error.contains("key must be a string"));
}
