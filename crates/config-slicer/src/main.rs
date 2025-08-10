//! Configuration Slicer CLI Tool
//!
//! Command-line tool for slicing and diffing network device configurations.

use clap::Parser;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "config-slicer")]
#[command(about = "Network configuration slicing and diffing tool")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    if cli.verbose {
        info!("Starting config-slicer CLI in verbose mode");
    }

    warn!("config-slicer implementation not yet complete");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parse_verbose_flag() {
        // Arrange: simulate args
        let args = ["config-slicer", "--verbose"]; // Act
        let cli = Cli::parse_from(args);
        // Assert
        assert!(cli.verbose);
    }

    #[test]
    fn test_help_runs() {
        // Use clap parser help generation to exercise code path
        let _ = Cli::command().render_help();
        assert!(true);
    }
}
