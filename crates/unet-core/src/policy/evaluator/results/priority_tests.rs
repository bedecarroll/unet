//! Tests for policy priority functionality

use crate::policy::evaluator::results::PolicyPriority;

#[test]
fn test_policy_priority_default() {
    assert_eq!(PolicyPriority::default(), PolicyPriority::Medium);
}

#[test]
fn test_policy_priority_serialization() {
    // Test that priorities can be serialized and deserialized
    let priorities = [
        PolicyPriority::Low,
        PolicyPriority::Medium,
        PolicyPriority::High,
        PolicyPriority::Critical,
    ];

    for priority in priorities {
        let serialized = serde_json::to_string(&priority).unwrap();
        let deserialized: PolicyPriority = serde_json::from_str(&serialized).unwrap();
        assert_eq!(priority, deserialized);
    }
}

#[test]
fn test_policy_priority_numeric_values() {
    // Test that priority values are correctly ordered numerically
    assert_eq!(PolicyPriority::Low as u8, 0);
    assert_eq!(PolicyPriority::Medium as u8, 1);
    assert_eq!(PolicyPriority::High as u8, 2);
    assert_eq!(PolicyPriority::Critical as u8, 3);
}
