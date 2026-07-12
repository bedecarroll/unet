//! Shared test utilities for mock datastore tests

use crate::datastore::{Pagination, QueryOptions};
use crate::models::{DeviceRole, Link, Location, Node, Vendor};
use crate::policy::{Action, Condition, FieldRef, PolicyExecutionResult, PolicyRule, Value};
use uuid::Uuid;

pub fn create_test_node() -> Node {
    Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    )
}

pub fn create_test_link() -> Link {
    let node_a = Uuid::new_v4();
    let node_b = Uuid::new_v4();
    Link::new(
        "test-link".to_string(),
        node_a,
        "eth0".to_string(),
        node_b,
        "eth1".to_string(),
    )
}

pub fn create_test_location() -> Location {
    Location::new_root("test-location".to_string(), "datacenter".to_string())
}

pub fn create_query_options() -> QueryOptions {
    QueryOptions {
        filters: vec![],
        sort: vec![],
        pagination: Some(Pagination::page(1, 10).unwrap()),
    }
}

pub fn create_test_policy_result() -> PolicyExecutionResult {
    let rule = PolicyRule {
        id: Some("test-rule".to_string()),
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["status".to_string()],
            },
            expected: Value::String("active".to_string()),
        },
    };

    PolicyExecutionResult::new(
        rule,
        crate::policy::EvaluationResult::Satisfied {
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["status".to_string()],
                },
                expected: Value::String("active".to_string()),
            },
        },
        None,
    )
}
