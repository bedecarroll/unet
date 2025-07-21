//! Router configuration and route definitions

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use super::app_state::AppState;
use crate::handlers;

/// Create the router with all API endpoints
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .merge(create_node_routes())
        .merge(create_policy_routes())
}

/// Create node-related routes
pub fn create_node_routes() -> Router<AppState> {
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
pub fn create_policy_routes() -> Router<AppState> {
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
    use crate::server::app_state::tests::create_mock_app_state;

    #[tokio::test]
    async fn test_create_router() {
        let router = create_router();
        let app_state = create_mock_app_state().await;
        let _router_with_state: axum::Router = router.with_state(app_state);
    }

    #[tokio::test]
    async fn test_create_node_routes() {
        let node_router = create_node_routes();
        let app_state = create_mock_app_state().await;
        let _router_with_state: axum::Router = node_router.with_state(app_state);
    }

    #[tokio::test]
    async fn test_create_policy_routes() {
        let policy_router = create_policy_routes();
        let app_state = create_mock_app_state().await;
        let _router_with_state: axum::Router = policy_router.with_state(app_state);
    }
}
