//! Configuration Slicer CLI Tool
//!
//! Command-line tool for slicing network device configurations.

use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use clap::Parser;
use tracing::info;

use config_slicer::{parse_match, slice_config};

#[derive(Parser)]
#[command(name = "config-slicer")]
#[command(about = "Network configuration slicing tool")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Match expression to select config subtree
    #[arg(short, long, value_name = "EXPR")]
    r#match: String,

    /// Optional configuration file path, reads stdin if omitted
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    tracing_subscriber::fmt::init();

    if cli.verbose {
        info!("Starting config-slicer CLI in verbose mode");
    }

    let spec = parse_match(&cli.r#match)?;

    let input = if let Some(path) = cli.file {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let lines = slice_config(&input, &spec)?;
    for line in lines {
        println!("{line}");
    }
    Ok(())
}
