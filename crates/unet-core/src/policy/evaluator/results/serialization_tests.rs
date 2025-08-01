//! Tests for serialization of policy evaluation result types

use crate::policy::evaluator::results::{
    AggregatedResult, BatchStatistics, ComplianceFailureDetail,
};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_compliance_failure_detail_serialization() {
    let detail = ComplianceFailureDetail {
        rule_name: "test_rule".to_string(),
        field: "version".to_string(),
        expected: json!("2.0"),
        actual: json!("1.0"),
    };

    let serialized = serde_json::to_string(&detail).unwrap();
    let deserialized: ComplianceFailureDetail = serde_json::from_str(&serialized).unwrap();

    assert_eq!(detail.rule_name, deserialized.rule_name);
    assert_eq!(detail.field, deserialized.field);
    assert_eq!(detail.expected, deserialized.expected);
    assert_eq!(detail.actual, deserialized.actual);
}

#[test]
fn test_batch_statistics_creation() {
    let stats = BatchStatistics {
        total_nodes: 5,
        total_rules_evaluated: 25,
        total_execution_time: Duration::from_millis(500),
        avg_execution_time_per_node: Duration::from_millis(100),
        nodes_with_failures: 2,
        nodes_with_errors: 1,
        overall_success_rate: 80.0,
    };

    assert_eq!(stats.total_nodes, 5);
    assert_eq!(stats.total_rules_evaluated, 25);
    assert_eq!(stats.total_execution_time, Duration::from_millis(500));
    assert_eq!(
        stats.avg_execution_time_per_node,
        Duration::from_millis(100)
    );
    assert_eq!(stats.nodes_with_failures, 2);
    assert_eq!(stats.nodes_with_errors, 1);
    assert!((stats.overall_success_rate - 80.0).abs() < f64::EPSILON);
}

#[test]
fn test_batch_statistics_serialization() {
    let stats = BatchStatistics::default();

    let serialized = serde_json::to_string(&stats).unwrap();
    let deserialized: BatchStatistics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(stats.total_nodes, deserialized.total_nodes);
    assert_eq!(
        stats.total_rules_evaluated,
        deserialized.total_rules_evaluated
    );
    assert_eq!(
        stats.total_execution_time,
        deserialized.total_execution_time
    );
    assert_eq!(
        stats.avg_execution_time_per_node,
        deserialized.avg_execution_time_per_node
    );
    assert_eq!(stats.nodes_with_failures, deserialized.nodes_with_failures);
    assert_eq!(stats.nodes_with_errors, deserialized.nodes_with_errors);
    assert!((stats.overall_success_rate - deserialized.overall_success_rate).abs() < f64::EPSILON);
}

#[test]
fn test_aggregated_result_serialization() {
    let node_id = Uuid::new_v4();
    let batch_id = "serialization_test".to_string();

    let aggregated =
        AggregatedResult::from_results(node_id, batch_id, vec![], Duration::from_millis(100));

    let serialized = serde_json::to_string(&aggregated).unwrap();
    let deserialized: AggregatedResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(aggregated.node_id, deserialized.node_id);
    assert_eq!(aggregated.batch_id, deserialized.batch_id);
    assert_eq!(aggregated.total_rules, deserialized.total_rules);
    assert_eq!(aggregated.satisfied_rules, deserialized.satisfied_rules);
    assert_eq!(aggregated.failed_rules, deserialized.failed_rules);
    assert_eq!(aggregated.error_rules, deserialized.error_rules);
    assert_eq!(
        aggregated.compliance_failures,
        deserialized.compliance_failures
    );
    assert_eq!(
        aggregated.execution_duration,
        deserialized.execution_duration
    );
    assert_eq!(aggregated.summary, deserialized.summary);
}
