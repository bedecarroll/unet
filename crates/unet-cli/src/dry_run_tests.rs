use crate::dry_run::DryRunStore;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use unet_core::datastore::types::{HistoryQueryOptions, PagedResult, QueryOptions};
use unet_core::datastore::{BatchOperation, DataStore, MockDataStore, testing::ready_ok};
use unet_core::models::derived::{NodeStatus, PerformanceMetrics};
use unet_core::models::{DeviceRole, Link, Location, Node, NodeBuilder, Vendor};
use unet_core::policy::{Action, Condition, FieldRef, PolicyExecutionResult};
use unet_core::policy::{PolicyRule, Value as PolicyValue};
use uuid::Uuid;

fn make_node() -> Node {
    NodeBuilder::new()
        .id(Uuid::new_v4())
        .name("edge-1")
        .domain("example.com")
        .vendor(Vendor::Cisco)
        .model("ISR4321")
        .role(DeviceRole::Router)
        .build()
        .unwrap()
}

fn make_policy_result() -> PolicyExecutionResult {
    PolicyExecutionResult::new_error(
        PolicyRule {
            id: Some("rule-1".to_string()),
            condition: Condition::True,
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["node".to_string(), "name".to_string()],
                },
                expected: PolicyValue::String("edge-1".to_string()),
            },
        },
        "boom".to_string(),
    )
}

