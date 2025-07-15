//! Performance tests for large policy sets

use super::*;

#[test]
fn test_large_policy_set_parsing_performance() {
    let start = Instant::now();

    // Create 1000 policy rules as text
    let mut policy_text = String::new();
    for i in 0..1000 {
        use std::fmt::Write;
        writeln!(
            policy_text,
            r#"WHEN node.id == "node-{i}" THEN SET custom_data.field_{i} TO "value_{i}""#
        )
        .expect("Writing to String should not fail");
    }

    let rule_creation_time = start.elapsed();
    println!("Created 1000 policy rule strings in {rule_creation_time:?}");

    // Test parsing performance
    let parse_start = Instant::now();
    let result = PolicyParser::parse_file(&policy_text);
    let parse_time = parse_start.elapsed();

    println!("Parsed 1000 rules in {parse_time:?}");

    // Verify parsing succeeded
    assert!(result.is_ok(), "Failed to parse policy text: {result:?}");
    let rules = result.expect("PolicyParser should successfully parse 1000 valid policy rules");
    assert_eq!(
        rules.len(),
        1000,
        "Expected 1000 rules, got {}",
        rules.len()
    );

    // Performance should be reasonable (< 5 seconds for 1000 rules)
    assert!(
        parse_time < Duration::from_secs(5),
        "Performance test failed: took {parse_time:?}"
    );
}

#[test]
fn test_rule_evaluation_performance() {
    // Create a complex rule with nested conditions
    let rule = PolicyRule {
        id: Some("performance-test".to_string()),
        condition: Condition::And(
            Box::new(Condition::Comparison {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            }),
            Box::new(Condition::Or(
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["node".to_string(), "model".to_string()],
                    },
                    operator: ComparisonOperator::Contains,
                    value: Value::String("2960".to_string()),
                }),
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["node".to_string(), "model".to_string()],
                    },
                    operator: ComparisonOperator::Matches,
                    value: Value::Regex(r"^catalyst.*".to_string()),
                }),
            )),
        ),
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "performance_test".to_string()],
            },
            value: Value::String("completed".to_string()),
        },
    };

    let context = EvaluationContext::new(json!({
        "node": {
            "vendor": "cisco",
            "model": "catalyst-2960-x"
        }
    }));

    // Evaluate the rule 1,000 times (reduced for performance)
    let start = Instant::now();
    for _ in 0..1_000 {
        let result = PolicyEvaluator::evaluate_rule(&rule, &context)
            .expect("PolicyEvaluator should successfully evaluate performance test rule");
        assert!(matches!(result, EvaluationResult::Satisfied { .. }));
    }
    let elapsed = start.elapsed();

    println!("Evaluated complex rule 1,000 times in {elapsed:?}");

    // Should complete in reasonable time (< 1 second for 1k evaluations)
    assert!(
        elapsed < Duration::from_secs(1),
        "Rule evaluation performance test failed: took {elapsed:?}"
    );
}
