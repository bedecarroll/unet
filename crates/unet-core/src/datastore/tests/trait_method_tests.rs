//! Tests for default trait method implementations

use super::mock_datastore_tests::MockDataStore;
use crate::datastore::{DataStore, types::DataStoreError};
use crate::models::{DeviceRole, Link, Location, Node, Vendor};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_get_node_required_success() {
    let node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    let node_id = node.id;
    let datastore = MockDataStore::new().with_node(node.clone());

    let result = datastore.get_node_required(&node_id).await;

    assert!(result.is_ok());
    let retrieved_node = result.unwrap();
    assert_eq!(retrieved_node.id, node_id);
    assert_eq!(retrieved_node.name, "test-node");
}

#[tokio::test]
async fn test_get_node_required_not_found() {
    let datastore = MockDataStore::new();
    let non_existent_id = Uuid::new_v4();

    let result = datastore.get_node_required(&non_existent_id).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "Node");
            assert_eq!(id, non_existent_id.to_string());
        }
        other => panic!("Expected NotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_get_link_required_success() {
    let node_a = Node::new(
        "node-a".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    let node_b = Node::new(
        "node-b".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Switch,
    );
    let link = Link::new(
        "test-link".to_string(),
        node_a.id,
        "eth0".to_string(),
        node_b.id,
        "eth1".to_string(),
    );
    let link_id = link.id;
    let datastore = MockDataStore::new().with_link(link.clone());

    let result = datastore.get_link_required(&link_id).await;

    assert!(result.is_ok());
    let retrieved_link = result.unwrap();
    assert_eq!(retrieved_link.id, link_id);
    assert_eq!(retrieved_link.name, "test-link");
}

#[tokio::test]
async fn test_get_link_required_not_found() {
    let datastore = MockDataStore::new();
    let non_existent_id = Uuid::new_v4();

    let result = datastore.get_link_required(&non_existent_id).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "Link");
            assert_eq!(id, non_existent_id.to_string());
        }
        other => panic!("Expected NotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_get_location_required_success() {
    let location = Location::new_root("datacenter-1".to_string(), "DataCenter".to_string());
    let location_id = location.id;
    let datastore = MockDataStore::new().with_location(location.clone());

    let result = datastore.get_location_required(&location_id).await;

    assert!(result.is_ok());
    let retrieved_location = result.unwrap();
    assert_eq!(retrieved_location.id, location_id);
    assert_eq!(retrieved_location.name, "datacenter-1");
}

#[tokio::test]
async fn test_get_location_required_not_found() {
    let datastore = MockDataStore::new();
    let non_existent_id = Uuid::new_v4();

    let result = datastore.get_location_required(&non_existent_id).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "Location");
            assert_eq!(id, non_existent_id.to_string());
        }
        other => panic!("Expected NotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_get_node_status_default_implementation() {
    let datastore = MockDataStore::new();
    let node_id = Uuid::new_v4();

    let result = datastore.get_node_status(&node_id).await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert!(status.is_some());
    let status = status.unwrap();
    assert_eq!(status.node_id, node_id);
}

#[tokio::test]
async fn test_get_node_interfaces_default_implementation() {
    let datastore = MockDataStore::new();
    let node_id = Uuid::new_v4();

    let result = datastore.get_node_interfaces(&node_id).await;

    assert!(result.is_ok());
    let interfaces = result.unwrap();
    assert!(interfaces.is_empty());
}

#[tokio::test]
async fn test_get_node_metrics_default_implementation() {
    let datastore = MockDataStore::new();
    let node_id = Uuid::new_v4();

    let result = datastore.get_node_metrics(&node_id).await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.is_none());
}

#[tokio::test]
async fn test_store_policy_result_default_implementation() {
    let datastore = MockDataStore::new();
    let node_id = Uuid::new_v4();
    let rule_id = "test-rule";
    let rule = crate::policy::PolicyRule {
        id: Some(rule_id.to_string()),
        condition: crate::policy::Condition::True,
        action: crate::policy::Action::Assert {
            field: crate::policy::FieldRef {
                path: vec!["test".to_string()],
            },
            expected: crate::policy::Value::String("value".to_string()),
        },
    };
    let result_data = crate::policy::PolicyExecutionResult::new(
        rule,
        crate::policy::EvaluationResult::Satisfied {
            action: crate::policy::Action::Assert {
                field: crate::policy::FieldRef {
                    path: vec!["test".to_string()],
                },
                expected: crate::policy::Value::String("value".to_string()),
            },
        },
        None,
    );

    let result = datastore
        .store_policy_result(&node_id, rule_id, &result_data)
        .await;

    // Default implementation should succeed (no-op)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_policy_results_default_implementation() {
    let datastore = MockDataStore::new();
    let node_id = Uuid::new_v4();

    let result = datastore.get_policy_results(&node_id).await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_get_latest_policy_results_default_implementation() {
    let datastore = MockDataStore::new();
    let node_id = Uuid::new_v4();

    let result = datastore.get_latest_policy_results(&node_id).await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_get_rule_results_default_implementation() {
    let datastore = MockDataStore::new();
    let rule_id = "test-rule";

    let result = datastore.get_rule_results(rule_id).await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_update_node_custom_data_success() {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.custom_data = json!({"original": "data"});
    let node_id = node.id;

    let datastore = MockDataStore::new().with_node(node);

    let new_custom_data = json!({"updated": "data", "config": {"vlan": 100}});
    let result = datastore
        .update_node_custom_data(&node_id, &new_custom_data)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_custom_data_node_not_found() {
    let datastore = MockDataStore::new();
    let non_existent_id = Uuid::new_v4();
    let custom_data = json!({"test": "data"});

    let result = datastore
        .update_node_custom_data(&non_existent_id, &custom_data)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "Node");
            assert_eq!(id, non_existent_id.to_string());
        }
        other => panic!("Expected NotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_get_nodes_for_policy_evaluation_default_implementation() {
    let node1 = Node::new(
        "node1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    let node2 = Node::new(
        "node2".to_string(),
        "example.com".to_string(),
        Vendor::Juniper,
        DeviceRole::Switch,
    );

    let datastore = MockDataStore::new()
        .with_node(node1.clone())
        .with_node(node2.clone());

    let result = datastore.get_nodes_for_policy_evaluation().await;

    assert!(result.is_ok());
    let all_nodes = result.unwrap();
    assert_eq!(all_nodes.len(), 2);

    let node_ids: std::collections::HashSet<Uuid> = all_nodes.iter().map(|n| n.id).collect();
    assert!(node_ids.contains(&node1.id));
    assert!(node_ids.contains(&node2.id));
}
