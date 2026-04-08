//! Remote CLI command coverage tests.

mod support;

use clap::Parser;
use serde_json::json;
use support::{json_response, remote_test_context, sample_node, sample_status, spawn_test_server};
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
async fn test_run_with_remote_nodes_list_forwards_filters() {
    let node = sample_node(Uuid::new_v4(), "edge-1");
    let page = json!({
        "data": [serde_json::to_value(&node).unwrap()],
        "total": 1,
        "page": 2,
        "per_page": 5,
        "total_pages": 1,
        "has_next": false,
        "has_prev": true
    });
    let (server_url, requests_rx) =
        spawn_test_server(1, move |_, _| json_response(200, page.clone())).await;
    let args = vec![
        "nodes",
        "list",
        "--vendor",
        "cisco",
        "--role",
        "router",
        "--lifecycle",
        "live",
        "--page",
        "2",
        "--per-page",
        "5",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    assert!(run_remote(&server_url, &args).await.is_ok());

    let request = requests_rx.await.unwrap().remove(0);
    assert!(request.contains("vendor=cisco"));
    assert!(request.contains("role=router"));
    assert!(request.contains("lifecycle=live"));
    assert!(request.contains("page=2"));
    assert!(request.contains("per_page=5"));
}

#[tokio::test]
async fn test_run_with_remote_nodes_add_posts_serialized_payload() {
    let node = sample_node(Uuid::new_v4(), "edge-1");
    let (server_url, requests_rx) = spawn_test_server(1, move |_, _| {
        json_response(201, serde_json::to_value(&node).unwrap())
    })
    .await;
    let args = vec![
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
        "--lifecycle",
        "live",
        "--custom-data",
        r#"{"rack":"r1"}"#,
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    assert!(run_remote(&server_url, &args).await.is_ok());

    let request = requests_rx.await.unwrap().remove(0);
    assert!(request.starts_with("POST /api/v1/nodes "));
    assert!(request.contains(r#""name":"edge-1""#));
    assert!(request.contains(r#""vendor":"cisco""#));
    assert!(request.contains(r#""custom_data":{"rack":"r1"}"#));
}

#[tokio::test]
async fn test_run_with_remote_nodes_update_puts_serialized_payload() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let (server_url, requests_rx) = spawn_test_server(1, move |_, _| {
        json_response(200, serde_json::to_value(&node).unwrap())
    })
    .await;
    let args = vec![
        "nodes".to_string(),
        "update".to_string(),
        node_id.to_string(),
        "--name".to_string(),
        "edge-2".to_string(),
        "--custom-data".to_string(),
        r#"{"rack":"r2"}"#.to_string(),
    ];

    assert!(run_remote(&server_url, &args).await.is_ok());

    let request = requests_rx.await.unwrap().remove(0);
    assert!(request.starts_with(&format!("PUT /api/v1/nodes/{node_id} ")));
    assert!(request.contains(r#""name":"edge-2""#));
    assert!(request.contains(r#""custom_data":{"rack":"r2"}"#));
}

#[tokio::test]
async fn test_run_with_remote_nodes_delete_with_yes_fetches_then_deletes() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let (server_url, requests_rx) = spawn_test_server(2, move |index, _| {
        if index == 0 {
            json_response(200, serde_json::to_value(&node).unwrap())
        } else {
            json_response(200, serde_json::Value::Null)
        }
    })
    .await;
    let args = vec![
        "nodes".to_string(),
        "delete".to_string(),
        node_id.to_string(),
        "--yes".to_string(),
    ];

    assert!(run_remote(&server_url, &args).await.is_ok());

    let requests = requests_rx.await.unwrap();
    assert!(requests[0].starts_with(&format!("GET /api/v1/nodes/{node_id} ")));
    assert!(requests[1].starts_with(&format!("DELETE /api/v1/nodes/{node_id} ")));
}

#[tokio::test]
async fn test_run_with_remote_nodes_show_fetches_derived_state_views() {
    let node_id = Uuid::new_v4();
    let node = sample_node(node_id, "edge-1");
    let status = sample_status(node_id);
    let (server_url, requests_rx) = spawn_test_server(4, move |_, request| {
        if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/status ")) {
            json_response(200, serde_json::to_value(&status).unwrap())
        } else if request.starts_with(&format!("GET /api/v1/nodes/{node_id}/interfaces ")) {
            json_response(200, serde_json::to_value(&status.interfaces).unwrap())
        } else {
            json_response(200, serde_json::to_value(&node).unwrap())
        }
    })
    .await;
    let args = vec![
        "nodes".to_string(),
        "show".to_string(),
        node_id.to_string(),
        "--include-status".to_string(),
        "--show-interfaces".to_string(),
        "--show-system-info".to_string(),
    ];

    assert!(run_remote(&server_url, &args).await.is_ok());
    assert_eq!(requests_rx.await.unwrap().len(), 4);
}
