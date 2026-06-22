/// Output-builder tests for diff-only compare payloads.
#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use unet_core::models::DeviceRole;
    use unet_core::models::derived::PerformanceMetrics;
    use uuid::Uuid;

    use super::super::compare::build_compare_output;
    use super::super::types::{CompareNodeArgs, CompareType};
    use crate::commands::nodes::compare_test_support::helpers::make_node;

    #[tokio::test]
    async fn test_build_compare_output_diff_only_filters_identical_metric_entries() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();
        let first_node = make_node(first_node_id, "edge-a", "ISR4321", DeviceRole::Router);
        let second_node = make_node(second_node_id, "edge-b", "C9300", DeviceRole::Switch);
        let mut store = MockDataStore::new();

        store
            .expect_get_node_required()
            .with(eq(first_node_id))
            .returning(move |_| ready_ok(first_node.clone()));
        store
            .expect_get_node_required()
            .with(eq(second_node_id))
            .returning(move |_| ready_ok(second_node.clone()));
        store
            .expect_get_node_metrics()
            .with(eq(first_node_id))
            .returning(|_| {
                ready_ok(Some(PerformanceMetrics {
                    cpu_utilization: Some(15),
                    memory_utilization: Some(60),
                    total_memory: Some(1_024),
                    used_memory: Some(600),
                    load_average: Some(0.4),
                }))
            });
        store
            .expect_get_node_metrics()
            .with(eq(second_node_id))
            .returning(|_| {
                ready_ok(Some(PerformanceMetrics {
                    cpu_utilization: Some(15),
                    memory_utilization: Some(75),
                    total_memory: Some(1_024),
                    used_memory: Some(768),
                    load_average: Some(0.4),
                }))
            });

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::Metrics],
            diff_only: true,
        };

        let output = build_compare_output(args, &store).await.unwrap();
        let metrics = &output["metrics_comparison"];
        let entries = metrics["entries"].as_array().unwrap();

        assert_eq!(metrics["compared_field_count"], 5);
        assert_eq!(metrics["difference_count"], 2);
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .all(|entry| entry["different"].as_bool().unwrap())
        );
        assert!(
            entries
                .iter()
                .all(|entry| entry["field"] != "cpu_utilization")
        );
        assert!(entries.iter().all(|entry| entry["field"] != "load_average"));
    }
}
