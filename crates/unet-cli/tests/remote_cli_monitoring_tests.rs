//! Remote CLI monitoring coverage tests.

mod support;

use clap::Parser;
use support::{
    api_error_response, json_response, remote_test_context, sample_metrics, sample_node,
    sample_status, spawn_test_server,
};
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
async fn test_run_with_remote_nodes_status_supports_basic_interfaces_system_and_polling() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let status = sample_status(node_id);

    let (status_url, status_rx) = spawn_test_server(3, move |_, request| {
        if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/interfaces ")) {
            json_response(200, serde_json::to_value(&status.interfaces).unwrap())
        } else if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/status ")) {
            json_response(200, serde_json::to_value(&status).unwrap())
        } else {
            json_response(200, serde_json::to_value(&node).unwrap())
        }
    })
    .await;
    let status_args = vec![
        "nodes",
        "status",
        &node_id.to_string(),
        "--status-type",
        "basic",
        "--status-type",
        "interfaces",
        "--status-type",
        "system",
        "--status-type",
        "polling",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    assert!(run_remote(&status_url, &status_args).await.is_ok());
    assert_eq!(status_rx.await.unwrap().len(), 3);
}

#[tokio::test]
async fn test_run_with_remote_nodes_status_all_fetches_status_and_interfaces() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let status = sample_status(node_id);
    let (all_url, all_rx) = spawn_test_server(3, move |_, request| {
        if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/interfaces ")) {
            json_response(200, serde_json::to_value(&status.interfaces).unwrap())
        } else if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/status ")) {
            json_response(200, serde_json::to_value(&status).unwrap())
        } else {
            json_response(200, serde_json::to_value(&node).unwrap())
        }
    })
    .await;
    let all_args = vec![
        "nodes",
        "status",
        &node_id.to_string(),
        "--status-type",
        "all",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    assert!(run_remote(&all_url, &all_args).await.is_ok());
    assert_eq!(all_rx.await.unwrap().len(), 3);
}

#[tokio::test]
async fn test_run_with_remote_nodes_metrics_detailed_fetches_interfaces() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let status = sample_status(node_id);
    let metrics = sample_metrics();
    let (metrics_url, metrics_rx) = spawn_test_server(3, move |_, request| {
        if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/interfaces ")) {
            json_response(200, serde_json::to_value(&status.interfaces).unwrap())
        } else if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/metrics ")) {
            json_response(200, serde_json::to_value(&metrics).unwrap())
        } else {
            json_response(200, serde_json::to_value(&node).unwrap())
        }
    })
    .await;
    let detailed_args = vec!["nodes", "metrics", &node_id.to_string(), "--detailed"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert!(run_remote(&metrics_url, &detailed_args).await.is_ok());
    assert_eq!(metrics_rx.await.unwrap().len(), 3);
}

#[tokio::test]
async fn test_run_with_remote_nodes_metrics_history_short_circuits_after_node_lookup() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let (history_url, history_rx) = spawn_test_server(1, move |_, _| {
        json_response(200, serde_json::to_value(&node).unwrap())
    })
    .await;
    let history_args = vec!["nodes", "metrics", &node_id.to_string(), "--history"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert!(run_remote(&history_url, &history_args).await.is_ok());
    assert_eq!(history_rx.await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_run_with_remote_nodes_metrics_not_found_returns_placeholder_payload() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let (none_url, none_rx) = spawn_test_server(2, move |index, _| {
        if index == 0 {
            json_response(200, serde_json::to_value(&node).unwrap())
        } else {
            api_error_response(404, "NOT_FOUND", "No metrics")
        }
    })
    .await;
    let args = vec![
        "nodes".to_string(),
        "metrics".to_string(),
        node_id.to_string(),
    ];

    assert!(run_remote(&none_url, &args).await.is_ok());
    assert_eq!(none_rx.await.unwrap().len(), 2);
}

#[tokio::test]
async fn test_run_with_remote_nodes_metrics_errors_are_reported_in_output() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let (error_url, error_rx) = spawn_test_server(2, move |index, _| {
        if index == 0 {
            json_response(200, serde_json::to_value(&node).unwrap())
        } else {
            api_error_response(500, "POLL_FAILED", "Polling failed")
        }
    })
    .await;
    let args = vec![
        "nodes".to_string(),
        "metrics".to_string(),
        node_id.to_string(),
    ];

    assert!(run_remote(&error_url, &args).await.is_ok());
    assert_eq!(error_rx.await.unwrap().len(), 2);
}
