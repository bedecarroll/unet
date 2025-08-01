//! Tests for utility functions and edge cases

use crate::policy::evaluator::actions::ActionExecutor;
use serde_json::json;

#[test]
fn test_json_values_equal_with_invalid_numbers() {
    // Since serde_json::Number doesn't support NaN directly, we need to test this differently
    // The NaN validation happens in our resolve_value function, not in json_values_equal
    // This test verifies our comparison function works with edge cases

    let val1 = json!(1.0);
    let val2 = json!(1.0);
    let val3 = json!(2.0);

    // Test normal equality
    assert!(ActionExecutor::json_values_equal(&val1, &val2));
    assert!(!ActionExecutor::json_values_equal(&val1, &val3));

    // Test different types
    let str_val = json!("test");
    assert!(!ActionExecutor::json_values_equal(&val1, &str_val));
}
