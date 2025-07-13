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
        .merge(create_location_routes())
        .merge(create_link_routes())
        .merge(create_policy_routes())
}

/// Create node-related routes
fn create_node_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/nodes", get(handlers::nodes::list_nodes))
        .route("/api/v1/nodes", post(handlers::nodes::create_node))
        .route("/api/v1/nodes/:id", get(handlers::nodes::get_node))
        .route("/api/v1/nodes/:id", put(handlers::nodes::update_node))
        .route("/api/v1/nodes/:id", delete(handlers::nodes::delete_node))
        .route(
            "/api/v1/nodes/:id/status",
            get(handlers::nodes::get_node_status),
        )
        .route(
            "/api/v1/nodes/:id/interfaces",
            get(handlers::nodes::get_node_interfaces),
        )
        .route(
            "/api/v1/nodes/:id/metrics",
            get(handlers::nodes::get_node_metrics),
        )
}

/// Create location-related routes
fn create_location_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/locations",
            get(handlers::locations::list_locations),
        )
        .route(
            "/api/v1/locations",
            post(handlers::locations::create_location),
        )
        .route(
            "/api/v1/locations/:id",
            get(handlers::locations::get_location),
        )
        .route(
            "/api/v1/locations/:id",
            put(handlers::locations::update_location),
        )
        .route(
            "/api/v1/locations/:id",
            delete(handlers::locations::delete_location),
        )
}

/// Create link-related routes
fn create_link_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/links", get(handlers::links::list_links))
        .route("/api/v1/links", post(handlers::links::create_link))
        .route("/api/v1/links/:id", get(handlers::links::get_link))
        .route("/api/v1/links/:id", put(handlers::links::update_link))
        .route("/api/v1/links/:id", delete(handlers::links::delete_link))
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
