//! Grammar construct tests - comprehensive coverage of all grammar features

use super::*;

#[test]
fn test_all_comparison_operators() {
    let context = EvaluationContext::new(json!({
        "node": {
            "port_count": 24,
            "name": "router-01",
            "version": "15.1.3"
        }
    }));

    // Test all comparison operators
    let operators_and_values = vec![
        (ComparisonOperator::Equal, Value::Number(24.0), true),
        (ComparisonOperator::NotEqual, Value::Number(48.0), true),
        (ComparisonOperator::LessThan, Value::Number(48.0), true),
        (
            ComparisonOperator::LessThanOrEqual,
            Value::Number(24.0),
            true,
        ),
        (ComparisonOperator::GreaterThan, Value::Number(12.0), true),
        (
            ComparisonOperator::GreaterThanOrEqual,
            Value::Number(24.0),
            true,
        ),
    ];

    for (operator, value, expected) in operators_and_values {
        let condition = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "port_count".to_string()],
            },
            operator: operator.clone(),
            value,
        };

        let rule = PolicyRule {
            id: Some("operator-test".to_string()),
            condition,
            action: Action::Set {
                field: FieldRef {
                    path: vec!["custom_data".to_string(), "test".to_string()],
                },
                value: Value::String("test".to_string()),
            },
        };

        let result = PolicyEvaluator::evaluate_rule(&rule, &context).unwrap();
        let is_satisfied = matches!(result, EvaluationResult::Satisfied { .. });
        assert_eq!(is_satisfied, expected, "Failed for operator {operator:?}");
    }
}

#[test]
fn test_string_operations() {
    let context = EvaluationContext::new(json!({
        "node": {
            "name": "router-01-cisco-2960",
            "description": "Main distribution router"
        }
    }));

    // Test CONTAINS operator
    let contains_condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "name".to_string()],
        },
        operator: ComparisonOperator::Contains,
        value: Value::String("cisco".to_string()),
    };

    let rule = PolicyRule {
        id: Some("contains-test".to_string()),
        condition: contains_condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "contains_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&rule, &context).unwrap();
    assert!(matches!(result, EvaluationResult::Satisfied { .. }));

    // Test MATCHES operator with regex
    let regex_condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "name".to_string()],
        },
        operator: ComparisonOperator::Matches,
        value: Value::Regex(r"router-\d+-\w+-\d+".to_string()),
    };

    let rule2 = PolicyRule {
        id: Some("regex-test".to_string()),
        condition: regex_condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "regex_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result2 = PolicyEvaluator::evaluate_rule(&rule2, &context).unwrap();
    assert!(matches!(result2, EvaluationResult::Satisfied { .. }));
}

#[test]
fn test_complex_boolean_expressions() {
    let context = EvaluationContext::new(json!({
        "node": {
            "vendor": "cisco",
            "model": "2960",
            "port_count": 24,
            "location": "datacenter"
        }
    }));

    // Test complex AND condition
    let vendor_condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "vendor".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("cisco".to_string()),
    };

    let model_condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "model".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("2960".to_string()),
    };

    let and_condition = Condition::And(Box::new(vendor_condition), Box::new(model_condition));

    let rule = PolicyRule {
        id: Some("and-test".to_string()),
        condition: and_condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "and_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&rule, &context).unwrap();
    assert!(matches!(result, EvaluationResult::Satisfied { .. }));

    // Test complex OR condition
    let wrong_vendor_condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "vendor".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("juniper".to_string()),
    };

    let correct_location_condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "location".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("datacenter".to_string()),
    };

    let or_condition = Condition::Or(
        Box::new(wrong_vendor_condition),
        Box::new(correct_location_condition),
    );

    let rule2 = PolicyRule {
        id: Some("or-test".to_string()),
        condition: or_condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "or_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result2 = PolicyEvaluator::evaluate_rule(&rule2, &context).unwrap();
    assert!(matches!(result2, EvaluationResult::Satisfied { .. }));

    // Test NOT condition
    let not_condition = Condition::Not(Box::new(Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "vendor".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("juniper".to_string()),
    }));

    let rule3 = PolicyRule {
        id: Some("not-test".to_string()),
        condition: not_condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "not_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result3 = PolicyEvaluator::evaluate_rule(&rule3, &context).unwrap();
    assert!(matches!(result3, EvaluationResult::Satisfied { .. }));
}

#[test]
fn test_all_action_types() {
    let context = EvaluationContext::new(json!({
        "node": {
            "vendor": "cisco",
            "model": "2960"
        }
    }));

    // Test SET action
    let set_rule = PolicyRule {
        id: Some("set-test".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "set_test".to_string()],
            },
            value: Value::String("completed".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&set_rule, &context).unwrap();
    assert!(matches!(result, EvaluationResult::Satisfied { .. }));

    // Test ASSERT action
    let assert_rule = PolicyRule {
        id: Some("assert-test".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            expected: Value::String("cisco".to_string()),
        },
    };

    let result2 = PolicyEvaluator::evaluate_rule(&assert_rule, &context).unwrap();
    assert!(matches!(result2, EvaluationResult::Satisfied { .. }));

    // Test APPLY action
    let apply_rule = PolicyRule {
        id: Some("apply-test".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "model".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("2960".to_string()),
        },
        action: Action::ApplyTemplate {
            template_path: "templates/switch-config.j2".to_string(),
        },
    };

    let result3 = PolicyEvaluator::evaluate_rule(&apply_rule, &context).unwrap();
    assert!(matches!(result3, EvaluationResult::Satisfied { .. }));
}

#[test]
fn test_complete_policy_file_parsing() {
    let policy_text = r#"
// This is a comprehensive test policy file
WHEN node.vendor == "cisco" AND node.model CONTAINS "2960" THEN SET custom_data.switch_type TO "access"

WHEN node.vendor == "juniper" THEN ASSERT node.vendor IS "juniper"

WHEN node.role == "Router" AND node.custom_data.location.datacenter == "DC1" THEN APPLY "templates/router-base.j2"

WHEN node.management_ip IS NOT NULL THEN SET custom_data.monitoring.enabled TO true

WHEN node.lifecycle == "Decommissioned" THEN SET custom_data.status TO "inactive"
        "#;

    let result = PolicyParser::parse_file(policy_text);
    assert!(
        result.is_ok(),
        "Failed to parse comprehensive policy file: {result:?}"
    );

    let rules = result.expect("PolicyParser should successfully parse complete policy file");
    assert_eq!(rules.len(), 5, "Expected 5 rules, got {}", rules.len());

    // Verify specific rule types are parsed correctly
    assert!(rules.iter().any(|r| matches!(r.action, Action::Set { .. })));
    assert!(
        rules
            .iter()
            .any(|r| matches!(r.action, Action::Assert { .. }))
    );
    assert!(
        rules
            .iter()
            .any(|r| matches!(r.action, Action::ApplyTemplate { .. }))
    );
}
