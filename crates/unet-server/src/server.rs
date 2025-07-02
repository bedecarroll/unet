//! Server configuration and startup

use anyhow::Result;
use axum::{
    Router,
    http::{StatusCode, Uri},
    response::Redirect,
    routing::{delete, get, post, put},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use unet_core::{
    config::Config,
    datastore::{DataStore, sqlite::SqliteStore},
    metrics::MetricsManager,
    policy_integration::PolicyService,
};

use crate::{
    background::BackgroundTasks,
    handlers, middleware,
    network_access::{NetworkAccessConfig, NetworkAccessControl},
    security_audit::{SecurityAuditConfig, init_security_audit_logger},
    tls::TlsManager,
};
use unet_core::auth::{AuthService, JwtConfig};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub datastore: Arc<dyn DataStore + Send + Sync>,
    pub policy_service: PolicyService,
    pub config: Config,
    pub auth_service: AuthService,
    pub network_access: Arc<NetworkAccessControl>,
    pub metrics_manager: MetricsManager,
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

    // Check if TLS is configured
    if let Some(ref tls_config) = config.server.tls {
        info!("Starting μNet server with TLS support on https://{}", addr);
        run_https_server(app, addr, tls_config.clone(), &config).await
    } else {
        info!("Starting μNet server on http://{}", addr);
        run_http_server(app, addr).await
    }
}

