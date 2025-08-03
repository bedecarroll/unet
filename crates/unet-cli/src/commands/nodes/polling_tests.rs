/// Tests for node polling functionality
#[cfg(test)]
mod tests {
    use crate::commands::nodes::types::*;
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
}
