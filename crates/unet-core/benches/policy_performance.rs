use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::json;
use unet_core::policy::{
    Action, ComparisonOperator, Condition, EvaluationContext, EvaluationResult, FieldRef,
    PolicyEvaluator, PolicyParser, PolicyRule, Value,
};

fn bench_large_policy_set_parsing(c: &mut Criterion) {
    c.bench_function("parse_1000_rules", |b| {
        b.iter(|| {
            let mut policy_text = String::new();
            for i in 0..1000 {
                use std::fmt::Write;
                writeln!(
                    policy_text,
                    r#"WHEN node.id == "node-{i}" THEN SET custom_data.field_{i} TO "value_{i}""#
                )
                .unwrap();
            }
            let rules = PolicyParser::parse_file(&policy_text).expect("parse");
            assert_eq!(rules.len(), 1000);
        });
    });
}

fn bench_rule_evaluation(c: &mut Criterion) {
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

    c.bench_function("evaluate_rule_1000_times", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                let result = PolicyEvaluator::evaluate_rule(&rule, &context).expect("eval");
                assert!(matches!(result, EvaluationResult::Satisfied { .. }));
            }
        });
    });
}

criterion_group!(
    policy_perf,
    bench_large_policy_set_parsing,
    bench_rule_evaluation
);
criterion_main!(policy_perf);
