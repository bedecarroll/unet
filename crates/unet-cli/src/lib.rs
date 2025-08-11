//! Public entrypoints for `unet-cli` to enable in-process testing.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::{error, info};
use unet_core::config::Config;
use unet_core::prelude::*;

pub mod commands;
pub mod runtime;
pub mod dry_run;

pub use runtime::{AppContext, Db};

#[derive(Parser)]
#[command(name = "unet")]
#[command(about = "μNet network configuration management")]
#[command(version)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<std::path::PathBuf>,

    /// Database URL (`SQLite`)
    #[arg(short, long, default_value = "sqlite://unet.db")]
    pub database_url: String,

    /// Server URL for remote operations
    #[arg(short, long)]
    pub server: Option<String>,

    /// Authentication token
    #[arg(short, long)]
    pub token: Option<String>,

    /// Output format
    #[arg(short = 'f', long, default_value = "table")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,

    /// Global dry-run mode (no changes are persisted)
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
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
pub enum Commands {
    /// Node management commands
    #[command(subcommand)]
    Nodes(commands::nodes::NodeCommands),
    /// Location management commands
    #[command(subcommand)]
    Locations(commands::locations::LocationCommands),
    /// Link management commands
    #[command(subcommand)]
    Links(commands::links::LinkCommands),
    /// Vendor management commands
    #[command(subcommand)]
    Vendors(commands::vendors::VendorCommands),
    /// Policy management commands
    #[command(subcommand)]
    Policy(commands::policy::PolicyCommands),
    /// Import data from fixtures or files
    Import(commands::import::ImportArgs),
    /// Export data to files
    Export(commands::export::ExportArgs),
}

/// Run the CLI using parsed `Cli` and an injected runtime context.
///
/// # Errors
/// Returns an error if configuration loading, logging initialization, database connection,
/// migration, or command execution fails.
pub async fn run_with(ctx: AppContext, cli: Cli) -> Result<()> {
    // Load configuration
    let config = load_config(&cli)?;

    // Initialize tracing with config
    init_tracing(&config.logging)?;

    // Initialize SQLite datastore via injected runtime
    let database_url = cli.database_url.clone();
    // Optionally emit debug logs controlled by config logging settings

    let datastore = build_datastore(&ctx, &database_url, cli.dry_run).await?;
    

    // Execute command
    dispatch_command(cli.command, datastore.as_ref(), cli.output).await
}

fn load_config(cli: &Cli) -> Result<Config> {
    let mut config = if let Some(config_path) = &cli.config {
        Config::from_file(config_path)?
    } else {
        Config::from_env().unwrap_or_else(|_| Config::default())
    };
    // Override log level based on verbose flag
    if cli.verbose {
        "debug".clone_into(&mut config.logging.level);
    }
    Ok(config)
}

async fn build_datastore(
    ctx: &AppContext,
    database_url: &str,
    dry_run: bool,
) -> Result<Box<dyn unet_core::datastore::DataStore>> {
    let db = (ctx.connect)(database_url).await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        anyhow::anyhow!("Failed to connect to database: {}", e)
    })?;

    (ctx.migrate)(&db).await.map_err(|e| {
        error!("Failed to run migrations: {}", e);
        anyhow::anyhow!("Failed to run migrations: {}", e)
    })?;

    let base: Box<dyn unet_core::datastore::DataStore> =
        Box::new(unet_core::datastore::sqlite::SqliteStore::from_connection(db.0));
    if dry_run {
        info!("Dry-run mode enabled: no changes will be persisted");
        Ok(Box::new(crate::dry_run::DryRunStore::new(Arc::from(base))))
    } else {
        Ok(base)
    }
}


async fn dispatch_command(
    command: Commands,
    datastore: &dyn unet_core::datastore::DataStore,
    output: OutputFormat,
) -> Result<()> {
    match command {
        Commands::Nodes(cmd) => commands::nodes::execute(cmd, datastore, output).await,
        Commands::Locations(cmd) => commands::locations::execute(cmd, datastore, output).await,
        Commands::Links(cmd) => commands::links::execute(cmd, datastore, output).await,
        Commands::Vendors(cmd) => commands::vendors::execute(cmd, datastore, output).await,
        Commands::Policy(cmd) => commands::policy::execute(cmd, datastore).await,
        Commands::Import(args) => commands::import::execute(args, datastore, output).await,
        Commands::Export(args) => commands::export::execute(args, datastore, output).await,
    }
}

/// Parse args and run with default runtime (used by tests for in-process execution).
///
/// # Errors
/// Returns an error if `run_with` fails.
pub async fn run<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    run_with(AppContext::default(), cli).await
}
