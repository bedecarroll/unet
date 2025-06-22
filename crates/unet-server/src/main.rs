//! μNet HTTP Server
//!
//! REST API server for μNet network configuration management.

mod api;
mod error;
mod handlers;
mod server;

use anyhow::Result;
use tracing::info;
use unet_core::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_default_tracing();

    info!("Starting μNet server...");

    // Start the server
    server::run().await
}
