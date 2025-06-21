//! μNet Command Line Interface
//!
//! CLI tool for μNet network configuration management.

use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "unet")]
#[command(about = "μNet network configuration management")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    if cli.verbose {
        info!("Starting μNet CLI in verbose mode");
    }

    warn!("CLI implementation not yet complete");

    Ok(())
}
