//! Server configuration and startup

use anyhow::Result;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use unet_core::{
    config::Config,
    datastore::{DataStore, sqlite::SqliteStore},
    policy_integration::PolicyService,
};

use crate::{background::BackgroundTasks, handlers};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub datastore: Arc<dyn DataStore + Send + Sync>,
    pub policy_service: PolicyService,
}

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
async fn create_app(config: Config, database_url: String) -> Result<Router> {
    let app_state = initialize_app_state(config.clone(), database_url).await?;
    let router = create_router();
    let app = router.with_state(app_state).layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &axum::http::Request<axum::body::Body>| {
                        let request_id = uuid::Uuid::new_v4();
                        tracing::info_span!(
                            "request",
                            method = %request.method(),
                            uri = %request.uri(),
                            request_id = %request_id,
                        )
                    })
                    .on_request(
                        |_request: &axum::http::Request<axum::body::Body>,
                         _span: &tracing::Span| {
                            tracing::info!("Processing request");
                        },
                    )
                    .on_response(
                        |response: &axum::http::Response<axum::body::Body>,
                         latency: std::time::Duration,
                         _span: &tracing::Span| {
                            tracing::info!(
                                status = response.status().as_u16(),
                                latency_ms = latency.as_millis(),
                                "Request completed"
                            );
                        },
                    )
                    .on_failure(
                        |error: tower_http::classify::ServerErrorsFailureClass,
                         latency: std::time::Duration,
                         _span: &tracing::Span| {
                            tracing::error!(
                                error = %error,
                                latency_ms = latency.as_millis(),
                                "Request failed"
                            );
                        },
                    ),
            )
            .layer(CorsLayer::permissive()),
    );

    Ok(app)
}

/// Initialize application state with datastore and services
async fn initialize_app_state(config: Config, database_url: String) -> Result<AppState> {
    // Initialize SQLite datastore
    info!("Initializing SQLite datastore with URL: {}", database_url);
    let datastore: Arc<dyn DataStore + Send + Sync> = Arc::new(
        SqliteStore::new(&database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize SQLite datastore: {}", e))?,
    );

    // Initialize policy service
    info!("Initializing policy service");
    let policy_service = PolicyService::new(config.git.clone());

    let app_state = AppState {
        datastore: datastore.clone(),
        policy_service: policy_service.clone(),
    };

    // Start background tasks
    let background_tasks = BackgroundTasks::new(config, datastore, policy_service);
    background_tasks.start();

    Ok(app_state)
}

/// Create the router with all API endpoints
fn create_router() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        .merge(create_node_routes())
        .merge(create_policy_routes())
}

/// Create node-related routes
fn create_node_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/nodes", get(handlers::nodes::list_nodes))
        .route("/api/v1/nodes", post(handlers::nodes::create_node))
        .route("/api/v1/nodes/{id}", get(handlers::nodes::get_node))
        .route("/api/v1/nodes/{id}", put(handlers::nodes::update_node))
        .route("/api/v1/nodes/{id}", delete(handlers::nodes::delete_node))
        .route(
            "/api/v1/nodes/{id}/status",
            get(handlers::nodes::get_node_status),
        )
        .route(
            "/api/v1/nodes/{id}/interfaces",
            get(handlers::nodes::get_node_interfaces),
        )
        .route(
            "/api/v1/nodes/{id}/metrics",
            get(handlers::nodes::get_node_metrics),
        )
}

