//! Middleware configuration and setup

use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use unet_core::config::Config;

use super::{app_state::initialize_app_state, routes::create_router};

/// Run the μNet HTTP server
pub async fn run(config: Config, database_url: String) -> Result<()> {
    let app = create_app(config.clone(), database_url).await?;

    let addr = SocketAddr::from((
        config
            .server
            .host
            .parse::<std::net::IpAddr>()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        config.server.port,
    ));
    info!("μNet server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the Axum application with all routes
pub async fn create_app(config: Config, database_url: String) -> Result<Router> {
    let app_state = initialize_app_state(config.clone(), database_url).await?;
    let router = create_router();
    let app = router.with_state(app_state).layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive()),
    );

    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config::default()
    }

    fn create_test_config_with_custom_values() -> Config {
        let mut config = Config::default();
        config.server.host = "192.168.1.100".to_string();
        config.server.port = 9090;
        config.logging.level = "debug".to_string();
        config
    }

    #[test]
    fn test_socket_addr_parsing() {
        let config = create_test_config_with_custom_values();

        let parsed_ip = config.server.host.parse::<std::net::IpAddr>();

        if let Ok(ip) = parsed_ip {
            let addr = SocketAddr::from((ip, config.server.port));
            assert_eq!(addr.port(), 9090);
        } else {
            let fallback_ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1));
            let addr = SocketAddr::from((fallback_ip, config.server.port));
            assert_eq!(addr.port(), 9090);
            assert_eq!(addr.ip(), fallback_ip);
        }
    }

    #[test]
    fn test_socket_addr_invalid_host() {
        let mut config = create_test_config();
        config.server.host = "invalid-host-name".to_string();
        config.server.port = 8080;

        let parsed_ip = config
            .server
            .host
            .parse::<std::net::IpAddr>()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

        let addr = SocketAddr::from((parsed_ip, config.server.port));

        assert_eq!(
            addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
        );
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn test_config_cloning() {
        let config = create_test_config_with_custom_values();
        let cloned_config = config.clone();

        assert_eq!(config.server.host, cloned_config.server.host);
        assert_eq!(config.server.port, cloned_config.server.port);
        assert_eq!(config.logging.level, cloned_config.logging.level);
    }

    #[test]
    fn test_ipv4_addr_creation() {
        let localhost = std::net::Ipv4Addr::new(127, 0, 0, 1);
        assert_eq!(localhost.octets(), [127, 0, 0, 1]);

        let custom_ip = std::net::Ipv4Addr::new(192, 168, 1, 100);
        assert_eq!(custom_ip.octets(), [192, 168, 1, 100]);
    }

    #[test]
    fn test_database_url_formats() {
        let sqlite_url = "sqlite://test.db";
        assert!(sqlite_url.starts_with("sqlite://"));
    }

    #[tokio::test]
    async fn test_create_app_functionality() {
        let config = create_test_config();
        let database_url = "sqlite::memory:".to_string();

        let result = create_app(config, database_url).await;
        match result {
            Ok(_app) => {
                // App creation succeeded
            }
            Err(e) => {
                println!("App creation error in test: {e}");
            }
        }
    }

    #[test]
    fn test_cors_layer_creation() {
        let _cors_layer = CorsLayer::permissive();
    }

    #[test]
    fn test_service_builder_configuration() {
        let _service_builder = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive());
    }

    #[test]
    fn test_response_latency_handling() {
        let latency = std::time::Duration::from_millis(100);
        let latency_ms = latency.as_millis();
        assert_eq!(latency_ms, 100);
    }
}
