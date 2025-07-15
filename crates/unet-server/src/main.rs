//! μNet HTTP Server
//!
//! REST API server for μNet network configuration management.

mod api;
mod background;
mod error;
mod handlers;
mod server;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use unet_core::{config::Config, prelude::*};

/// μNet HTTP Server
#[derive(Parser, Debug)]
#[command(name = "unet-server")]
#[command(about = "μNet HTTP Server for network configuration management")]
#[command(version)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Server host to bind to
    #[arg(long)]
    host: Option<String>,

    /// Server port to bind to
    #[arg(short, long)]
    port: Option<u16>,

    /// Database URL (`SQLite`)
    #[arg(short, long, default_value = "sqlite://unet.db")]
    database_url: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long)]
    log_level: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let mut config = if let Some(config_path) = args.config {
        info!("Loading configuration from: {}", config_path.display());
        Config::from_file(config_path)?
    } else {
        // Try to load from environment, fallback to defaults
        Config::from_env().unwrap_or_else(|_| {
            info!("Using default configuration");
            Config::default()
        })
    };

    // Override config with command line arguments
    if let Some(host) = args.host {
        config.server.host = host;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if let Some(log_level) = args.log_level {
        config.logging.level = log_level;
    }

    // Override database URL from command line or use config
    let database_url = if args.database_url == "sqlite://unet.db" {
        config.database_url()
    } else {
        args.database_url
    };

    // Validate configuration before starting
    config.validate()?;

    // Initialize tracing with config
    init_tracing(&config.logging)?;

    info!("Starting μNet server...");
    info!(
        "Configuration: server={}:{}, database_url={}",
        config.server.host, config.server.port, database_url
    );

    // Start the server
    server::run(config, database_url).await
}
