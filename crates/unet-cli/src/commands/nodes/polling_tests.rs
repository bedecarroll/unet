/// Tests for node polling functionality
#[cfg(test)]
mod tests {
    use super::super::polling::polling_node;
    use crate::OutputFormat;
    use crate::commands::nodes::types::*;
    use migration::{Migrator, MigratorTrait};
    use unet_core::{
        datastore::{DataStore, sqlite::SqliteStore},
        models::*,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_polling_node_args_creation() {
        let node_id = Uuid::new_v4();

        let args = PollingNodeArgs {
            id: node_id,
            action: PollingAction::Status,
            detailed: false,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.action, PollingAction::Status));
        assert!(!args.detailed);
    }

    #[tokio::test]
    async fn test_polling_action_variants() {
        assert!(matches!(PollingAction::Status, PollingAction::Status));
        assert!(matches!(PollingAction::Start, PollingAction::Start));
        assert!(matches!(PollingAction::Stop, PollingAction::Stop));
        assert!(matches!(PollingAction::Restart, PollingAction::Restart));
        assert!(matches!(PollingAction::History, PollingAction::History));
    }

    #[tokio::test]
    async fn test_polling_node_start_action() {
        let node_id = Uuid::new_v4();

        let args = PollingNodeArgs {
            id: node_id,
            action: PollingAction::Start,
            detailed: true,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.action, PollingAction::Start));
        assert!(args.detailed);
    }

    #[tokio::test]
    async fn test_polling_node_stop_action() {
        let node_id = Uuid::new_v4();

        let args = PollingNodeArgs {
            id: node_id,
            action: PollingAction::Stop,
            detailed: false,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.action, PollingAction::Stop));
        assert!(!args.detailed);
    }

    #[tokio::test]
    async fn test_polling_node_restart_action() {
        let node_id = Uuid::new_v4();

        let args = PollingNodeArgs {
            id: node_id,
            action: PollingAction::Restart,
            detailed: true,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.action, PollingAction::Restart));
        assert!(args.detailed);
    }

    #[tokio::test]
    async fn test_polling_node_history_action() {
        let node_id = Uuid::new_v4();

        let args = PollingNodeArgs {
            id: node_id,
            action: PollingAction::History,
            detailed: false,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.action, PollingAction::History));
        assert!(!args.detailed);
    }

    #[tokio::test]
    async fn test_polling_node_detailed_true() {
        let node_id = Uuid::new_v4();

        let args = PollingNodeArgs {
            id: node_id,
            action: PollingAction::Restart,
            detailed: true,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.action, PollingAction::Restart));
        assert!(args.detailed);
    }

    #[tokio::test]
    async fn test_polling_action_debug_format() {
        let action = PollingAction::Start;
        let debug_str = format!("{action:?}");
        assert!(debug_str.contains("Start"));
    }

    // Test helper functions
    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    async fn create_test_node(datastore: &SqliteStore) -> Node {
        let mut node = Node::new(
            "test-polling-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ASR1000".to_string();
        node.lifecycle = Lifecycle::Live;
        datastore.create_node(&node).await.unwrap()
    }

    // Tests for polling_node function to cover uncovered lines

    #[tokio::test]
    async fn test_polling_node_status_action() {
        // Test lines 7-28 including Status case
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Status,
            detailed: false,
        };

        let result = polling_node(args, &datastore, OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_function_start_action() {
        // Test lines 7-34 including Start case
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Start,
            detailed: true,
        };

        let result = polling_node(args, &datastore, OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_function_stop_action() {
        // Test lines 7-40 including Stop case
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Stop,
            detailed: false,
        };

        let result = polling_node(args, &datastore, OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_function_restart_action() {
        // Test lines 7-46 including Restart case
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Restart,
            detailed: true,
        };

        let result = polling_node(args, &datastore, OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_function_history_action() {
        // Test lines 7-52 including History case
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::History,
            detailed: false,
        };

        let result = polling_node(args, &datastore, OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_detailed_flag() {
        // Test lines 7-19 with detailed = true
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Status,
            detailed: true, // Test detailed flag
        };

        let result = polling_node(args, &datastore, OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_yaml_output() {
        // Test lines 55-58 with YAML output format
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Status,
            detailed: false,
        };

        let result = polling_node(args, &datastore, OutputFormat::Yaml).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_table_output() {
        // Test lines 55-58 with Table output format
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let args = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Status,
            detailed: false,
        };

        let result = polling_node(args, &datastore, OutputFormat::Table).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_node_nonexistent_node() {
        // Test error handling when node doesn't exist (line 13)
        let datastore = setup_test_datastore().await;
        let nonexistent_id = Uuid::new_v4();

        let args = PollingNodeArgs {
            id: nonexistent_id,
            action: PollingAction::Status,
            detailed: false,
        };

        let result = polling_node(args, &datastore, OutputFormat::Json).await;
        assert!(result.is_err());
    }
}