/// Run HTTPS server with TLS support
async fn run_https_server(
    app: Router,
    addr: SocketAddr,
    tls_config: unet_core::config::TlsConfig,
    config: &Config,
) -> Result<()> {
    // Initialize TLS manager
    let tls_manager = TlsManager::new(tls_config.clone());

    // Validate certificates
    tls_manager.validate_certificates()?;

    // Load TLS configuration
    let rustls_config = tls_manager.load_rustls_config().await?;

    // Get certificate info for logging
    let cert_info = tls_manager.get_certificate_info()?;
    info!("TLS configuration: {}", cert_info);

    // Start HTTP redirect server if configured
    if tls_config.force_https {
        if let Some(http_port) = tls_config.http_redirect_port {
            let redirect_addr = SocketAddr::from((addr.ip(), http_port));
            info!("Starting HTTP->HTTPS redirect server on {}", redirect_addr);

            tokio::spawn(async move {
                if let Err(e) = run_http_redirect_server(redirect_addr, addr.port()).await {
                    warn!("HTTP redirect server failed: {}", e);
                }
            });
        }
    }

    // Start HTTPS server
    axum_server::bind_rustls(addr, rustls_config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Run HTTP server (non-TLS)
async fn run_http_server(app: Router, addr: SocketAddr) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Run HTTP redirect server that redirects all requests to HTTPS
async fn run_http_redirect_server(bind_addr: SocketAddr, https_port: u16) -> Result<()> {
    let redirect_app = Router::new().fallback(move |uri: Uri| async move {
        let host = uri.host().unwrap_or("localhost");
        let https_url = if https_port == 443 {
            format!(
                "https://{}{}",
                host,
                uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("")
            )
        } else {
            format!(
                "https://{}:{}{}",
                host,
                https_port,
                uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("")
            )
        };

        Redirect::permanent(&https_url)
    });

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, redirect_app).await?;
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

    // Initialize security audit logger
    info!("Initializing security audit logger");
    let audit_config = SecurityAuditConfig::default();
    let audit_logger = init_security_audit_logger(audit_config).await;

    // Initialize authentication service
    info!("Initializing authentication service");
    let jwt_config = JwtConfig {
        secret: config.auth.jwt_secret.clone(),
        issuer: "unet".to_string(),
        audience: "unet-api".to_string(),
        access_token_ttl: chrono::Duration::hours(24),
        algorithm: jsonwebtoken::Algorithm::HS256,
    };
    let auth_service = AuthService::new(jwt_config);

    // Initialize system roles
    auth_service
        .initialize_system_roles()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize system roles: {}", e))?;

    // Initialize network access control
    info!("Initializing network access control");
    let network_access_config = NetworkAccessConfig {
        enabled: config.network.enabled,
        allow_list: config
            .network
            .allowed_ips
            .iter()
            .filter_map(|ip| ip.parse().ok())
            .collect(),
        deny_list: config
            .network
            .blocked_ips
            .iter()
            .filter_map(|ip| ip.parse().ok())
            .collect(),
        allow_ranges: config.network.allowed_ranges.clone(),
        deny_ranges: config.network.blocked_ranges.clone(),
        blocked_countries: config.network.blocked_countries.iter().cloned().collect(),
        allowed_countries: config
            .network
            .allowed_countries
            .as_ref()
            .map(|countries| countries.iter().cloned().collect()),
        enable_geolocation: config.network.enable_geolocation,
        untrusted_max_request_size: config.network.untrusted_max_request_size,
        enable_network_rate_limits: config.network.enable_network_rate_limits,
        ..Default::default()
    };

    let network_access = Arc::new(
        NetworkAccessControl::new(network_access_config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize network access control: {}", e))?,
    );

    // Initialize metrics manager
    info!("Initializing metrics manager");
    let metrics_manager = MetricsManager::new(config.metrics.clone())
        .map_err(|e| anyhow::anyhow!("Failed to initialize metrics manager: {}", e))?;

    // Start background metrics collection
    metrics_manager.start_background_collection();

    let app_state = AppState {
        datastore: datastore.clone(),
        policy_service: policy_service.clone(),
        config: config.clone(),
        auth_service,
        network_access,
        metrics_manager,
    };

    // Start background tasks
    let background_tasks = BackgroundTasks::new(config.clone(), datastore, policy_service);
    background_tasks.start().await;

    let app = Router::new()
        // Authentication endpoints (public)
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route(
            "/api/v1/auth/permissions",
            get(handlers::auth::get_permissions),
        )
        // Protected authentication endpoints
        .route("/api/v1/auth/me", get(handlers::auth::get_current_user))
        .route(
            "/api/v1/auth/change-password",
            post(handlers::auth::change_password),
        )
        .route("/api/v1/auth/users", post(handlers::auth::create_user))
        .route("/api/v1/auth/users", get(handlers::auth::list_users))
        .route("/api/v1/auth/users/:id", get(handlers::auth::get_user))
        .route("/api/v1/auth/users/:id", put(handlers::auth::update_user))
        .route("/api/v1/auth/roles", post(handlers::auth::create_role))
        .route("/api/v1/auth/roles", get(handlers::auth::list_roles))
        .route(
            "/api/v1/auth/api-keys",
            post(handlers::auth::create_api_key),
        )
        .route("/api/v1/auth/api-keys", get(handlers::auth::list_api_keys))
        .route(
            "/api/v1/auth/api-keys/:id",
            delete(handlers::auth::delete_api_key),
        )
        // Add authentication middleware to protected routes
        .layer(axum::middleware::from_fn_with_state(
            app_state.auth_service.clone(),
            middleware::auth::jwt_auth,
        ))
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
        // Git version control endpoints
        .route(
            "/api/v1/git/sync/status",
            get(handlers::git::get_git_sync_status),
        )
        .route("/api/v1/git/sync", post(handlers::git::trigger_git_sync))
        .route(
            "/api/v1/git/changes",
            get(handlers::git::get_change_history),
        )
        .route(
            "/api/v1/git/changes/:id",
            get(handlers::git::get_change_details),
        )
        .route(
            "/api/v1/git/repository",
            get(handlers::git::get_repository_info),
        )
        .route(
            "/api/v1/git/webhooks",
            post(handlers::git::handle_git_webhook),
        )
        .route(
            "/api/v1/git/webhooks/config",
            get(handlers::git::get_webhook_config),
        )
        .route(
            "/api/v1/git/webhooks/config",
            put(handlers::git::update_webhook_config),
        )
        // Change management endpoints
        .route("/api/v1/changes", get(handlers::changes::list_changes))
        .route("/api/v1/changes", post(handlers::changes::create_change))
        .route("/api/v1/changes/:id", get(handlers::changes::get_change))
        .route(
            "/api/v1/changes/:id/approve",
            post(handlers::changes::approve_change),
        )
        .route(
            "/api/v1/changes/:id/reject",
            post(handlers::changes::reject_change),
        )
        .route(
            "/api/v1/changes/:id/apply",
            post(handlers::changes::apply_change),
        )
        .route(
            "/api/v1/changes/:id/rollback",
            post(handlers::changes::rollback_change),
        )
        .route(
            "/api/v1/changes/:id/audit",
            get(handlers::changes::get_change_audit_trail),
        )
        .route(
            "/api/v1/changes/history/:entity_type/:entity_id",
            get(handlers::changes::get_change_history),
        )
        .route(
            "/api/v1/changes/pending",
            get(handlers::changes::get_pending_approvals),
        )
        .route(
            "/api/v1/changes/stats",
            get(handlers::changes::get_change_statistics),
        )
        .route(
            "/api/v1/changes/status",
            get(handlers::changes::get_change_management_status),
        )
        // Change notification endpoints
        .route(
            "/api/v1/changes/notifications/subscribe",
            post(handlers::changes::subscribe_to_notifications),
        )
        .route(
            "/api/v1/changes/notifications/subscribe/:user_id",
            delete(handlers::changes::unsubscribe_from_notifications),
        )
        .route(
            "/api/v1/changes/notifications/send",
            post(handlers::changes::send_notification),
        )
        .route(
            "/api/v1/changes/notifications/config/:user_id",
            get(handlers::changes::get_notification_config),
        )
        .route(
            "/api/v1/changes/notifications/config/:user_id",
            put(handlers::changes::update_notification_config),
        )
        .route(
            "/api/v1/changes/notifications/history/:user_id",
            get(handlers::changes::get_notification_history),
        )
        // Certificate management endpoints
        .route(
            "/api/v1/certificates/status",
            get(handlers::certificates::get_certificate_status),
        )
        .route(
            "/api/v1/certificates/rotate",
            post(handlers::certificates::rotate_certificates),
        )
        .route(
            "/api/v1/certificates/backup",
            post(handlers::certificates::backup_certificates),
        )
        .route(
            "/api/v1/certificates/expiration",
            get(handlers::certificates::get_certificate_expiration),
        )
        .route(
            "/api/v1/certificates/health",
            get(handlers::certificates::certificate_health_check),
        )
        // Alerting endpoints
        .route("/api/v1/alerts", get(handlers::alerting::get_alerts))
        .route("/api/v1/alerts/:id", get(handlers::alerting::get_alert))
        .route(
            "/api/v1/alerts/:id/acknowledge",
            post(handlers::alerting::acknowledge_alert),
        )
        .route(
            "/api/v1/alerts/:id/resolve",
            post(handlers::alerting::resolve_alert),
        )
        .route(
            "/api/v1/alerts/:id/escalate",
            post(handlers::alerting::manual_escalate_alert),
        )
        .route(
            "/api/v1/alert-rules",
            get(handlers::alerting::get_alert_rules),
        )
        .route(
            "/api/v1/alert-rules",
            post(handlers::alerting::create_alert_rule),
        )
        .route(
            "/api/v1/alert-rules/:id",
            get(handlers::alerting::get_alert_rule),
        )
        .route(
            "/api/v1/alert-rules/:id",
            put(handlers::alerting::update_alert_rule),
        )
        .route(
            "/api/v1/alert-rules/:id",
            delete(handlers::alerting::delete_alert_rule),
        )
        .route(
            "/api/v1/notification-channels",
            get(handlers::alerting::get_notification_channels),
        )
        .route(
            "/api/v1/notification-channels",
            post(handlers::alerting::create_notification_channel),
        )
        .route(
            "/api/v1/notifications/test",
            post(handlers::alerting::test_notifications),
        )
        .route(
            "/api/v1/escalation/policies",
            get(handlers::alerting::get_escalation_policies),
        )
        .route(
            "/api/v1/escalation/policies",
            post(handlers::alerting::create_escalation_policy),
        )
        .route(
            "/api/v1/escalation/stats",
            get(handlers::alerting::get_escalation_stats),
        )
        .route(
            "/api/v1/alerting/config",
            get(handlers::alerting::get_alerting_config),
        )
        .route(
            "/api/v1/alerting/config",
            put(handlers::alerting::update_alerting_config),
        )
        .route(
            "/api/v1/alerting/stats",
            get(handlers::alerting::get_alert_statistics),
        )
        // Network access control endpoints
        .route(
            "/api/v1/network-access/config",
            get(handlers::network_access::get_network_config),
        )
        .route(
            "/api/v1/network-access/config",
            put(handlers::network_access::update_network_config),
        )
        .route(
            "/api/v1/network-access/stats",
            get(handlers::network_access::get_network_stats),
        )
        .route(
            "/api/v1/network-access/test",
            post(handlers::network_access::test_network_access),
        )
        .route(
            "/api/v1/network-access/blocked-ips",
            get(handlers::network_access::get_blocked_ips),
        )
        .route(
            "/api/v1/network-access/block/:ip",
            post(handlers::network_access::block_ip),
        )
        .route(
            "/api/v1/network-access/unblock/:ip",
            delete(handlers::network_access::unblock_ip),
        )
        .route(
            "/api/v1/network-access/health",
            get(handlers::network_access::get_network_access_health),
        )
        // Distributed locking endpoints
        .route(
            "/api/v1/locks/stats",
            get(handlers::distributed_locking::get_lock_stats),
        )
        .route(
            "/api/v1/locks",
            get(handlers::distributed_locking::list_locks),
        )
        .route(
            "/api/v1/locks/acquire",
            post(handlers::distributed_locking::acquire_lock),
        )
        .route(
            "/api/v1/locks/:key",
            delete(handlers::distributed_locking::release_lock),
        )
        .route(
            "/api/v1/locks/:key",
            get(handlers::distributed_locking::get_lock_info),
        )
        .route(
            "/api/v1/locks/:key/extend",
            post(handlers::distributed_locking::extend_lock),
        )
        .route(
            "/api/v1/locks/leader-election",
            post(handlers::distributed_locking::create_leader_election),
        )
        .route(
            "/api/v1/locks/leader-election/:election_key/status",
            get(handlers::distributed_locking::get_leader_election_status),
        )
        .route(
            "/api/v1/locks/monitor",
            get(handlers::distributed_locking::get_lock_monitor_report),
        )
        .route(
            "/api/v1/locks/config",
            get(handlers::distributed_locking::get_lock_config),
        )
        .route(
            "/api/v1/locks/config",
            put(handlers::distributed_locking::update_lock_config),
        )
        .route(
            "/api/v1/locks/health",
            get(handlers::distributed_locking::get_lock_health),
        )
        .route(
            "/api/v1/locks/test",
            post(handlers::distributed_locking::test_distributed_locking),
        )
        // Vulnerability scanning endpoints (temporarily disabled)
        // .route(
        //     "/api/v1/vulnerability/scanners/configure",
        //     post(handlers::vulnerability::configure_scanner),
        // )
        // .route(
        //     "/api/v1/vulnerability/scans",
        //     post(handlers::vulnerability::execute_scan),
        // )
        // .route(
        //     "/api/v1/vulnerability/scans/:scan_id/status",
        //     get(handlers::vulnerability::get_scan_status),
        // )
        // .route(
        //     "/api/v1/vulnerability/scans/:scan_id/result",
        //     get(handlers::vulnerability::get_scan_result),
        // )
        // .route(
        //     "/api/v1/vulnerability/scans/:scan_id/cancel",
        //     post(handlers::vulnerability::cancel_scan),
        // )
        // .route(
        //     "/api/v1/vulnerability/scans",
        //     get(handlers::vulnerability::get_scan_results),
        // )
        // .route(
        //     "/api/v1/vulnerability/summary",
        //     get(handlers::vulnerability::get_vulnerability_summary),
        // )
        // .route(
        //     "/api/v1/vulnerability/scanners",
        //     get(handlers::vulnerability::get_available_scanners),
        // )
        // .route(
        //     "/api/v1/vulnerability/scanners/:scanner_type/config",
        //     get(handlers::vulnerability::get_scanner_config),
        // )
        // .route(
        //     "/api/v1/vulnerability/scanners/:scanner_type/config",
        //     put(handlers::vulnerability::update_scanner_config),
        // )
        // .route(
        //     "/api/v1/vulnerability/targets/:target/history",
        //     get(handlers::vulnerability::get_target_scan_history),
        // )
        // .route(
        //     "/api/v1/vulnerability/metrics",
        //     get(handlers::vulnerability::get_vulnerability_metrics),
        // )
        // Health check
        // Health check endpoints
        .route("/health", get(handlers::health::health_check))
        .route(
            "/health/detailed",
            get(handlers::health::detailed_health_check),
        )
        .route("/ready", get(handlers::health::readiness_check))
        .route("/live", get(handlers::health::liveness_check))
        .route("/health/lb", get(handlers::health::load_balancer_status))
        // Metrics endpoints
        .route("/metrics", get(handlers::metrics::get_prometheus_metrics))
        .route(
            "/api/v1/metrics/health",
            get(handlers::metrics::get_system_health),
        )
        .route(
            "/api/v1/metrics/performance",
            get(handlers::metrics::get_performance_metrics),
        )
        .route(
            "/api/v1/metrics/business",
            get(handlers::metrics::get_business_metrics),
        )
        .route(
            "/api/v1/metrics/config",
            get(handlers::metrics::get_metrics_config),
        )
        .route(
            "/api/v1/metrics/query",
            get(handlers::metrics::query_metrics),
        )
        // Performance optimization endpoints
        .route(
            "/api/v1/performance/metrics",
            get(handlers::performance::get_performance_metrics),
        )
        .route(
            "/api/v1/performance/metrics/:operation",
            get(handlers::performance::get_operation_metrics),
        )
        .route(
            "/api/v1/performance/report",
            get(handlers::performance::get_performance_report),
        )
        .route(
            "/api/v1/performance/metrics/reset",
            post(handlers::performance::reset_performance_metrics),
        )
        .route(
            "/api/v1/performance/cache/stats",
            get(handlers::performance::get_cache_stats),
        )
        .route(
            "/api/v1/performance/cache/clear",
            post(handlers::performance::clear_cache),
        )
        .route(
            "/api/v1/performance/benchmark",
            post(handlers::performance::run_benchmark),
        )
        .route(
            "/api/v1/performance/benchmark/templates",
            get(handlers::performance::get_benchmark_templates),
        )
        .route(
            "/api/v1/performance/status",
            get(handlers::performance::get_performance_status),
        )
        .route(
            "/api/v1/performance/recommendations",
            get(handlers::performance::get_optimization_recommendations),
        )
        // Cluster coordination routes
        .route(
            "/api/v1/cluster/stats",
            get(handlers::cluster::get_cluster_stats),
        )
        .route(
            "/api/v1/cluster/health",
            get(handlers::cluster::get_cluster_health),
        )
        .route(
            "/api/v1/cluster/nodes",
            get(handlers::cluster::list_cluster_nodes),
        )
        .route(
            "/api/v1/cluster/nodes",
            post(handlers::cluster::register_node),
        )
        .route(
            "/api/v1/cluster/nodes/:node_id",
            get(handlers::cluster::get_node_details),
        )
        .route(
            "/api/v1/cluster/nodes/:node_id",
            delete(handlers::cluster::remove_node),
        )
        .route(
            "/api/v1/cluster/nodes/metrics",
            post(handlers::cluster::update_node_metrics),
        )
        .route(
            "/api/v1/cluster/config",
            get(handlers::cluster::get_cluster_config),
        )
        .route(
            "/api/v1/cluster/config",
            put(handlers::cluster::update_cluster_config),
        )
        .route(
            "/api/v1/cluster/scaling/recommendation",
            post(handlers::cluster::get_scaling_recommendation),
        )
        .route(
            "/api/v1/cluster/scaling/action",
            post(handlers::cluster::trigger_scaling_action),
        )
        .route(
            "/api/v1/cluster/scaling/history",
            get(handlers::cluster::get_scaling_history),
        )
        // Resource management endpoints
        .route(
            "/api/v1/resource-management/status",
            get(handlers::resource_management::get_resource_status),
        )
        .route(
            "/api/v1/resource-management/memory/status",
            get(handlers::resource_management::get_memory_status),
        )
        .route(
            "/api/v1/resource-management/memory/optimize",
            post(handlers::resource_management::optimize_memory),
        )
        .route(
            "/api/v1/resource-management/limits/status",
            get(handlers::resource_management::get_limits_status),
        )
        .route(
            "/api/v1/resource-management/limits",
            put(handlers::resource_management::update_limits),
        )
        .route(
            "/api/v1/resource-management/degradation/status",
            get(handlers::resource_management::get_degradation_status),
        )
        .route(
            "/api/v1/resource-management/emergency",
            post(handlers::resource_management::trigger_emergency_mode),
        )
        .route(
            "/api/v1/resource-management/metrics",
            get(handlers::resource_management::get_monitoring_metrics),
        )
        .route(
            "/api/v1/resource-management/alerts",
            get(handlers::resource_management::get_resource_alerts),
        )
        .route(
            "/api/v1/resource-management/alerts/:alert_id/acknowledge",
            post(handlers::resource_management::acknowledge_alert),
        )
        .route(
            "/api/v1/resource-management/capacity/recommendations",
            get(handlers::resource_management::get_capacity_recommendations),
        )
        .route(
            "/api/v1/resource-management/config",
            get(handlers::resource_management::get_resource_config),
        )
        .route(
            "/api/v1/resource-management/config",
            put(handlers::resource_management::update_resource_config),
        )
        .route(
            "/api/v1/resource-management/health",
            get(handlers::resource_management::resource_health_check),
        )
        // Add middleware
        .layer(
            ServiceBuilder::new()
                // Performance monitoring (should be early in the stack)
                .layer(axum::middleware::from_fn_with_state(
                    app_state.metrics_manager.clone(),
                    middleware::performance_monitoring_middleware,
                ))
                // Security and validation middleware
                .layer(axum::middleware::from_fn(
                    middleware::security_headers_middleware,
                ))
                .layer(axum::middleware::from_fn(
                    middleware::security_audit_middleware,
                ))
                .layer(axum::middleware::from_fn(
                    middleware::network_access_middleware,
                ))
                .layer(axum::middleware::from_fn(middleware::rate_limit_middleware))
                .layer(axum::middleware::from_fn(
                    middleware::validate_request_middleware,
                ))
                .layer(axum::middleware::from_fn(
                    middleware::validate_headers_middleware,
                ))
                // Logging and tracing
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
                            // Callback signature required by tower-http TraceLayer
                            tracing::info!("Processing request");
                        })
                        .on_response(
                            |response: &axum::http::Response<_>,
                             latency: std::time::Duration,
                             _span: &tracing::Span| {
                                // Callback signature required by tower-http TraceLayer
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
                                // Callback signature required by tower-http TraceLayer
                                tracing::error!(
                                    error = %error,
                                    latency_ms = latency.as_millis(),
                                    "Request failed"
                                );
                            },
                        ),
                )
                .layer(CorsLayer::permissive()),
        )
        // Add application state
        .with_state(app_state);

    Ok(app)
}
