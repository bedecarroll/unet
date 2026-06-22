/// Output-builder tests for full compare payloads.
#[cfg(test)]
mod tests {
    use unet_core::datastore::MockDataStore;
    use unet_core::models::DeviceRole;
    use uuid::Uuid;

    use super::super::compare::build_compare_output;
    use super::super::types::{CompareNodeArgs, CompareType};
    use crate::commands::nodes::compare_test_support::helpers::{
        configure_all_compare_store, find_entry, make_node,
    };

    #[tokio::test]
    async fn test_build_compare_output_all_reports_real_differences() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();
        let first_node = make_node(first_node_id, "edge-a", "ISR4321", DeviceRole::Router);
        let second_node = make_node(second_node_id, "edge-b", "C9300", DeviceRole::Switch);
        let store = configure_all_compare_store(&first_node, &second_node);

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        let output = build_compare_output(args, &store).await.unwrap();
        let serialized = serde_json::to_string(&output).unwrap();

        assert!(!serialized.contains("not yet implemented"));
        assert!(output.get("basic_comparison").is_some());
        assert!(output.get("interfaces_comparison").is_some());
        assert!(output.get("metrics_comparison").is_some());
        assert!(output.get("system_comparison").is_some());

        let basic_model = find_entry(&output["basic_comparison"], "node", "model");
        assert_eq!(basic_model["node_a"], "ISR4321");
        assert_eq!(basic_model["node_b"], "C9300");
        assert!(basic_model["different"].as_bool().unwrap());

        let interface_presence = find_entry(&output["interfaces_comparison"], "dmz0", "present");
        assert_eq!(interface_presence["node_a"], false);
        assert_eq!(interface_presence["node_b"], true);

        let interface_state = find_entry(&output["interfaces_comparison"], "wan0", "oper_status");
        assert_eq!(interface_state["node_a"], "Up");
        assert_eq!(interface_state["node_b"], "Down");

        let metric_entry = find_entry(
            &output["metrics_comparison"],
            "metrics",
            "memory_utilization",
        );
        assert_eq!(metric_entry["node_a"], 60);
        assert_eq!(metric_entry["node_b"], 75);

        let system_entry = find_entry(&output["system_comparison"], "system", "location");
        assert_eq!(system_entry["node_a"], "DC1");
        assert_eq!(system_entry["node_b"], "DC2");
    }

    #[tokio::test]
    async fn test_build_compare_output_rejects_historical_comparison() {
        let args = CompareNodeArgs {
            node_a: Uuid::new_v4(),
            node_b: None,
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        let result = build_compare_output(args, &MockDataStore::new()).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Historical comparison is not supported")
        );
    }
}
