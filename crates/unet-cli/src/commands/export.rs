use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use tracing::{info, warn};
use unet_core::datastore::{DataStore, QueryOptions};

#[derive(Args)]
pub struct ExportArgs {
    /// Destination directory to export to
    #[arg(short, long)]
    to: PathBuf,

    /// Format (json, yaml) - defaults to json
    #[arg(long, default_value = "json")]
    format: String,

    /// Overwrite existing files
    #[arg(long)]
    force: bool,

    /// Export only specific data types (nodes, locations, links)
    #[arg(long, value_delimiter = ',')]
    only: Option<Vec<String>>,
}

pub async fn execute(
    args: ExportArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    info!("Starting export to: {}", args.to.display());

    // Create destination directory if it doesn't exist
    if !args.to.exists() {
        tokio::fs::create_dir_all(&args.to).await?;
        info!("Created destination directory: {}", args.to.display());
    }

    let mut exported_count = 0;
    let mut errors = Vec::new();

    let export_all = args.only.is_none();
    let export_types = args.only.clone().unwrap_or_else(|| {
        vec![
            "locations".to_owned(),
            "nodes".to_owned(),
            "links".to_owned(),
        ]
    });

    // Export locations
    if export_all || export_types.contains(&"locations".to_owned()) {
        match export_locations(&args, datastore).await {
            Ok(count) => {
                exported_count += count;
                info!("Exported {} locations", count);
            }
            Err(e) => {
                let error_msg = format!("Failed to export locations: {e}");
                errors.push(error_msg.clone());
                warn!("{}", error_msg);
            }
        }
    }

    // Export nodes
    if export_all || export_types.contains(&"nodes".to_owned()) {
        match export_nodes(&args, datastore).await {
            Ok(count) => {
                exported_count += count;
                info!("Exported {} nodes", count);
            }
            Err(e) => {
                let error_msg = format!("Failed to export nodes: {e}");
                errors.push(error_msg.clone());
                warn!("{}", error_msg);
            }
        }
    }

    // Export links
    if export_all || export_types.contains(&"links".to_owned()) {
        match export_links(&args, datastore).await {
            Ok(count) => {
                exported_count += count;
                info!("Exported {} links", count);
            }
            Err(e) => {
                let error_msg = format!("Failed to export links: {e}");
                errors.push(error_msg.clone());
                warn!("{}", error_msg);
            }
        }
    }

    // Print summary
    let error_count = errors.len();
    let summary = ExportSummary {
        exported_count,
        error_count,
        errors: errors.clone(),
        destination: args.to.clone(),
        format: args.format.clone(),
    };

    crate::commands::print_output(&summary, output_format)?;

    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Export completed with {} errors",
            error_count
        ));
    }

    info!(
        "Export completed successfully: {} items exported",
        exported_count
    );
    Ok(())
}

#[derive(serde::Serialize)]
struct ExportSummary {
    exported_count: usize,
    error_count: usize,
    errors: Vec<String>,
    destination: PathBuf,
    format: String,
}

async fn export_locations(args: &ExportArgs, datastore: &dyn DataStore) -> Result<usize> {
    let query_options = QueryOptions::default();
    let locations_result = datastore.list_locations(&query_options).await?;
    if locations_result.items.is_empty() {
        return Ok(0);
    }

    let filename = format!("locations.{}", args.format);
    let file_path = args.to.join(&filename);

    if file_path.exists() && !args.force {
        return Err(anyhow::anyhow!(
            "File {} already exists. Use --force to overwrite",
            file_path.display()
        ));
    }

    let content = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&locations_result.items)?,
        "yaml" => serde_yaml::to_string(&locations_result.items)?,
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", args.format)),
    };

    tokio::fs::write(&file_path, content).await?;
    info!(
        "Wrote {} locations to {}",
        locations_result.items.len(),
        file_path.display()
    );

    Ok(locations_result.items.len())
}

async fn export_nodes(args: &ExportArgs, datastore: &dyn DataStore) -> Result<usize> {
    let query_options = QueryOptions::default();
    let nodes_result = datastore.list_nodes(&query_options).await?;
    if nodes_result.items.is_empty() {
        return Ok(0);
    }

    let filename = format!("nodes.{}", args.format);
    let file_path = args.to.join(&filename);

    if file_path.exists() && !args.force {
        return Err(anyhow::anyhow!(
            "File {} already exists. Use --force to overwrite",
            file_path.display()
        ));
    }

    let content = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&nodes_result.items)?,
        "yaml" => serde_yaml::to_string(&nodes_result.items)?,
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", args.format)),
    };

    tokio::fs::write(&file_path, content).await?;
    info!(
        "Wrote {} nodes to {}",
        nodes_result.items.len(),
        file_path.display()
    );

    Ok(nodes_result.items.len())
}

async fn export_links(args: &ExportArgs, datastore: &dyn DataStore) -> Result<usize> {
    let query_options = QueryOptions::default();
    let links_result = datastore.list_links(&query_options).await?;
    if links_result.items.is_empty() {
        return Ok(0);
    }

    let filename = format!("links.{}", args.format);
    let file_path = args.to.join(&filename);

    if file_path.exists() && !args.force {
        return Err(anyhow::anyhow!(
            "File {} already exists. Use --force to overwrite",
            file_path.display()
        ));
    }

    let content = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&links_result.items)?,
        "yaml" => serde_yaml::to_string(&links_result.items)?,
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", args.format)),
    };

    tokio::fs::write(&file_path, content).await?;
    info!(
        "Wrote {} links to {}",
        links_result.items.len(),
        file_path.display()
    );

    Ok(links_result.items.len())
}
