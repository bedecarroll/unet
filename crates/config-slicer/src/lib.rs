//! Config-slicer library: exposes CLI parsing and run for reuse in tests/integration.

use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "config-slicer")]
#[command(about = "Network configuration slicing and diffing tool")]
#[command(version)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

/// Execute the CLI logic with a parsed `Cli`.
///
/// # Errors
/// Returns an error if initializing logging fails or if downstream operations fail.
pub fn run_with(cli: &Cli) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    if cli.verbose {
        info!("Starting config-slicer CLI in verbose mode");
    }

    warn!("config-slicer implementation not yet complete");
    Ok(())
}

/// Parse CLI args and run.
///
/// # Errors
/// Returns an error from `run_with` if initialization or execution fails.
pub fn run<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    run_with(&cli)
}
