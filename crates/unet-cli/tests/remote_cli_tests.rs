//! Remote CLI integration tests.

use clap::Parser;
use std::future::Future;
use std::pin::Pin;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
    time::{Duration, timeout},
};
use unet_cli::{AppContext, Cli, Db};

async fn spawn_list_nodes_server() -> (String, oneshot::Receiver<Option<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let addr = listener.local_addr().expect("address should exist");
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let accept_result = timeout(Duration::from_secs(2), listener.accept()).await;
        let Ok(Ok((mut stream, _))) = accept_result else {
            let _ = tx.send(None);
            return;
        };

        let mut buffer = vec![0_u8; 4096];
        let bytes_read = stream.read(&mut buffer).await.expect("request should read");
        let request = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
        let request_lower = request.to_lowercase();
        let authorized = request_lower.contains("authorization: bearer bed-24-secret");

        let body = if authorized {
            r#"{"success":true,"message":null,"data":{"data":[],"total":0,"page":1,"per_page":20,"total_pages":0,"has_next":false,"has_prev":false}}"#
        } else {
            r#"{"error":"Missing bearer token","code":"AUTH_REQUIRED","success":false}"#
        };
        let status = if authorized {
            "200 OK"
        } else {
            "401 Unauthorized"
        };
        let response = format!(
            "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .await
            .expect("response should write");

        let _ = tx.send(Some(request));
    });

    (format!("http://{addr}"), rx)
}

fn remote_test_context() -> AppContext {
    let connect = Box::new(|_url: &str| {
        Box::pin(async {
            Err::<Db, anyhow::Error>(anyhow::anyhow!(
                "local database connect should not run in remote mode"
            ))
        }) as Pin<Box<dyn Future<Output = anyhow::Result<Db>> + Send>>
    });
    let migrate = Box::new(|_db: &Db| {
        Box::pin(async { Ok::<(), anyhow::Error>(()) })
            as Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
    });

    AppContext { connect, migrate }
}

#[tokio::test]
async fn test_run_with_remote_nodes_list_uses_bearer_token() {
    let (server_url, request_rx) = spawn_list_nodes_server().await;
    let cli = Cli::parse_from([
        "unet",
        "--server",
        &server_url,
        "--token",
        "bed-24-secret",
        "--output",
        "json",
        "nodes",
        "list",
    ]);

    let result = unet_cli::run_with(remote_test_context(), cli).await;
    assert!(result.is_ok());

    let request = request_rx
        .await
        .expect("request channel should complete")
        .expect("remote server should receive a request");
    let request_lower = request.to_lowercase();
    assert!(request.starts_with("GET /api/v1/nodes"));
    assert!(request_lower.contains("authorization: bearer bed-24-secret"));
}

#[tokio::test]
async fn test_run_with_remote_nodes_list_surfaces_unauthorized_response() {
    let (server_url, request_rx) = spawn_list_nodes_server().await;
    let cli = Cli::parse_from([
        "unet",
        "--server",
        &server_url,
        "--output",
        "json",
        "nodes",
        "list",
    ]);

    let result = unet_cli::run_with(remote_test_context(), cli).await;
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("AUTH_REQUIRED"));

    let request = request_rx
        .await
        .expect("request channel should complete")
        .expect("remote server should receive a request");
    let request_lower = request.to_lowercase();
    assert!(!request_lower.contains("authorization: bearer"));
}
