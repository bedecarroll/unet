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
