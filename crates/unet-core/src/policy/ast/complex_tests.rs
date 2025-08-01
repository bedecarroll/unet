//! Tests for complex nested AST structures

use crate::policy::ast::*;
use std::collections::HashMap;

#[test]
fn test_complex_nested_conditions() {
    let complex_condition = Condition::And(
        Box::new(Condition::Or(
            Box::new(Condition::Comparison {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            }),
            Box::new(Condition::Comparison {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("juniper".to_string()),
            }),
        )),
        Box::new(Condition::Not(Box::new(Condition::Comparison {
            field: FieldRef {
                path: vec!["lifecycle".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("decommissioned".to_string()),
        }))),
    );

    // Verify the complex structure can be created
    if let Condition::And(left, right) = complex_condition {
        assert!(matches!(*left, Condition::Or(_, _)));
        assert!(matches!(*right, Condition::Not(_)));
    } else {
        panic!("Expected And condition");
    }
}

#[test]
fn test_value_with_nested_field_ref() {
    let field_ref = FieldRef {
        path: vec!["custom_data".to_string(), "nested".to_string()],
    };
    let value = Value::FieldRef(field_ref);

    if let Value::FieldRef(fr) = value {
        assert_eq!(fr.path.len(), 2);
        assert_eq!(fr.path[0], "custom_data");
        assert_eq!(fr.path[1], "nested");
    }
}

#[test]
fn test_value_with_complex_array() {
    let complex_array = Value::Array(vec![
        Value::String("simple_string".to_string()),
        Value::Number(42.5),
        Value::Boolean(true),
        Value::Array(vec![
            Value::String("nested_string".to_string()),
            Value::Number(123.0),
        ]),
        Value::Object({
            let mut obj = HashMap::new();
            obj.insert(
                "nested_key".to_string(),
                Value::String("nested_value".to_string()),
            );
            obj.insert("nested_number".to_string(), Value::Number(456.0));
            obj
        }),
        Value::Null,
        Value::Regex(r"^test.*$".to_string()),
        Value::FieldRef(FieldRef {
            path: vec!["reference".to_string(), "field".to_string()],
        }),
    ]);

    if let Value::Array(items) = complex_array {
        assert_eq!(items.len(), 8);

        // Verify each item type
        assert!(matches!(items[0], Value::String(_)));
        assert!(matches!(items[1], Value::Number(_)));
        assert!(matches!(items[2], Value::Boolean(_)));
        assert!(matches!(items[3], Value::Array(_)));
        assert!(matches!(items[4], Value::Object(_)));
        assert!(matches!(items[5], Value::Null));
        assert!(matches!(items[6], Value::Regex(_)));
        assert!(matches!(items[7], Value::FieldRef(_)));
    }
}

#[test]
fn test_object_with_complex_values() {
    let mut complex_obj = HashMap::new();

    complex_obj.insert(
        "simple_string".to_string(),
        Value::String("value".to_string()),
    );
    complex_obj.insert("number".to_string(), Value::Number(42.0));
    complex_obj.insert("boolean".to_string(), Value::Boolean(false));
    complex_obj.insert("null_value".to_string(), Value::Null);
    complex_obj.insert("regex".to_string(), Value::Regex(r"\d+".to_string()));

    complex_obj.insert(
        "nested_array".to_string(),
        Value::Array(vec![
            Value::String("array_item".to_string()),
            Value::Number(123.0),
        ]),
    );

    complex_obj.insert(
        "nested_object".to_string(),
        Value::Object({
            let mut nested = HashMap::new();
            nested.insert(
                "inner_key".to_string(),
                Value::String("inner_value".to_string()),
            );
            nested
        }),
    );

    complex_obj.insert(
        "field_reference".to_string(),
        Value::FieldRef(FieldRef {
            path: vec!["referenced".to_string(), "field".to_string()],
        }),
    );

    let object_value = Value::Object(complex_obj);

    if let Value::Object(obj) = object_value {
        assert_eq!(obj.len(), 8);
        assert!(obj.contains_key("simple_string"));
        assert!(obj.contains_key("nested_array"));
        assert!(obj.contains_key("nested_object"));
        assert!(obj.contains_key("field_reference"));
    }
}
