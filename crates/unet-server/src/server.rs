//! Server configuration and startup

use anyhow::Result;
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::handlers;

/// Run the μNet HTTP server
pub async fn run() -> Result<()> {
    let app = create_app().await?;
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("μNet server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create the Axum application with all routes
async fn create_app() -> Result<Router> {
    let app = Router::new()
        // Node endpoints
        .route("/api/v1/nodes", get(handlers::nodes::list_nodes))
        .route("/api/v1/nodes", post(handlers::nodes::create_node))
        .route("/api/v1/nodes/:id", get(handlers::nodes::get_node))
        .route("/api/v1/nodes/:id", put(handlers::nodes::update_node))
        .route("/api/v1/nodes/:id", delete(handlers::nodes::delete_node))
        .route("/api/v1/nodes/:id/status", get(handlers::nodes::get_node_status))
        
        // Location endpoints
        .route("/api/v1/locations", get(handlers::locations::list_locations))
        .route("/api/v1/locations", post(handlers::locations::create_location))
        .route("/api/v1/locations/:id", get(handlers::locations::get_location))
        .route("/api/v1/locations/:id", put(handlers::locations::update_location))
        .route("/api/v1/locations/:id", delete(handlers::locations::delete_location))
        
        // Link endpoints
        .route("/api/v1/links", get(handlers::links::list_links))
        .route("/api/v1/links", post(handlers::links::create_link))
        .route("/api/v1/links/:id", get(handlers::links::get_link))
        .route("/api/v1/links/:id", put(handlers::links::update_link))
        .route("/api/v1/links/:id", delete(handlers::links::delete_link))
        
        // Health check
        .route("/health", get(handlers::health::health_check))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        );
    
    Ok(app)
}