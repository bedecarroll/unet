use super::*;

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