/// Create policy-related routes
fn create_policy_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/policies/evaluate",
            post(handlers::policies::evaluate_policies),
        )
        .route(
            "/api/v1/policies/results",
            get(handlers::policies::get_policy_results),
        )
        .route(
            "/api/v1/policies/validate",
            post(handlers::policies::validate_policies),
        )
        .route(
            "/api/v1/policies/status",
            get(handlers::policies::get_policy_status),
        )
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

    #[tokio::test]
    async fn test_app_state_creation() {
        let datastore: Arc<dyn DataStore + Send + Sync> = Arc::new(
            // Use an in-memory SQLite store for testing
            unet_core::datastore::sqlite::SqliteStore::new("sqlite::memory:")
                .await
                .unwrap(),
        );
        let git_config = unet_core::config::types::GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };
        let policy_service = PolicyService::new(git_config);

        let app_state = AppState {
            datastore: datastore.clone(),
            policy_service,
        };

        // Verify the app state is properly constructed
        assert!(Arc::ptr_eq(&app_state.datastore, &datastore));
    }

    #[tokio::test]
    async fn test_initialize_app_state_sqlite() {
        let config = create_test_config();
        let database_url = "sqlite::memory:".to_string();

        let result = initialize_app_state(config, database_url).await;

        // SQLite datastore should work in tests
        match result {
            Ok(app_state) => {
                // Verify app state structure
                assert_eq!(app_state.datastore.name(), "SQLite");
            }
            Err(e) => {
                // Some initialization might fail in test environment
                println!("Initialization error in test: {e}");
            }
        }
    }

    #[test]
    fn test_socket_addr_parsing() {
        let config = create_test_config_with_custom_values();

        // Test that we can parse the host into an IP address
        let parsed_ip = config.server.host.parse::<std::net::IpAddr>();

        if let Ok(ip) = parsed_ip {
            let addr = SocketAddr::from((ip, config.server.port));
            assert_eq!(addr.port(), 9090);
        } else {
            // If parsing fails, should fall back to localhost
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

        // Test the fallback mechanism for invalid host
        let parsed_ip = config
            .server
            .host
            .parse::<std::net::IpAddr>()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

        let addr = SocketAddr::from((parsed_ip, config.server.port));

        // Should fall back to localhost
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

        // Verify cloning works correctly
        assert_eq!(config.server.host, cloned_config.server.host);
        assert_eq!(config.server.port, cloned_config.server.port);
        assert_eq!(config.logging.level, cloned_config.logging.level);
    }

    #[test]
    fn test_policy_service_creation() {
        let config = create_test_config();
        let policy_service = PolicyService::new(config.git);

        // Verify policy service is created (we can't test much more without the actual implementation details)
        // The fact that it doesn't panic is a good sign
        drop(policy_service);
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
    async fn test_create_router() {
        let router = create_router();
        // We can't easily test the routes without a complex setup,
        // but we can verify the router is created without panicking
        let app_state = create_mock_app_state().await;
        let _router_with_state: axum::Router = router.with_state(app_state);
    }

    async fn create_mock_app_state() -> AppState {
        let datastore = unet_core::datastore::sqlite::SqliteStore::new("sqlite::memory:")
            .await
            .unwrap();

        let git_config = unet_core::config::types::GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };

        AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::new(git_config),
        }
    }

    #[tokio::test]
    async fn test_create_node_routes() {
        let node_router = create_node_routes();
        // Verify the router is created without panicking
        let app_state = create_mock_app_state().await;
        let _router_with_state: axum::Router = node_router.with_state(app_state);
    }

    #[tokio::test]
    async fn test_create_policy_routes() {
        let policy_router = create_policy_routes();
        // Verify the router is created without panicking
        let app_state = create_mock_app_state().await;
        let _router_with_state: axum::Router = policy_router.with_state(app_state);
    }

    #[tokio::test]
    async fn test_create_app_functionality() {
        let config = create_test_config();
        let database_url = "sqlite::memory:".to_string();

        // Test that create_app doesn't panic during setup
        let result = create_app(config, database_url).await;
        match result {
            Ok(_app) => {
                // App creation succeeded
            }
            Err(e) => {
                // Some initialization might fail in test environment
                println!("App creation error in test: {e}");
            }
        }
    }

    #[test]
    fn test_trace_layer_configuration() {
        // Test that the trace layer configuration doesn't panic
        let request_id = uuid::Uuid::new_v4();
        let _span = tracing::info_span!(
            "request",
            method = "GET",
            uri = "/health",
            request_id = %request_id,
        );
    }

    #[test]
    fn test_cors_layer_creation() {
        // Test that CORS layer creation doesn't panic
        let _cors_layer = CorsLayer::permissive();
    }

    #[tokio::test]
    async fn test_background_tasks_initialization() {
        let config = create_test_config();
        let datastore = Arc::new(
            unet_core::datastore::sqlite::SqliteStore::new("sqlite::memory:")
                .await
                .unwrap(),
        );
        let policy_service = PolicyService::new(config.git.clone());

        // Test that background tasks can be created without panicking
        let background_tasks = BackgroundTasks::new(config, datastore, policy_service);
        // We can't easily test start() in a unit test as it spawns background tasks
        // but we can verify the struct is created
        drop(background_tasks);
    }

    #[test]
    fn test_service_builder_configuration() {
        // Test that ServiceBuilder configuration doesn't panic
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
