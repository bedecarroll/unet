//! μNet Command Line Interface
//!
//! CLI tool for μNet network configuration management.

use anyhow::Result;
use clap::{Parser, Subcommand};
use migration::{Migrator, MigratorTrait as _};
use sea_orm::{ConnectOptions, Database};
use std::path::PathBuf;
use tracing::{error, info};
use unet_core::{config::Config, prelude::*};

mod commands;

#[derive(Parser)]
#[command(name = "unet")]
#[command(about = "μNet network configuration management")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Database URL (`SQLite`)
    #[arg(short, long, default_value = "sqlite://unet.db")]
    database_url: String,

    /// Server URL for remote operations
    #[arg(short, long)]
    server: Option<String>,

    /// Authentication token
    #[arg(short, long)]
    token: Option<String>,

    /// Output format
    #[arg(short = 'f', long, default_value = "table")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug)]
enum OutputFormat {
    Table,
    Json,
    Yaml,
}

impl core::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            _ => Err(format!("Invalid output format: {s}")),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Node management commands
    #[command(subcommand)]
    Nodes(commands::nodes::NodeCommands),
    /// Location management commands  
    #[command(subcommand)]
    Locations(commands::locations::LocationCommands),
    /// Link management commands
    #[command(subcommand)]
    Links(commands::links::LinkCommands),
    /// Policy management commands
    #[command(subcommand)]
    Policy(commands::policy::PolicyCommands),
    /// Import data from fixtures or files
    Import(commands::import::ImportArgs),
    /// Export data to files
    Export(commands::export::ExportArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let mut config = if let Some(config_path) = &cli.config {
        Config::from_file(config_path)?
    } else {
        // Try to load from environment, fallback to defaults
        Config::from_env().unwrap_or_else(|_| Config::default())
    };

    // Override log level based on verbose flag
    if cli.verbose {
        "debug".clone_into(&mut config.logging.level);
    }

    // Initialize tracing with config
    init_tracing(&config.logging)?;

    if cli.verbose {
        info!("Starting \u{3bc}Net CLI in verbose mode");
        info!("Database URL: {}", cli.database_url);
        if let Some(config_path) = &cli.config {
            info!("Using configuration from: {}", config_path.display());
        }
    }

    // Initialize SQLite datastore
    let database_url = cli.database_url.clone();
    if cli.verbose {
        info!("Initializing SQLite datastore with URL: {}", database_url);
    }

    // Ensure database is initialized by running migrations

    if cli.verbose {
        info!("Running database migrations...");
    }

    let mut opt = ConnectOptions::new(&database_url);
    opt.sqlx_logging(false);
    let db_for_migration = Database::connect(opt).await.map_err(|e| {
        error!("Failed to connect to database for migration: {}", e);
        anyhow::anyhow!("Failed to connect to database for migration: {}", e)
    })?;

    Migrator::up(&db_for_migration, None).await.map_err(|e| {
        error!("Failed to run migrations: {}", e);
        anyhow::anyhow!("Failed to run migrations: {}", e)
    })?;

    if cli.verbose {
        info!("Database migrations completed successfully");
    }

    let datastore: Box<dyn unet_core::datastore::DataStore> = Box::new(
        unet_core::datastore::sqlite::SqliteStore::new(&database_url)
            .await
            .map_err(|e| {
                error!("Failed to initialize SQLite datastore: {}", e);
                anyhow::anyhow!("Failed to initialize SQLite datastore: {}", e)
            })?,
    );

    // Execute command
    match cli.command {
        Commands::Nodes(node_cmd) => {
            commands::nodes::execute(node_cmd, datastore.as_ref(), cli.output).await
        }
        Commands::Locations(location_cmd) => {
            commands::locations::execute(location_cmd, datastore.as_ref(), cli.output).await
        }
        Commands::Links(link_cmd) => {
            commands::links::execute(link_cmd, datastore.as_ref(), cli.output).await
        }
        Commands::Policy(policy_cmd) => {
            commands::policy::execute(policy_cmd, datastore.as_ref()).await
        }
        Commands::Import(import_args) => {
            commands::import::execute(import_args, datastore.as_ref(), cli.output).await
        }
        Commands::Export(export_args) => {
            commands::export::execute(export_args, datastore.as_ref(), cli.output).await
        }
    }
}
