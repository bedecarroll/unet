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

pub async fn execute(
    args: ImportArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    info!("Starting import from: {}", args.from.display());

    if args.dry_run {
        info!("Running in dry-run mode - no data will be imported");
    }

    let mut import_stats = ImportStats::new();

    // Import in dependency order: locations, nodes, links
    import_locations(&args, datastore, &mut import_stats).await?;
    import_nodes(&args, datastore, &mut import_stats).await?;
    import_links(&args, datastore, &mut import_stats).await?;

    finalize_import(&import_stats, &args, output_format)
}

struct ImportStats {
    success_count: usize,
    error_count: usize,
    errors: Vec<String>,
}

impl ImportStats {
    const fn new() -> Self {
        Self {
            success_count: 0,
            error_count: 0,
            errors: Vec::new(),
        }
    }

    const fn record_success(&mut self) {
        self.success_count += 1;
    }

    fn record_error(&mut self, error_msg: String) {
        self.error_count += 1;
        self.errors.push(error_msg);
    }
}

async fn import_locations(
    args: &ImportArgs,
    datastore: &dyn DataStore,
    stats: &mut ImportStats,
) -> Result<()> {
    let Some(locations) = load_locations(&args.from).await? else {
        return Ok(());
    };

    info!("Importing {} locations...", locations.len());
    for location in locations {
        process_import_item(
            || import_location(&location, datastore, args.dry_run),
            &format!("location '{}'", location.name),
            args.continue_on_error,
            stats,
        )
        .await?;
    }
    Ok(())
}

async fn import_nodes(
    args: &ImportArgs,
    datastore: &dyn DataStore,
    stats: &mut ImportStats,
) -> Result<()> {
    let Some(nodes) = load_nodes(&args.from).await? else {
        return Ok(());
    };

    info!("Importing {} nodes...", nodes.len());
    for node in nodes {
        process_import_item(
            || import_node(&node, datastore, args.dry_run),
            &format!("node '{}'", node.name),
            args.continue_on_error,
            stats,
        )
        .await?;
    }
    Ok(())
}

async fn import_links(
    args: &ImportArgs,
    datastore: &dyn DataStore,
    stats: &mut ImportStats,
) -> Result<()> {
    let Some(links) = load_links(&args.from).await? else {
        return Ok(());
    };

    info!("Importing {} links...", links.len());
    for link in links {
        process_import_item(
            || import_link(&link, datastore, args.dry_run),
            &format!("link '{}'", link.name),
            args.continue_on_error,
            stats,
        )
        .await?;
    }
    Ok(())
}

async fn process_import_item<F, Fut>(
    import_fn: F,
    item_description: &str,
    continue_on_error: bool,
    stats: &mut ImportStats,
) -> Result<()>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    match import_fn().await {
        Ok(()) => stats.record_success(),
        Err(e) => {
            let error_msg = format!("Failed to import {item_description}: {e}");
            stats.record_error(error_msg.clone());
            warn!("{}", error_msg);

            if !continue_on_error {
                return Err(anyhow::anyhow!("Import failed: {}", error_msg));
            }
        }
    }
    Ok(())
}

fn finalize_import(
    stats: &ImportStats,
    args: &ImportArgs,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let summary = ImportSummary {
        success_count: stats.success_count,
        error_count: stats.error_count,
        errors: stats.errors.clone(),
        dry_run: args.dry_run,
    };

    crate::commands::print_output(&summary, output_format)?;

    if stats.error_count > 0 && !args.continue_on_error {
        return Err(anyhow::anyhow!(
            "Import completed with {} errors",
            stats.error_count
        ));
    }

    info!(
        "Import completed: {} successful, {} errors",
        stats.success_count, stats.error_count
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
