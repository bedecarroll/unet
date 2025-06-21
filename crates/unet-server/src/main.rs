//! μNet HTTP Server
//!
//! REST API server for μNet network configuration management.

use anyhow::Result;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting μNet server...");
    warn!("Server implementation not yet complete");

    Ok(())
}
