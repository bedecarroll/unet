//! Tests for `EvaluationContext` creation and field access

use super::super::context::*;
use serde_json::json;

#[test]
fn test_evaluation_context_new() {
    let node_data = json!({
        "name": "test-node",
        "vendor": "cisco"
    });

    let context = EvaluationContext::new(node_data.clone());

    assert_eq!(context.node_data, node_data);
    assert!(context.derived_data.is_none());
}

#[test]
fn test_evaluation_context_with_derived_data() {
    let node_data = json!({
        "name": "test-node",
        "vendor": "cisco"
    });

    let derived_data = json!({
        "uptime": 86400,
        "temperature": 45
    });

    let context = EvaluationContext::with_derived_data(node_data.clone(), derived_data.clone());

    assert_eq!(context.node_data, node_data);
    assert_eq!(context.derived_data, Some(derived_data));
}

#[test]
fn test_evaluation_context_get_field_simple() {
    let context = EvaluationContext::new(json!({
        "name": "test-node",
        "vendor": "cisco",
        "port_count": 24
    }));

    assert_eq!(context.get_field("name"), Some(&json!("test-node")));
    assert_eq!(context.get_field("vendor"), Some(&json!("cisco")));
    assert_eq!(context.get_field("port_count"), Some(&json!(24)));
    assert_eq!(context.get_field("non_existent"), None);
}

#[test]
fn test_evaluation_context_get_field_nested() {
    let context = EvaluationContext::new(json!({
        "device": {
            "info": {
                "vendor": "cisco",
                "model": "2960",
                "details": {
                    "port_count": 24,
                    "power_consumption": 15.5
                }
            }
        }
    }));

    assert_eq!(
        context.get_field("device.info.vendor"),
        Some(&json!("cisco"))
    );
    assert_eq!(context.get_field("device.info.model"), Some(&json!("2960")));
    assert_eq!(
        context.get_field("device.info.details.port_count"),
        Some(&json!(24))
    );
    assert_eq!(
        context.get_field("device.info.details.power_consumption"),
        Some(&json!(15.5))
    );

    // Test non-existent paths
    assert_eq!(context.get_field("device.info.non_existent"), None);
    assert_eq!(context.get_field("device.non_existent.vendor"), None);
    assert_eq!(context.get_field("non_existent.info.vendor"), None);
}

#[test]
fn test_evaluation_context_get_field_derived_data() {
    let node_data = json!({
        "name": "test-node",
        "vendor": "cisco"
    });

    let derived_data = json!({
        "monitoring": {
            "uptime": 86400,
            "cpu_usage": 45.2,
            "memory": {
                "total": 1024,
                "used": 512
            }
        }
    });

    let context = EvaluationContext::with_derived_data(node_data, derived_data);

    // Test derived data access with "derived." prefix
    assert_eq!(
        context.get_field("derived.monitoring.uptime"),
        Some(&json!(86400))
    );
    assert_eq!(
        context.get_field("derived.monitoring.cpu_usage"),
        Some(&json!(45.2))
    );
    assert_eq!(
        context.get_field("derived.monitoring.memory.total"),
        Some(&json!(1024))
    );
    assert_eq!(
        context.get_field("derived.monitoring.memory.used"),
        Some(&json!(512))
    );

    // Test non-existent derived paths
    assert_eq!(context.get_field("derived.non_existent"), None);
    assert_eq!(context.get_field("derived.monitoring.non_existent"), None);

    // Test that node data is still accessible
    assert_eq!(context.get_field("name"), Some(&json!("test-node")));
    assert_eq!(context.get_field("vendor"), Some(&json!("cisco")));
}

#[test]
fn test_evaluation_context_get_field_no_derived_data() {
    let context = EvaluationContext::new(json!({
        "name": "test-node"
    }));

    // Should return None for derived data when it doesn't exist
    assert_eq!(context.get_field("derived.anything"), None);
    assert_eq!(context.get_field("derived.monitoring.uptime"), None);
}

#[test]
fn test_evaluation_context_get_field_edge_cases() {
    let context = EvaluationContext::new(json!({
        "empty_object": {},
        "empty_array": [],
        "null_value": null,
        "empty_string": "",
        "zero": 0,
        "false_value": false
    }));

    assert_eq!(context.get_field("empty_object"), Some(&json!({})));
    assert_eq!(context.get_field("empty_array"), Some(&json!([])));
    assert_eq!(context.get_field("null_value"), Some(&json!(null)));
    assert_eq!(context.get_field("empty_string"), Some(&json!("")));
    assert_eq!(context.get_field("zero"), Some(&json!(0)));
    assert_eq!(context.get_field("false_value"), Some(&json!(false)));

    // Trying to access fields on non-object values should return None
    assert_eq!(context.get_field("empty_array.field"), None);
    assert_eq!(context.get_field("null_value.field"), None);
    assert_eq!(context.get_field("empty_string.field"), None);
}