#[tokio::test]
async fn test_dry_run_store_covers_forwarded_read_and_history_paths() {
    let node = make_node();
    let other_node = make_node();
    let link = Link::new(
        "wan-link".to_string(),
        node.id,
        "Gi0/0".to_string(),
        other_node.id,
        "Gi0/1".to_string(),
    );
    let location = Location::new_root("HQ".to_string(), "building".to_string());
    let options = QueryOptions {
        filters: Vec::new(),
        sort: Vec::new(),
        pagination: None,
    };
    let history_options = HistoryQueryOptions {
        limit: 5,
        since: None,
    };
    let status = NodeStatus::new(node.id);
    let interfaces = status.interfaces.clone();
    let metrics = Some(PerformanceMetrics {
        cpu_utilization: Some(55),
        memory_utilization: Some(70),
        total_memory: Some(8192),
        used_memory: Some(4096),
        load_average: Some(0.5),
    });
    let history = vec![status.clone()];
    let policy_result = make_policy_result();
    let entity_counts = HashMap::from([("nodes".to_string(), 1usize)]);
    let statistics = HashMap::from([("nodes_total".to_string(), json!(1))]);
    let rule_results = vec![(node.id, policy_result.clone())];

    let mut mock = MockDataStore::new();
    mock.expect_health_check().returning(|| ready_ok(()));
    mock.expect_get_node().times(2).returning({
        let node = node.clone();
        move |_| ready_ok(Some(node.clone()))
    });
    mock.expect_list_nodes().returning({
        let node = node.clone();
        move |_| ready_ok(PagedResult::new(vec![node.clone()], 1, None))
    });
    mock.expect_get_nodes_by_location().returning({
        let node = node.clone();
        move |_| ready_ok(vec![node.clone()])
    });
    mock.expect_search_nodes_by_name().returning({
        let node = node.clone();
        move |_| ready_ok(vec![node.clone()])
    });
    mock.expect_get_link().returning({
        let link = link.clone();
        move |_| ready_ok(Some(link.clone()))
    });
    mock.expect_list_links().returning({
        let link = link.clone();
        move |_| ready_ok(PagedResult::new(vec![link.clone()], 1, None))
    });
    mock.expect_get_links_for_node().returning({
        let link = link.clone();
        move |_| ready_ok(vec![link.clone()])
    });
    mock.expect_get_links_between_nodes().returning({
        let link = link.clone();
        move |_, _| ready_ok(vec![link.clone()])
    });
    mock.expect_get_location().returning({
        let location = location.clone();
        move |_| ready_ok(Some(location.clone()))
    });
    mock.expect_list_locations().returning({
        let location = location.clone();
        move |_| ready_ok(PagedResult::new(vec![location.clone()], 1, None))
    });
    mock.expect_list_vendors()
        .returning(|| ready_ok(vec!["Cisco".to_string()]));
    mock.expect_get_entity_counts().returning({
        let entity_counts = entity_counts.clone();
        move || ready_ok(entity_counts.clone())
    });
    mock.expect_get_statistics().returning({
        let statistics = statistics.clone();
        move || ready_ok(statistics.clone())
    });
    mock.expect_get_node_status().returning({
        let status = status.clone();
        move |_| ready_ok(Some(status.clone()))
    });
    mock.expect_get_node_interfaces().returning({
        let interfaces = interfaces.clone();
        move |_| ready_ok(interfaces.clone())
    });
    mock.expect_get_node_metrics().returning({
        let metrics = metrics.clone();
        move |_| ready_ok(metrics.clone())
    });
    mock.expect_store_node_status_snapshot()
        .returning(|_| ready_ok(()));
    mock.expect_get_node_status_history().returning({
        let history = history.clone();
        move |_, options| {
            assert_eq!(options.limit, 5);
            ready_ok(history.clone())
        }
    });
    mock.expect_get_policy_results().returning({
        let policy_result = policy_result.clone();
        move |_| ready_ok(vec![policy_result.clone()])
    });
    mock.expect_get_latest_policy_results().returning({
        let policy_result = policy_result.clone();
        move |_| ready_ok(vec![policy_result.clone()])
    });
    mock.expect_get_rule_results().returning({
        let rule_results = rule_results.clone();
        move |_| ready_ok(rule_results.clone())
    });
    mock.expect_get_nodes_for_policy_evaluation().returning({
        let node = node.clone();
        move || ready_ok(vec![node.clone()])
    });

    let store = DryRunStore::new(Arc::new(mock));

    assert_eq!(store.name(), "dry-run");
    store.health_check().await.unwrap();
    assert_eq!(store.get_node(&node.id).await.unwrap(), Some(node.clone()));
    assert_eq!(
        store.list_nodes(&options).await.unwrap().items,
        vec![node.clone()]
    );
    assert_eq!(
        store.get_nodes_by_location(&location.id).await.unwrap(),
        vec![node.clone()]
    );
    assert_eq!(
        store.search_nodes_by_name("edge").await.unwrap(),
        vec![node.clone()]
    );
    assert_eq!(store.create_link(&link).await.unwrap(), link.clone());
    assert_eq!(store.get_link(&link.id).await.unwrap(), Some(link.clone()));
    assert_eq!(
        store.list_links(&options).await.unwrap().items,
        vec![link.clone()]
    );
    assert_eq!(store.update_link(&link).await.unwrap(), link.clone());
    assert!(store.delete_link(&link.id).await.is_ok());
    assert_eq!(
        store.get_links_for_node(&node.id).await.unwrap(),
        vec![link.clone()]
    );
    assert_eq!(
        store
            .get_links_between_nodes(&node.id, &other_node.id)
            .await
            .unwrap(),
        vec![link.clone()]
    );
    assert_eq!(
        store.create_location(&location).await.unwrap(),
        location.clone()
    );
    assert_eq!(
        store.get_location(&location.id).await.unwrap(),
        Some(location.clone())
    );
    assert_eq!(
        store.list_locations(&options).await.unwrap().items,
        vec![location.clone()]
    );
    assert_eq!(
        store.update_location(&location).await.unwrap(),
        location.clone()
    );
    assert!(store.delete_location(&location.id).await.is_ok());
    assert!(store.create_vendor("Cisco").await.is_ok());
    assert_eq!(
        store.list_vendors().await.unwrap(),
        vec!["Cisco".to_string()]
    );
    assert!(store.delete_vendor("Cisco").await.is_ok());
    assert_eq!(
        store
            .batch_nodes(&[BatchOperation::Insert(node.clone())])
            .await
            .unwrap()
            .success_count,
        1
    );
    assert_eq!(
        store
            .batch_links(&[BatchOperation::Insert(link.clone())])
            .await
            .unwrap()
            .success_count,
        1
    );
    assert_eq!(
        store
            .batch_locations(&[BatchOperation::Insert(location.clone())])
            .await
            .unwrap()
            .success_count,
        1
    );
    assert_eq!(store.get_entity_counts().await.unwrap(), entity_counts);
    assert_eq!(store.get_statistics().await.unwrap(), statistics);
    assert_eq!(
        store.get_node_status(&node.id).await.unwrap(),
        Some(status.clone())
    );
    assert_eq!(
        store.get_node_interfaces(&node.id).await.unwrap(),
        interfaces
    );
    assert_eq!(store.get_node_metrics(&node.id).await.unwrap(), metrics);
    assert!(store.store_node_status_snapshot(&status).await.is_ok());
    assert_eq!(
        store
            .get_node_status_history(&node.id, &history_options)
            .await
            .unwrap(),
        history
    );
    assert!(
        store
            .store_policy_result(&node.id, "rule-1", &policy_result)
            .await
            .is_ok()
    );
    let policy_results = store.get_policy_results(&node.id).await.unwrap();
    assert_eq!(policy_results.len(), 1);
    assert_eq!(policy_results[0].rule.id.as_deref(), Some("rule-1"));
    let latest_policy_results = store.get_latest_policy_results(&node.id).await.unwrap();
    assert_eq!(latest_policy_results.len(), 1);
    assert_eq!(latest_policy_results[0].rule.id.as_deref(), Some("rule-1"));
    let fetched_rule_results = store.get_rule_results("rule-1").await.unwrap();
    assert_eq!(fetched_rule_results.len(), 1);
    assert_eq!(fetched_rule_results[0].0, node.id);
    assert_eq!(fetched_rule_results[0].1.rule.id.as_deref(), Some("rule-1"));
    assert!(
        store
            .update_node_custom_data(&node.id, &json!({"role": "edge"}))
            .await
            .is_ok()
    );
    assert_eq!(
        store.get_nodes_for_policy_evaluation().await.unwrap(),
        vec![node]
    );
}
