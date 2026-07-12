/// Tests for advanced node operations (compare, polling, history)
#[cfg(test)]
mod integration_tests {
    use crate::commands::nodes::types::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_arguments_structure_validation() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();

        // Test CompareNodeArgs construction
        let compare_args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        // Test PollingNodeArgs construction
        let polling_args = PollingNodeArgs {
            id: first_node_id,
            action: PollingAction::Status,
            detailed: false,
        };

        // Test HistoryNodeArgs construction
        let history_args = HistoryNodeArgs {
            id: first_node_id,
            history_type: HistoryType::Status,
            limit: 10,
            last_hours: None,
            detailed: false,
        };

        // Verify all argument types are properly structured
        assert_eq!(compare_args.node_a, first_node_id);
        assert_eq!(polling_args.id, first_node_id);
        assert_eq!(history_args.id, first_node_id);
    }

    #[tokio::test]
    async fn test_enum_variant_matching() {
        // Test CompareType enum variants
        let compare_types = vec![
            CompareType::All,
            CompareType::Interfaces,
            CompareType::Metrics,
            CompareType::System,
        ];

        // Test that all compare types are properly constructed
        assert_eq!(compare_types.len(), 4);
        // Verify all variants can be matched
        for compare_type in compare_types {
            match compare_type {
                CompareType::All
                | CompareType::Interfaces
                | CompareType::Metrics
                | CompareType::System => {}
            }
        }

        // Test PollingAction enum variants
        let polling_actions = vec![
            PollingAction::Status,
            PollingAction::Start,
            PollingAction::Stop,
            PollingAction::Restart,
            PollingAction::History,
        ];

        // Test that all polling actions are properly constructed
        assert_eq!(polling_actions.len(), 5);
        // Verify all variants can be matched
        for polling_action in polling_actions {
            match polling_action {
                PollingAction::Status
                | PollingAction::Start
                | PollingAction::Stop
                | PollingAction::Restart
                | PollingAction::History => {}
            }
        }

        // Test HistoryType enum variants
        let history_types = vec![
            HistoryType::Status,
            HistoryType::Interfaces,
            HistoryType::Metrics,
            HistoryType::System,
            HistoryType::All,
        ];

        // Test that all history types are properly constructed
        assert_eq!(history_types.len(), 5);
        // Verify all variants can be matched
        for history_type in history_types {
            match history_type {
                HistoryType::Status
                | HistoryType::Interfaces
                | HistoryType::Metrics
                | HistoryType::System
                | HistoryType::All => {}
            }
        }
    }

    #[tokio::test]
    async fn test_option_handling() {
        let node_id = Uuid::new_v4();

        // Test None values
        let args_with_none = CompareNodeArgs {
            node_a: node_id,
            node_b: None,
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        assert_eq!(args_with_none.node_b, None);

        // Test Some values
        let other_node_id = Uuid::new_v4();
        let args_with_some = CompareNodeArgs {
            node_a: node_id,
            node_b: Some(other_node_id),
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        assert_eq!(args_with_some.node_b, Some(other_node_id));

        // Test HistoryNodeArgs with Some/None last_hours
        let history_none = HistoryNodeArgs {
            id: node_id,
            history_type: HistoryType::All,
            limit: 10,
            last_hours: None,
            detailed: false,
        };

        let history_some = HistoryNodeArgs {
            id: node_id,
            history_type: HistoryType::All,
            limit: 10,
            last_hours: Some(24),
            detailed: false,
        };

        assert_eq!(history_none.last_hours, None);
        assert_eq!(history_some.last_hours, Some(24));
    }

    #[tokio::test]
    async fn test_boolean_combinations() {
        let node_id = Uuid::new_v4();

        // Test all boolean combinations for CompareNodeArgs
        let combinations = vec![(true, true), (true, false), (false, true), (false, false)];

        for (diff_only, detailed) in combinations {
            let compare_args = CompareNodeArgs {
                node_a: node_id,
                node_b: Some(node_id),
                compare_type: vec![CompareType::All],
                diff_only,
            };

            let polling_args = PollingNodeArgs {
                id: node_id,
                action: PollingAction::Status,
                detailed,
            };

            let history_args = HistoryNodeArgs {
                id: node_id,
                history_type: HistoryType::Status,
                limit: 10,
                last_hours: None,
                detailed,
            };

            assert_eq!(compare_args.diff_only, diff_only);
            assert_eq!(polling_args.detailed, detailed);
            assert_eq!(history_args.detailed, detailed);
        }
    }

    #[tokio::test]
    async fn test_vector_operations() {
        let node_id = Uuid::new_v4();

        // Test empty vector
        let empty_compare_types = CompareNodeArgs {
            node_a: node_id,
            node_b: Some(node_id),
            compare_type: vec![],
            diff_only: false,
        };

        assert!(empty_compare_types.compare_type.is_empty());
        assert_eq!(empty_compare_types.compare_type.len(), 0);

        // Test single element vector
        let single_compare_types = CompareNodeArgs {
            node_a: node_id,
            node_b: Some(node_id),
            compare_type: vec![CompareType::Metrics],
            diff_only: false,
        };

        assert_eq!(single_compare_types.compare_type.len(), 1);
        assert_eq!(single_compare_types.compare_type[0], CompareType::Metrics);

        // Test multiple element vector
        let multiple_compare_types = CompareNodeArgs {
            node_a: node_id,
            node_b: Some(node_id),
            compare_type: vec![
                CompareType::All,
                CompareType::Interfaces,
                CompareType::Metrics,
                CompareType::System,
            ],
            diff_only: false,
        };

        assert_eq!(multiple_compare_types.compare_type.len(), 4);
        assert!(
            multiple_compare_types
                .compare_type
                .contains(&CompareType::All)
        );
        assert!(
            multiple_compare_types
                .compare_type
                .contains(&CompareType::Interfaces)
        );
        assert!(
            multiple_compare_types
                .compare_type
                .contains(&CompareType::Metrics)
        );
        assert!(
            multiple_compare_types
                .compare_type
                .contains(&CompareType::System)
        );
    }
}
