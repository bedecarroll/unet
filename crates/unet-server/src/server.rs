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
    let background_tasks = BackgroundTasks::new(config.clone(), datastore, policy_service);
    background_tasks.start().await;

    let app = Router::new()
        // Node endpoints
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
        // Location endpoints
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
        // Link endpoints
        .route("/api/v1/links", get(handlers::links::list_links))
        .route("/api/v1/links", post(handlers::links::create_link))
        .route("/api/v1/links/:id", get(handlers::links::get_link))
        .route("/api/v1/links/:id", put(handlers::links::update_link))
        .route("/api/v1/links/:id", delete(handlers::links::delete_link))
        // Policy endpoints
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
        // Template endpoints
        .route(
            "/api/v1/templates",
            get(handlers::templates::list_templates),
        )
        .route(
            "/api/v1/templates",
            post(handlers::templates::create_template),
        )
        .route(
            "/api/v1/templates/:id",
            get(handlers::templates::get_template),
        )
        .route(
            "/api/v1/templates/:id",
            put(handlers::templates::update_template),
        )
        .route(
            "/api/v1/templates/:id",
            delete(handlers::templates::delete_template),
        )
        .route(
            "/api/v1/templates/render",
            post(handlers::templates::render_template),
        )
        .route(
            "/api/v1/templates/:id/validate",
            post(handlers::templates::validate_template),
        )
        .route(
            "/api/v1/templates/:id/usage",
            get(handlers::templates::get_template_usage),
        )
        // Template assignment endpoints
        .route(
            "/api/v1/template-assignments",
            post(handlers::templates::create_template_assignment),
        )
        .route(
            "/api/v1/template-assignments/:id",
            put(handlers::templates::update_template_assignment),
        )
        .route(
            "/api/v1/template-assignments/:id",
            delete(handlers::templates::delete_template_assignment),
        )
        .route(
            "/api/v1/nodes/:id/template-assignments",
            get(handlers::templates::get_template_assignments_for_node),
        )
        .route(
            "/api/v1/templates/:id/assignments",
            get(handlers::templates::get_template_assignments_for_template),
        )
        // Health check
        .route("/health", get(handlers::health::health_check))
        // Add application state
        .with_state(app_state)
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
                            // Add request ID for tracking
                            let request_id = uuid::Uuid::new_v4();
                            tracing::info_span!(
                                "request",
                                method = %request.method(),
                                uri = %request.uri(),
                                request_id = %request_id,
                            )
                        })
                        .on_request(|_request: &axum::http::Request<_>, _span: &tracing::Span| {
                            tracing::info!("Processing request");
                        })
                        .on_response(
                            |response: &axum::http::Response<_>,
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
