//! Tests for basic AST structure and field references

use crate::policy::ast::*;

#[test]
fn test_field_ref_single_path() {
    let field_ref = FieldRef {
        path: vec!["vendor".to_string()],
    };
    assert_eq!(field_ref.path.len(), 1);
    assert_eq!(field_ref.path[0], "vendor");
}

#[test]
fn test_field_ref_empty_path() {
    let field_ref = FieldRef { path: vec![] };
    assert_eq!(field_ref.path.len(), 0);
}

#[test]
fn test_field_ref_long_path() {
    let field_ref = FieldRef {
        path: vec![
            "custom_data".to_string(),
            "config".to_string(),
            "interfaces".to_string(),
            "eth0".to_string(),
            "status".to_string(),
        ],
    };
    assert_eq!(field_ref.path.len(), 5);
    assert_eq!(field_ref.path[0], "custom_data");
    assert_eq!(field_ref.path[4], "status");
}

#[test]
fn test_comparison_with_all_operators() {
    let operators = [
        ComparisonOperator::Equal,
        ComparisonOperator::NotEqual,
        ComparisonOperator::LessThan,
        ComparisonOperator::LessThanOrEqual,
        ComparisonOperator::GreaterThan,
        ComparisonOperator::GreaterThanOrEqual,
        ComparisonOperator::Contains,
        ComparisonOperator::Matches,
    ];

    for operator in operators {
        let condition = Condition::Comparison {
            field: FieldRef {
                path: vec!["test_field".to_string()],
            },
            operator,
            value: Value::String("test_value".to_string()),
        };

        // Verify the condition can be created and contains expected field
        if let Condition::Comparison { field, .. } = condition {
            assert_eq!(field.path[0], "test_field");
        } else {
            panic!("Expected Comparison condition");
        }
    }
}
