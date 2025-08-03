//! Tests for policy evaluation components
//!
//! Contains comprehensive tests for the policy evaluation engine including
//! condition evaluation, action execution, and orchestration.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::policy::ast::*;
    use serde_json::json;

    #[test]
    fn test_simple_condition_evaluation() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "version": "15.1"
            }
        }));

        let condition = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        };

        let result = PolicyEvaluator::evaluate_condition(&condition, &context);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_boolean_and_condition() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "model": "2960"
            }
        }));

        let left = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        };

        let right = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "model".to_string()],
            },
            operator: ComparisonOperator::Contains,
            value: Value::String("29".to_string()),
        };

        let condition = Condition::And(Box::new(left), Box::new(right));
        let result = PolicyEvaluator::evaluate_condition(&condition, &context);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_rule_evaluation() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco"
            }
        }));

        let rule = PolicyRule {
            id: Some("test-rule".to_string()),
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["node".to_string(), "version".to_string()],
                },
                expected: Value::String("15.1".to_string()),
            },
        };

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);
        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::Satisfied { .. } => (),
            _ => panic!("Expected satisfied result"),
        }
    }

    #[test]
    fn test_existence_check() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "location": null
            }
        }));

        let vendor_field = FieldRef {
            path: vec!["node".to_string(), "vendor".to_string()],
        };
        assert!(!PolicyEvaluator::evaluate_existence(&vendor_field, true, &context));
        assert!(PolicyEvaluator::evaluate_existence(&vendor_field, false, &context));

        let location_field = FieldRef {
            path: vec!["node".to_string(), "location".to_string()],
        };
        assert!(PolicyEvaluator::evaluate_existence(&location_field, true, &context));
        assert!(!PolicyEvaluator::evaluate_existence(&location_field, false, &context));

        let missing_field = FieldRef {
            path: vec!["missing".to_string()],
        };
        assert!(PolicyEvaluator::evaluate_existence(&missing_field, true, &context));
        assert!(!PolicyEvaluator::evaluate_existence(&missing_field, false, &context));
    }

    #[test]
    fn test_regex_matching() {
        let context = EvaluationContext::new(json!({
            "node": {
                "version": "15.1(3)T4"
            }
        }));

        let field = FieldRef {
            path: vec!["node".to_string(), "version".to_string()],
        };
        let pattern = Value::String(r"^15\.1.*".to_string());

        let result = PolicyEvaluator::evaluate_comparison(
            &field,
            &ComparisonOperator::Matches,
            &pattern,
            &context,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_numeric_comparison() {
        let context = EvaluationContext::new(json!({
            "interface": {
                "bandwidth": 1000,
                "utilization": 85.5
            }
        }));

        let bandwidth_field = FieldRef {
            path: vec!["interface".to_string(), "bandwidth".to_string()],
        };
        let threshold = Value::Number(100.0);

        let result = PolicyEvaluator::evaluate_comparison(
            &bandwidth_field,
            &ComparisonOperator::GreaterThan,
            &threshold,
            &context,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());

        let utilization_field = FieldRef {
            path: vec!["interface".to_string(), "utilization".to_string()],
        };
        let max_util = Value::Number(90.0);

        let result = PolicyEvaluator::evaluate_comparison(
            &utilization_field,
            &ComparisonOperator::LessThan,
            &max_util,
            &context,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_contains_operation() {
        let context = EvaluationContext::new(json!({
            "config": {
                "vlans": [10, 20, 30],
                "description": "Core switch in building A"
            }
        }));

        // Test array contains
        let vlan_field = FieldRef {
            path: vec!["config".to_string(), "vlans".to_string()],
        };
        let target_vlan = Value::Number(20.0);

        let result = PolicyEvaluator::evaluate_comparison(
            &vlan_field,
            &ComparisonOperator::Contains,
            &target_vlan,
            &context,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test string contains
        let desc_field = FieldRef {
            path: vec!["config".to_string(), "description".to_string()],
        };
        let search_term = Value::String("Core".to_string());

        let result = PolicyEvaluator::evaluate_comparison(
            &desc_field,
            &ComparisonOperator::Contains,
            &search_term,
            &context,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_derived_data_access() {
        let node_data = json!({
            "name": "test-node",
            "vendor": "cisco"
        });

        let derived_data = json!({
            "snmp": {
                "sysUpTime": 12345,
                "interfaces": {
                    "count": 24
                }
            }
        });

        let context = EvaluationContext::with_derived_data(node_data, derived_data);

        // Test accessing derived data
        let derived_field = FieldRef {
            path: vec!["derived".to_string(), "snmp".to_string(), "sysUpTime".to_string()],
        };
        let expected_uptime = Value::Number(12345.0);

        let result = PolicyEvaluator::evaluate_comparison(
            &derived_field,
            &ComparisonOperator::Equal,
            &expected_uptime,
            &context,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test accessing node data
        let node_field = FieldRef {
            path: vec!["vendor".to_string()],
        };
        let expected_vendor = Value::String("cisco".to_string());

        let result = PolicyEvaluator::evaluate_comparison(
            &node_field,
            &ComparisonOperator::Equal,
            &expected_vendor,
            &context,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluation_context_field_access() {
        let context = EvaluationContext::new(json!({
            "config": {
                "vlan": 100,
                "nested": {
                    "value": "test"
                }
            }
        }));

        // Test direct field access
        assert_eq!(context.get_field("config.vlan"), Some(&json!(100)));

        // Test nested field access
        assert_eq!(context.get_field("config.nested.value"), Some(&json!("test")));

        // Test missing field
        assert_eq!(context.get_field("missing.field"), None);

        // Test partial path
        assert_eq!(context.get_field("config.nested"), Some(&json!({"value": "test"})));
    }

    #[test]
    fn test_policy_priority_ordering() {
        use crate::policy::evaluator::results::PolicyPriority;

        let mut priorities = vec![
            PolicyPriority::Low,
            PolicyPriority::Critical,
            PolicyPriority::Medium,
            PolicyPriority::High,
        ];

        priorities.sort();

        assert_eq!(priorities, vec![
            PolicyPriority::Low,
            PolicyPriority::Medium,
            PolicyPriority::High,
            PolicyPriority::Critical,
        ]);
    }
}