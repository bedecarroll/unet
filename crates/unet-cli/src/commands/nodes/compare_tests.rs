/// Tests for node comparison functionality
#[cfg(test)]
mod tests {
    use crate::commands::nodes::types::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_compare_nodes_args_creation() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        assert_eq!(args.node_a, first_node_id);
        assert_eq!(args.node_b, Some(second_node_id));
        assert_eq!(args.compare_type, vec![CompareType::All]);
        assert!(!args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_type_variants() {
        assert!(matches!(CompareType::All, CompareType::All));
        assert!(matches!(CompareType::Interfaces, CompareType::Interfaces));
        assert!(matches!(CompareType::Metrics, CompareType::Metrics));
        assert!(matches!(CompareType::System, CompareType::System));
    }

    #[tokio::test]
    async fn test_compare_nodes_interfaces_only() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::Interfaces],
            diff_only: true,
        };

        assert_eq!(args.node_a, first_node_id);
        assert_eq!(args.node_b, Some(second_node_id));
        assert_eq!(args.compare_type, vec![CompareType::Interfaces]);
        assert!(args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_nodes_metrics_only() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::Metrics],
            diff_only: false,
        };

        assert_eq!(args.node_a, first_node_id);
        assert_eq!(args.node_b, Some(second_node_id));
        assert_eq!(args.compare_type, vec![CompareType::Metrics]);
        assert!(!args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_nodes_system_only() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::System],
            diff_only: true,
        };

        assert_eq!(args.node_a, first_node_id);
        assert_eq!(args.node_b, Some(second_node_id));
        assert_eq!(args.compare_type, vec![CompareType::System]);
        assert!(args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_nodes_multiple_types() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![
                CompareType::Interfaces,
                CompareType::Metrics,
                CompareType::System,
            ],
            diff_only: false,
        };

        assert_eq!(args.node_a, first_node_id);
        assert_eq!(args.node_b, Some(second_node_id));
        assert_eq!(args.compare_type.len(), 3);
        assert!(!args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_nodes_historical_comparison() {
        let node_a_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: node_a_id,
            node_b: None, // Historical comparison
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        assert_eq!(args.node_a, node_a_id);
        assert_eq!(args.node_b, None);
        assert_eq!(args.compare_type, vec![CompareType::All]);
        assert!(!args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_nodes_diff_only_true() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::Interfaces, CompareType::System],
            diff_only: true,
        };

        assert_eq!(args.node_a, first_node_id);
        assert_eq!(args.node_b, Some(second_node_id));
        assert_eq!(args.compare_type.len(), 2);
        assert!(args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_nodes_empty_compare_types() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![], // Empty compare types
            diff_only: false,
        };

        assert_eq!(args.node_a, first_node_id);
        assert_eq!(args.node_b, Some(second_node_id));
        assert!(args.compare_type.is_empty());
        assert!(!args.diff_only);
    }

    #[tokio::test]
    async fn test_compare_type_debug_format() {
        let compare_type = CompareType::Interfaces;
        let debug_str = format!("{compare_type:?}");
        assert!(debug_str.contains("Interfaces"));
    }
}

#[cfg(test)]
mod exec_tests {
    use crate::commands::nodes::compare::compare_nodes;
    use crate::commands::nodes::types::{CompareNodeArgs, CompareType};
    use mockall::predicate::eq;
    use unet_core::datastore::{DataStoreError, MockDataStore};
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_compare_nodes_with_two_nodes_all_and_specific_types() {
        // Arrange
        let node_a_id = Uuid::new_v4();
        let node_b_uuid = Uuid::new_v4();

        let node_a = NodeBuilder::new()
            .id(node_a_id)
            .name("node-a")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ABC123")
            .role(DeviceRole::Router)
            .build()
            .unwrap();

        let node_b = NodeBuilder::new()
            .id(node_b_uuid)
            .name("node-b")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("XYZ987")
            .role(DeviceRole::Switch)
            .build()
            .unwrap();

        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(node_a_id))
            .returning(move |_| {
                let node = node_a.clone();
                Box::pin(async move { Ok(node) })
            });
        mock.expect_get_node_required()
            .with(eq(node_b_uuid))
            .returning(move |_| {
                let node = node_b.clone();
                Box::pin(async move { Ok(node) })
            });

        let args = CompareNodeArgs {
            node_a: node_a_id,
            node_b: Some(node_b_uuid),
            compare_type: vec![CompareType::All, CompareType::Interfaces],
            diff_only: false,
        };

        // Act
        let result = compare_nodes(args, &mock, crate::OutputFormat::Json).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compare_nodes_historical_path_when_node_b_missing() {
        // Arrange
        let node_a_id = Uuid::new_v4();
        let node_a = NodeBuilder::new()
            .id(node_a_id)
            .name("node-a")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ABC123")
            .role(DeviceRole::Router)
            .build()
            .unwrap();

        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(node_a_id))
            .returning(move |_| {
                let node = node_a.clone();
                Box::pin(async move { Ok(node) })
            });

        let args = CompareNodeArgs {
            node_a: node_a_id,
            node_b: None,
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        // Act
        let result = compare_nodes(args, &mock, crate::OutputFormat::Yaml).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compare_nodes_errors_when_node_missing() {
        // Arrange
        let node_a_id = Uuid::new_v4();
        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(node_a_id))
            .returning(move |_| {
                Box::pin(async move {
                    Err(DataStoreError::NotFound {
                        entity_type: "Node".to_string(),
                        id: node_a_id.to_string(),
                    })
                })
            });

        let args = CompareNodeArgs {
            node_a: node_a_id,
            node_b: None,
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        // Act
        let result = compare_nodes(args, &mock, crate::OutputFormat::Table).await;

        // Assert: Should surface NotFound from get_node_required
        assert!(matches!(
            result.unwrap_err().downcast_ref::<DataStoreError>(),
            Some(DataStoreError::NotFound { .. })
        ));
    }
}
