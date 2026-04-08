use serde::Serialize;
use serde_json::json;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    time::{Duration, SystemTime},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
    time::timeout,
};
use unet_cli::{AppContext, Db};
use unet_core::{
    models::{
        DeviceRole, Lifecycle, Node, Vendor,
        derived::{
            InterfaceAdminStatus, InterfaceOperStatus, InterfaceStats, InterfaceStatus, NodeStatus,
            PerformanceMetrics, SystemInfo,
        },
    },
    snmp::SnmpValue,
};
use uuid::Uuid;

pub struct TestResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

pub fn json_response<T: Serialize>(status: u16, data: T) -> TestResponse {
    TestResponse {
        status,
        content_type: "application/json",
        body: json!({ "data": data }).to_string(),
    }
}

pub fn api_error_response(status: u16, code: &str, message: &str) -> TestResponse {
    TestResponse {
        status,
        content_type: "application/json",
        body: json!({
            "error": message,
            "code": code,
            "success": false,
        })
        .to_string(),
    }
}

pub fn text_response(status: u16, body: &str) -> TestResponse {
    TestResponse {
        status,
        content_type: "text/plain",
        body: body.to_string(),
    }
}

pub async fn spawn_test_server<F>(
    expected_requests: usize,
    handler: F,
) -> (String, oneshot::Receiver<Vec<String>>)
where
    F: Fn(usize, &str) -> TestResponse + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let addr = listener.local_addr().expect("address should exist");
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let mut requests = Vec::with_capacity(expected_requests);

        for index in 0..expected_requests {
            let accept_result = timeout(Duration::from_secs(2), listener.accept()).await;
            let Ok(Ok((mut stream, _))) = accept_result else {
                break;
            };

            let request = read_http_request(&mut stream).await;
            let response = handler(index, &request);
            write_http_response(&mut stream, response)
                .await
                .expect("response should write");
            requests.push(request);
        }

        let _ = tx.send(requests);
    });

    (format!("http://{addr}"), rx)
}

pub fn remote_test_context() -> AppContext {
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

pub fn sample_node(node_id: Uuid, name: &str) -> Node {
    let mut node = Node::new(
        name.to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.id = node_id;
    node.model = "asr-1001".to_string();
    node.lifecycle = Lifecycle::Live;
    node.management_ip = Some("192.0.2.10".parse().expect("ip should parse"));
    node.custom_data = json!({ "rack": "r1" });
    node
}

pub fn sample_status(node_id: Uuid) -> NodeStatus {
    let mut status = NodeStatus::new(node_id);
    status.last_updated = SystemTime::UNIX_EPOCH;
    status.reachable = true;
    status.system_info = Some(SystemInfo {
        description: Some("Cisco router".to_string()),
        object_id: Some("1.3.6.1.4.1.9".to_string()),
        uptime_ticks: Some(123_400),
        contact: Some("noc@example.com".to_string()),
        name: Some("edge-1".to_string()),
        location: Some("SJC1".to_string()),
        services: Some(72),
    });
    status.interfaces = vec![sample_interface()];
    status.performance = Some(sample_metrics());
    status.last_snmp_success = Some(SystemTime::UNIX_EPOCH);
    status.vendor_metrics = HashMap::<String, SnmpValue>::new();
    status.raw_snmp_data = HashMap::<String, SnmpValue>::new();
    status
}

pub fn sample_metrics() -> PerformanceMetrics {
    PerformanceMetrics {
        cpu_utilization: Some(42),
        memory_utilization: Some(73),
        total_memory: Some(32_000),
        used_memory: Some(23_360),
        load_average: Some(1.25),
    }
}

fn sample_interface() -> InterfaceStatus {
    InterfaceStatus {
        index: 1,
        name: "GigabitEthernet0/0".to_string(),
        interface_type: 6,
        mtu: Some(1500),
        speed: Some(1_000_000_000),
        physical_address: Some("00:11:22:33:44:55".to_string()),
        admin_status: InterfaceAdminStatus::Up,
        oper_status: InterfaceOperStatus::Up,
        last_change: Some(100),
        input_stats: InterfaceStats::default(),
        output_stats: InterfaceStats::default(),
    }
}

async fn read_http_request(stream: &mut tokio::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];

    loop {
        let read_result = timeout(Duration::from_secs(2), stream.read(&mut chunk)).await;
        let Ok(Ok(bytes_read)) = read_result else {
            break;
        };
        if bytes_read == 0 {
            break;
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);
        if request_complete(&buffer) {
            break;
        }
    }

    String::from_utf8_lossy(&buffer).into_owned()
}

fn request_complete(buffer: &[u8]) -> bool {
    let Some(headers_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };

    let header_text = String::from_utf8_lossy(&buffer[..headers_end]);
    let content_length = header_text
        .lines()
        .find_map(|line| {
            let lower = line.to_ascii_lowercase();
            lower
                .strip_prefix("content-length:")
                .and_then(|value| value.trim().parse::<usize>().ok())
        })
        .unwrap_or(0);

    buffer.len() >= headers_end + 4 + content_length
}

async fn write_http_response(
    stream: &mut tokio::net::TcpStream,
    response: TestResponse,
) -> anyhow::Result<()> {
    let status_text = match response.status {
        200 => "OK",
        201 => "Created",
        401 => "Unauthorized",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let payload = format!(
        "HTTP/1.1 {} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        response.status,
        status_text,
        response.content_type,
        response.body.len(),
        response.body
    );
    stream.write_all(payload.as_bytes()).await?;
    Ok(())
}
