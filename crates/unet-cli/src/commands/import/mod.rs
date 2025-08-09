/// Import command module - handles importing network data from files
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use tracing::info;
use unet_core::datastore::DataStore;

mod loaders;
mod processors;
mod stats;

use processors::{import_links, import_locations, import_nodes};
use stats::{ImportStats, ImportSummary};

#[derive(Args)]
pub struct ImportArgs {
    /// Source directory or file to import from
    #[arg(short, long)]
    pub from: PathBuf,

    /// Format (json, yaml) - auto-detected if not specified
    #[arg(long)]
    pub format: Option<String>,

    /// Validate only, don't actually import
    #[arg(long)]
    pub dry_run: bool,

    /// Continue on errors instead of stopping
    #[arg(long)]
    pub continue_on_error: bool,
}

/// Execute import command with provided arguments
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

/// Finalize import operation with summary and error handling
fn finalize_import(
    stats: &ImportStats,
    args: &ImportArgs,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let summary = ImportSummary::new(stats, args.dry_run);

    crate::commands::print_output(&summary, output_format)?;

    if stats.error_count() > 0 && !args.continue_on_error {
        return Err(anyhow::anyhow!(
            "Import completed with {} errors",
            stats.error_count()
        ));
    }

    info!(
        "Import completed: {} successful, {} errors",
        stats.success_count(),
        stats.error_count()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_finalize_import_success() {
        let stats = ImportStats::new();

        let args = ImportArgs {
            from: PathBuf::from("/tmp"),
            format: None,
            dry_run: false,
            continue_on_error: false,
        };

        let result = finalize_import(&stats, &args, crate::OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_finalize_import_with_errors_continue_on_error() {
        let mut stats = ImportStats::new();
        stats.record_error("Error 1".to_string());
        stats.record_error("Error 2".to_string());

        let args = ImportArgs {
            from: PathBuf::from("/tmp"),
            format: None,
            dry_run: false,
            continue_on_error: true,
        };

        let result = finalize_import(&stats, &args, crate::OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_finalize_import_with_errors_stop_on_error() {
        let mut stats = ImportStats::new();
        stats.record_error("Error 1".to_string());
        stats.record_error("Error 2".to_string());

        let args = ImportArgs {
            from: PathBuf::from("/tmp"),
            format: None,
            dry_run: false,
            continue_on_error: false,
        };

        let result = finalize_import(&stats, &args, crate::OutputFormat::Json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("2 errors"));
    }

    #[tokio::test]
    async fn test_finalize_import_dry_run() {
        let stats = ImportStats::new();

        let args = ImportArgs {
            from: PathBuf::from("/tmp"),
            format: None,
            dry_run: true,
            continue_on_error: false,
        };

        let result = finalize_import(&stats, &args, crate::OutputFormat::Json);
        assert!(result.is_ok());
    }
}
#[cfg(test)]
mod processors_tests;

#[cfg(test)]
mod exec_tests {
    use super::*;
    use tempfile::TempDir;
    use unet_core::datastore::MockDataStore;
    use unet_core::models::{DeviceRole, Location, Node, Vendor};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_execute_import_no_files_ok() {
        let temp = TempDir::new().unwrap();
        let mock = MockDataStore::new();
        let args = ImportArgs { from: temp.path().to_path_buf(), format: None, dry_run: true, continue_on_error: false };
        let res = execute(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_import_dry_run_with_files_ok() {
        let temp = TempDir::new().unwrap();
        // Write minimal valid JSON files for locations, nodes, links
        let loc = Location::new_root("HQ".into(), "building".into());
        let mut node = Node::new("n1".into(), "example.com".into(), Vendor::Cisco, DeviceRole::Router);
        node.model = "ISR".into();
        let link = unet_core::models::Link::new("L1".into(), Uuid::new_v4(), "Gi0/0".into(), Uuid::new_v4(), "Gi0/1".into());

        tokio::fs::write(temp.path().join("locations.json"), serde_json::to_string_pretty(&vec![loc]).unwrap()).await.unwrap();
        tokio::fs::write(temp.path().join("nodes.json"), serde_json::to_string_pretty(&vec![node]).unwrap()).await.unwrap();
        tokio::fs::write(temp.path().join("links.json"), serde_json::to_string_pretty(&vec![link]).unwrap()).await.unwrap();

        // dry_run means no datastore calls are performed, so mock without expectations
        let mock = MockDataStore::new();
        let args = ImportArgs { from: temp.path().to_path_buf(), format: None, dry_run: true, continue_on_error: false };
        let res = execute(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_ok());
    }
}
