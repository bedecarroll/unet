use anyhow::Result;
use clap::Args;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use unet_core::datastore::DataStore;
use unet_core::prelude::*;

#[derive(Args)]
pub struct ImportArgs {
    /// Source directory or file to import from
    #[arg(short, long)]
    from: PathBuf,

    /// Format (json, yaml) - auto-detected if not specified
    #[arg(long)]
    format: Option<String>,

    /// Validate only, don't actually import
    #[arg(long)]
    dry_run: bool,

    /// Continue on errors instead of stopping
    #[arg(long)]
    continue_on_error: bool,
}

#[allow(clippy::cognitive_complexity)]
pub async fn execute(
    args: ImportArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    info!("Starting import from: {}", args.from.display());

    if args.dry_run {
        info!("Running in dry-run mode - no data will be imported");
    }

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    // Import locations first (dependencies)
    if let Some(locations) = load_locations(&args.from).await? {
        info!("Importing {} locations...", locations.len());
        for location in locations {
            match import_location(&location, datastore, args.dry_run).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    let error_msg = format!("Failed to import location '{}': {}", location.name, e);
                    errors.push(error_msg.clone());
                    warn!("{}", error_msg);

                    if !args.continue_on_error {
                        return Err(anyhow::anyhow!("Import failed: {}", error_msg));
                    }
                }
            }
        }
    }

    // Import nodes second
    if let Some(nodes) = load_nodes(&args.from).await? {
        info!("Importing {} nodes...", nodes.len());
        for node in nodes {
            match import_node(&node, datastore, args.dry_run).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    let error_msg = format!("Failed to import node '{}': {}", node.name, e);
                    errors.push(error_msg.clone());
                    warn!("{}", error_msg);

                    if !args.continue_on_error {
                        return Err(anyhow::anyhow!("Import failed: {}", error_msg));
                    }
                }
            }
        }
    }

    // Import links last (depends on nodes)
    if let Some(links) = load_links(&args.from).await? {
        info!("Importing {} links...", links.len());
        for link in links {
            match import_link(&link, datastore, args.dry_run).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    let error_msg = format!("Failed to import link '{}': {}", link.name, e);
                    errors.push(error_msg.clone());
                    warn!("{}", error_msg);

                    if !args.continue_on_error {
                        return Err(anyhow::anyhow!("Import failed: {}", error_msg));
                    }
                }
            }
        }
    }

    // Print summary
    let summary = ImportSummary {
        success_count,
        error_count,
        errors,
        dry_run: args.dry_run,
    };

    crate::commands::print_output(&summary, output_format)?;

    if error_count > 0 && !args.continue_on_error {
        return Err(anyhow::anyhow!(
            "Import completed with {} errors",
            error_count
        ));
    }

    info!(
        "Import completed: {} successful, {} errors",
        success_count, error_count
    );
    Ok(())
}

#[derive(serde::Serialize)]
struct ImportSummary {
    success_count: usize,
    error_count: usize,
    errors: Vec<String>,
    dry_run: bool,
}

async fn load_locations(base_path: &Path) -> Result<Option<Vec<Location>>> {
    let locations_file = base_path.join("locations.json");
    if !locations_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&locations_file).await?;
    let locations: Vec<Location> = serde_json::from_str(&content)?;
    Ok(Some(locations))
}

async fn load_nodes(base_path: &Path) -> Result<Option<Vec<Node>>> {
    let nodes_file = base_path.join("nodes.json");
    if !nodes_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&nodes_file).await?;
    let nodes: Vec<Node> = serde_json::from_str(&content)?;
    Ok(Some(nodes))
}

async fn load_links(base_path: &Path) -> Result<Option<Vec<Link>>> {
    let links_file = base_path.join("links.json");
    if !links_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&links_file).await?;
    let links: Vec<Link> = serde_json::from_str(&content)?;
    Ok(Some(links))
}

async fn import_location(
    location: &Location,
    datastore: &dyn DataStore,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        info!("Would import location: {}", location.name);
        return Ok(());
    }

    datastore.create_location(location).await?;
    Ok(())
}

async fn import_node(node: &Node, datastore: &dyn DataStore, dry_run: bool) -> Result<()> {
    if dry_run {
        info!("Would import node: {}", node.name);
        return Ok(());
    }

    datastore.create_node(node).await?;
    Ok(())
}

async fn import_link(link: &Link, datastore: &dyn DataStore, dry_run: bool) -> Result<()> {
    if dry_run {
        info!("Would import link: {}", link.name);
        return Ok(());
    }

    datastore.create_link(link).await?;
    Ok(())
}
