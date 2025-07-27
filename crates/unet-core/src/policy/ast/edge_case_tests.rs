//! Tests for edge cases and special scenarios

use crate::policy::ast::*;

#[test]
fn test_edge_case_empty_strings() {
    let empty_string_value = Value::String(String::new());
    assert_eq!(empty_string_value.to_string(), "\"\"");

    let field_ref_with_empty = FieldRef {
        path: vec![String::new()],
    };
    assert_eq!(field_ref_with_empty.path.len(), 1);
    assert_eq!(field_ref_with_empty.path[0], "");

    let condition_with_empty = Condition::Comparison {
        field: field_ref_with_empty,
        operator: ComparisonOperator::Equal,
        value: empty_string_value,
    };

    // Verify the condition can be constructed with empty strings
    if let Condition::Comparison { field, value, .. } = condition_with_empty {
        assert_eq!(field.path[0], "");
        assert_eq!(value, Value::String(String::new()));
    }
}

#[test]
fn test_special_characters_in_strings() {
    let special_chars = Value::String("Hello, World! @#$%^&*()_+-=[]{}|;':\",./<>?".to_string());
    let display = special_chars.to_string();
    assert!(display.contains("Hello, World!"));
    assert!(display.contains("@#$%^&*()"));

    let unicode_value = Value::String("🚀 Unicode 测试 العربية".to_string());
    let display = unicode_value.to_string();
    assert!(display.contains("🚀"));
    assert!(display.contains("测试"));
    assert!(display.contains("العربية"));
}

#[test]
fn test_large_numbers() {
    let large_positive = Value::Number(1_000_000_000_000.0);
    let large_negative = Value::Number(-999_999_999_999.0);
    let very_small = Value::Number(0.000_000_001);

    assert_eq!(large_positive, Value::Number(1e12));
    assert_eq!(large_negative, Value::Number(-999_999_999_999.0));
    assert!(matches!(very_small, Value::Number(n) if n > 0.0 && n < 0.001));

    // Test floating point precision edge cases
    let precise_float = Value::Number(std::f64::consts::PI);
    if let Value::Number(n) = precise_float {
        assert!((n - std::f64::consts::PI).abs() < 1e-10);
    }
}

#[test]
fn test_regex_pattern_edge_cases() {
    // Test empty regex pattern
    let empty_regex = Value::Regex(String::new());
    assert_eq!(empty_regex.to_string(), "//");

    // Test complex regex patterns (using simpler email pattern to avoid string issues)
    let complex_regex =
        Value::Regex(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string());

    // Verify it can be constructed and contains expected pattern
    if let Value::Regex(pattern) = complex_regex {
        assert!(pattern.starts_with('^'));
        assert!(pattern.ends_with('$'));
    }

    // Test regex with escape sequences
    let escaped_regex = Value::Regex(r"\.+\*\?\[\]\(\)\{\}\|\\\^$".to_string());
    if let Value::Regex(pattern) = escaped_regex {
        assert!(pattern.contains(r"\."));
        assert!(pattern.contains(r"\*"));
        assert!(pattern.contains(r"\?"));
    }
}

#[test]
fn test_deeply_nested_structures() {
    // Create a deeply nested condition structure
    let mut current_condition = Condition::True;

    for i in 0..10 {
        current_condition = Condition::And(
            Box::new(current_condition),
            Box::new(Condition::Comparison {
                field: FieldRef {
                    path: vec![format!("level_{i}")],
                },
                operator: ComparisonOperator::Equal,
                value: Value::Number(f64::from(i)),
            }),
        );
    }

    // Verify the deeply nested structure can be created
    let display = current_condition.to_string();
    assert!(display.contains("AND"));
    assert!(display.contains("level_0"));
    assert!(display.contains("level_9"));
}
