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

/// Execute export commands.
///
/// # Errors
/// Returns an error if filesystem I/O, serialization, or datastore operations fail.
pub async fn execute(
    args: ExportArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    info!("Starting export to: {}", args.to.display());

    prepare_export_directory(&args.to).await?;

    let export_types = determine_export_types(args.only.as_ref());
    let mut export_stats = ExportStats::new();

    // Export each type
    export_data_type(
        "locations",
        &export_types,
        &args,
        datastore,
        &mut export_stats,
    )
    .await;
    export_data_type("nodes", &export_types, &args, datastore, &mut export_stats).await;
    export_data_type("links", &export_types, &args, datastore, &mut export_stats).await;

    finalize_export(&export_stats, args, output_format)
}

struct ExportStats {
    exported_count: usize,
    errors: Vec<String>,
}

impl ExportStats {
    const fn new() -> Self {
        Self {
            exported_count: 0,
            errors: Vec::new(),
        }
    }

    fn record_success(&mut self, count: usize, data_type: &str) {
        self.exported_count += count;
        info!("Exported {} {}", count, data_type);
    }

    fn record_error(&mut self, error_msg: &str) {
        self.errors.push(error_msg.to_string());
        warn!("{}", error_msg);
    }
}

async fn prepare_export_directory(to: &std::path::Path) -> Result<()> {
    if !to.exists() {
        tokio::fs::create_dir_all(to).await?;
        info!("Created destination directory: {}", to.display());
    }
    Ok(())
}

fn determine_export_types(only: Option<&Vec<String>>) -> Vec<String> {
    only.cloned().unwrap_or_else(|| {
        vec![
            "locations".to_owned(),
            "nodes".to_owned(),
            "links".to_owned(),
        ]
    })
}

async fn export_data_type(
    data_type: &str,
    export_types: &[String],
    args: &ExportArgs,
    datastore: &dyn DataStore,
    stats: &mut ExportStats,
) {
    if !export_types.contains(&data_type.to_owned()) {
        return;
    }

    let result = match data_type {
        "locations" => export_locations(args, datastore).await,
        "nodes" => export_nodes(args, datastore).await,
        "links" => export_links(args, datastore).await,
        _ => return,
    };

    match result {
        Ok(count) => stats.record_success(count, data_type),
        Err(e) => {
            let error_msg = format!("Failed to export {data_type}: {e}");
            stats.record_error(&error_msg);
        }
    }
}

