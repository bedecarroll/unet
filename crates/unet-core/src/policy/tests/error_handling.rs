//! Error handling and edge case testing

use super::*;

#[test]
fn test_invalid_field_reference() {
    let context = EvaluationContext::new(json!({"node": {"vendor": "cisco"}}));

    let condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["nonexistent".to_string(), "field".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("test".to_string()),
    };

    let rule = PolicyRule {
        id: Some("field-test".to_string()),
        condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "test".to_string()],
            },
            value: Value::String("test".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&rule, &context);
    assert!(result.is_err());

    match result.unwrap_err() {
        PolicyError::FieldNotFound { field } => {
            assert!(field.contains("nonexistent.field"));
        }
        _ => panic!("Expected FieldNotFound error"),
    }
}

#[test]
fn test_invalid_regex_pattern() {
    let context = EvaluationContext::new(json!({"node": {"name": "router-01"}}));

    let condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "name".to_string()],
        },
        operator: ComparisonOperator::Matches,
        value: Value::Regex("[invalid-regex[".to_string()), // Invalid regex
    };

    let rule = PolicyRule {
        id: Some("regex-test".to_string()),
        condition,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["node".to_string(), "name".to_string()],
            },
            expected: Value::String("router-01".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&rule, &context);
    assert!(result.is_err());

    match result.unwrap_err() {
        PolicyError::InvalidRegex { pattern } => {
            assert_eq!(pattern, "[invalid-regex[");
        }
        _ => panic!("Expected InvalidRegex error"),
    }
}

#[test]
fn test_null_value_handling() {
    let context = EvaluationContext::new(json!({
        "node": {
            "vendor": "cisco",
            "location": null,
            "description": null
        }
    }));

    // Test null existence check
    let null_condition = Condition::Existence {
        field: FieldRef {
            path: vec!["node".to_string(), "location".to_string()],
        },
        is_null: true,
    };

    let rule = PolicyRule {
        id: Some("null-test".to_string()),
        condition: null_condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "null_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&rule, &context)
        .expect("PolicyEvaluator should handle null value existence check");
    assert!(matches!(result, EvaluationResult::Satisfied { .. }));

    // Test not null existence check
    let not_null_condition = Condition::Existence {
        field: FieldRef {
            path: vec!["node".to_string(), "vendor".to_string()],
        },
        is_null: false,
    };

    let rule2 = PolicyRule {
        id: Some("not-null-test".to_string()),
        condition: not_null_condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "not_null_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result2 = PolicyEvaluator::evaluate_rule(&rule2, &context).unwrap();
    assert!(matches!(result2, EvaluationResult::Satisfied { .. }));
}

#[test]
fn test_deeply_nested_field_access() {
    let context = EvaluationContext::new(json!({
        "node": {
            "custom_data": {
                "level1": {
                    "level2": {
                        "level3": {
                            "level4": {
                                "deep_value": "found"
                            }
                        }
                    }
                }
            }
        }
    }));

    let condition = Condition::Comparison {
        field: FieldRef {
            path: vec![
                "node".to_string(),
                "custom_data".to_string(),
                "level1".to_string(),
                "level2".to_string(),
                "level3".to_string(),
                "level4".to_string(),
                "deep_value".to_string(),
            ],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("found".to_string()),
    };

    let rule = PolicyRule {
        id: Some("deep-test".to_string()),
        condition,
        action: Action::Assert {
            field: FieldRef {
                path: vec![
                    "node".to_string(),
                    "custom_data".to_string(),
                    "level1".to_string(),
                    "level2".to_string(),
                    "level3".to_string(),
                    "level4".to_string(),
                    "deep_value".to_string(),
                ],
            },
            expected: Value::String("found".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&rule, &context).unwrap();
    assert!(matches!(result, EvaluationResult::Satisfied { .. }));
}

#[test]
fn test_edge_case_conditions() {
    // Test empty string comparison
    let context = EvaluationContext::new(json!({"node": {"name": ""}}));

    let condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "name".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String(String::new()),
    };

    let rule = PolicyRule {
        id: Some("empty-string-test".to_string()),
        condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "empty_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result = PolicyEvaluator::evaluate_rule(&rule, &context).unwrap();
    assert!(matches!(result, EvaluationResult::Satisfied { .. }));

    // Test zero comparison
    let context = EvaluationContext::new(json!({"node": {"count": 0}}));

    let condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "count".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::Number(0.0),
    };

    let rule2 = PolicyRule {
        id: Some("zero-test".to_string()),
        condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "zero_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result2 = PolicyEvaluator::evaluate_rule(&rule2, &context).unwrap();
    assert!(matches!(result2, EvaluationResult::Satisfied { .. }));

    // Test boolean comparison
    let context = EvaluationContext::new(json!({"node": {"enabled": false}}));

    let condition = Condition::Comparison {
        field: FieldRef {
            path: vec!["node".to_string(), "enabled".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::Boolean(false),
    };

    let rule3 = PolicyRule {
        id: Some("boolean-test".to_string()),
        condition,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "boolean_test".to_string()],
            },
            value: Value::String("passed".to_string()),
        },
    };

    let result3 = PolicyEvaluator::evaluate_rule(&rule3, &context).unwrap();
    assert!(matches!(result3, EvaluationResult::Satisfied { .. }));
}

#[test]
fn test_malformed_policy_parsing() {
    // Test various malformed policy strings
    let malformed_policies = [
        "WHEN node.vendor == THEN SET custom_data.test TO 'value'", // Missing value
        "WHEN THEN SET custom_data.test TO 'value'",                // Missing condition
        "WHEN node.vendor == 'cisco' SET custom_data.test TO 'value'", // Missing THEN
        "WHEN node.vendor == 'cisco' THEN custom_data.test TO 'value'", // Missing SET
        "WHEN node.vendor == 'cisco' THEN SET TO 'value'",          // Missing field
        "WHEN node.vendor == 'cisco' THEN SET custom_data.test TO", // Missing value
        "INVALID POLICY SYNTAX",                                    // Completely invalid
    ];

    for (i, policy) in malformed_policies.iter().enumerate() {
        let result = PolicyParser::parse_file(policy);
        assert!(
            result.is_err(),
            "Expected error for malformed policy {i}: {policy}"
        );
    }

    // Test empty string separately (it's valid - no rules)
    let empty_result = PolicyParser::parse_file("");
    assert!(
        empty_result.is_ok(),
        "Empty string should parse to empty rule list"
    );
    assert_eq!(
        empty_result
            .expect("Empty policy string should parse successfully")
            .len(),
        0,
        "Empty string should result in no rules"
    );
}