fn finalize_export(
    stats: &ExportStats,
    args: ExportArgs,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let error_count = stats.errors.len();
    let summary = ExportSummary {
        exported_count: stats.exported_count,
        error_count,
        errors: stats.errors.clone(),
        destination: args.to.clone(),
        format: args.format,
    };

    crate::commands::print_output(&summary, output_format)?;

    if !stats.errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Export completed with {} errors",
            error_count
        ));
    }

    info!(
        "Export completed successfully: {} items exported",
        stats.exported_count
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    

    #[tokio::test]
    async fn test_export_stats_new() {
        let stats = ExportStats::new();
        assert_eq!(stats.exported_count, 0);
        assert!(stats.errors.is_empty());
    }

    #[tokio::test]
    async fn test_export_stats_record_success() {
        let mut stats = ExportStats::new();
        stats.record_success(5, "locations");
        assert_eq!(stats.exported_count, 5);
        assert!(stats.errors.is_empty());
    }

    #[tokio::test]
    async fn test_export_stats_record_error() {
        let mut stats = ExportStats::new();
        stats.record_error("Test error message");
        assert_eq!(stats.exported_count, 0);
        assert_eq!(stats.errors.len(), 1);
        assert_eq!(stats.errors[0], "Test error message");
    }

    #[tokio::test]
    async fn test_prepare_export_directory_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("export");

        assert!(!export_path.exists());

        let result = prepare_export_directory(&export_path).await;
        assert!(result.is_ok());
        assert!(export_path.exists());
        assert!(export_path.is_dir());
    }

    #[tokio::test]
    async fn test_prepare_export_directory_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path();

        // Directory already exists
        assert!(export_path.exists());

        let result = prepare_export_directory(export_path).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_determine_export_types_with_only() {
        let only = Some(vec!["nodes".to_string(), "links".to_string()]);
        let types = determine_export_types(only.as_ref());

        assert_eq!(types.len(), 2);
        assert!(types.contains(&"nodes".to_string()));
        assert!(types.contains(&"links".to_string()));
        assert!(!types.contains(&"locations".to_string()));
    }

    #[tokio::test]
    async fn test_determine_export_types_without_only() {
        let types = determine_export_types(None);

        assert_eq!(types.len(), 3);
        assert!(types.contains(&"locations".to_string()));
        assert!(types.contains(&"nodes".to_string()));
        assert!(types.contains(&"links".to_string()));
    }

    #[tokio::test]
    async fn test_finalize_export_success() {
        let temp_dir = TempDir::new().unwrap();
        let stats = ExportStats {
            exported_count: 10,
            errors: Vec::new(),
        };

        let args = ExportArgs {
            to: temp_dir.path().to_path_buf(),
            format: "json".to_string(),
            force: false,
            only: None,
        };

        let result = finalize_export(&stats, args, crate::OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_finalize_export_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        let stats = ExportStats {
            exported_count: 5,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        let args = ExportArgs {
            to: temp_dir.path().to_path_buf(),
            format: "json".to_string(),
            force: false,
            only: None,
        };

        let result = finalize_export(&stats, args, crate::OutputFormat::Json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("2 errors"));
    }
}

#[cfg(test)]
mod exec_tests {
    use super::*;
    use mockall::predicate::always;
    use tempfile::TempDir;
    use unet_core::datastore::{types::PagedResult, MockDataStore};
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};

    #[tokio::test]
    async fn test_execute_empty_exports_ok() {
        let temp = TempDir::new().unwrap();
        let mut mock = MockDataStore::new();
        mock.expect_list_locations()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));
        mock.expect_list_nodes()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));
        mock.expect_list_links()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let args = ExportArgs { to: temp.path().to_path_buf(), format: "json".into(), force: false, only: None };
        let res = execute(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_nodes_overwrite_error_propagates() {
        let temp = TempDir::new().unwrap();
        // Precreate nodes.json to trigger overwrite error (when not force)
        let pre = temp.path().join("nodes.json");
        tokio::fs::write(&pre, "[]").await.unwrap();

        let node = NodeBuilder::new()
            .name("n1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR")
            .role(DeviceRole::Router)
            .build()
            .unwrap();

        let mut mock = MockDataStore::new();
        mock.expect_list_locations()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));
        mock.expect_list_nodes()
            .with(always())
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(PagedResult::new(vec![n], 1, None)) })
            });
        mock.expect_list_links()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let args = ExportArgs { to: temp.path().to_path_buf(), format: "json".into(), force: false, only: Some(vec!["nodes".into()]) };
        let res = execute(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_execute_locations_yaml_writes_file() {
        use mockall::predicate::always;
        use unet_core::datastore::{types::PagedResult, MockDataStore};
        use unet_core::models::Location;
        let temp = TempDir::new().unwrap();
        let loc = Location::new_root("HQ".into(), "building".into());

        let mut mock = MockDataStore::new();
        mock.expect_list_locations()
            .with(always())
            .returning(move |_| {
                let l = loc.clone();
                Box::pin(async move { Ok(PagedResult::new(vec![l], 1, None)) })
            });
        // nodes/links empty for this run
        mock.expect_list_nodes()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));
        mock.expect_list_links()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let args = ExportArgs { to: temp.path().to_path_buf(), format: "yaml".into(), force: true, only: Some(vec!["locations".into()]) };
        let res = execute(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_ok());
        // Verify file exists
        let out = temp.path().join("locations.yaml");
        assert!(out.exists());
    }

    #[tokio::test]
    async fn test_execute_links_json_writes_file() {
        use mockall::predicate::always;
        use unet_core::datastore::{types::PagedResult, MockDataStore};
        let temp = TempDir::new().unwrap();
        let a = uuid::Uuid::new_v4();
        let z = uuid::Uuid::new_v4();
        let link = unet_core::models::Link::new("L1".into(), a, "Gi0/0".into(), z, "Gi0/1".into());

        let mut mock = MockDataStore::new();
        mock.expect_list_locations()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));
        mock.expect_list_nodes()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));
        mock.expect_list_links()
            .with(always())
            .returning(move |_| {
                let l = link.clone();
                Box::pin(async move { Ok(PagedResult::new(vec![l], 1, None)) })
            });

        let args = ExportArgs { to: temp.path().to_path_buf(), format: "json".into(), force: true, only: Some(vec!["links".into()]) };
        let res = execute(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_ok());
        let out = temp.path().join("links.json");
        assert!(out.exists());
    }

    #[tokio::test]
    async fn test_execute_unsupported_format_errors() {
        use mockall::predicate::always;
        use unet_core::datastore::{types::PagedResult, MockDataStore};
        let temp = TempDir::new().unwrap();
        let mut mock = MockDataStore::new();
        mock.expect_list_locations()
            .with(always())
            .returning(|_| {
                let loc = unet_core::models::location::model::Location::new_root(
                    "HQ".into(),
                    "building".into(),
                );
                Box::pin(async move { Ok(PagedResult::new(vec![loc], 1, None)) })
            });
        // keep nodes/links empty
        mock.expect_list_nodes().with(always()).returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));
        mock.expect_list_links().with(always()).returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let args = ExportArgs { to: temp.path().to_path_buf(), format: "xml".into(), force: true, only: Some(vec!["locations".into()]) };
        let res = execute(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_err());
    }
}
