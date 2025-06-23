//! Abstract Syntax Tree definitions for policy rules
//! 
//! This module defines the data structures that represent parsed policy rules
//! and their components.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A complete policy rule with condition and action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Optional identifier for the rule
    pub id: Option<String>,
    pub condition: Condition,
    pub action: Action,
}

/// Conditions that can be evaluated against network nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    /// Boolean AND operation
    And(Box<Condition>, Box<Condition>),
    /// Boolean OR operation  
    Or(Box<Condition>, Box<Condition>),
    /// Boolean NOT operation
    Not(Box<Condition>),
    /// Comparison between field and value
    Comparison {
        field: FieldRef,
        operator: ComparisonOperator,
        value: Value,
    },
    /// Check if field exists/is null
    Existence {
        field: FieldRef,
        is_null: bool,
    },
}

/// Comparison operators for conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Contains,
    Matches, // Regex matching
}

/// Actions that can be executed when conditions are met
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Assert that a field has a specific value (compliance checking)
    Assert {
        field: FieldRef,
        expected: Value,
    },
    /// Set a field to a specific value (mutation)
    Set {
        field: FieldRef,
        value: Value,
    },
    /// Apply a template to generate configuration
    ApplyTemplate {
        template_path: String,
    },
}

/// Reference to a field in the data model (dot notation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldRef {
    pub path: Vec<String>,
}

/// Values that can be used in conditions and actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Regex(String), // Regex pattern as string
    FieldRef(FieldRef), // Reference to another field
}

impl fmt::Display for FieldRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.join("."))
    }
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOperator::Equal => write!(f, "=="),
            ComparisonOperator::NotEqual => write!(f, "!="),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::LessThanOrEqual => write!(f, "<="),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::GreaterThanOrEqual => write!(f, ">="),
            ComparisonOperator::Contains => write!(f, "CONTAINS"),
            ComparisonOperator::Matches => write!(f, "MATCHES"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Regex(r) => write!(f, "/{}/", r),
            Value::FieldRef(field) => write!(f, "{}", field),
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::And(left, right) => write!(f, "({} AND {})", left, right),
            Condition::Or(left, right) => write!(f, "({} OR {})", left, right),
            Condition::Not(cond) => write!(f, "NOT {}", cond),
            Condition::Comparison { field, operator, value } => {
                write!(f, "{} {} {}", field, operator, value)
            }
            Condition::Existence { field, is_null } => {
                if *is_null {
                    write!(f, "{} IS NULL", field)
                } else {
                    write!(f, "{} IS NOT NULL", field)
                }
            }
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Assert { field, expected } => {
                write!(f, "ASSERT {} IS {}", field, expected)
            }
            Action::Set { field, value } => {
                write!(f, "SET {} TO {}", field, value)
            }
            Action::ApplyTemplate { template_path } => {
                write!(f, "APPLY \"{}\"", template_path)
            }
        }
    }
}

impl fmt::Display for PolicyRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WHEN {} THEN {}", self.condition, self.action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_ref_display() {
        let field = FieldRef {
            path: vec!["node".to_string(), "vendor".to_string()],
        };
        assert_eq!(field.to_string(), "node.vendor");
    }

    #[test]
    fn test_value_display() {
        assert_eq!(Value::String("cisco".to_string()).to_string(), "\"cisco\"");
        assert_eq!(Value::Number(15.1).to_string(), "15.1");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::Regex("^dist-\\d+$".to_string()).to_string(), "/^dist-\\d+$/");
        assert_eq!(Value::FieldRef(FieldRef { path: vec!["node".to_string(), "vendor".to_string()] }).to_string(), "node.vendor");
    }

    #[test]
    fn test_condition_display() {
        let field = FieldRef {
            path: vec!["node".to_string(), "vendor".to_string()],
        };
        let condition = Condition::Comparison {
            field,
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        };
        assert_eq!(condition.to_string(), "node.vendor == \"cisco\"");
    }

    #[test]
    fn test_action_display() {
        let field = FieldRef {
            path: vec!["node".to_string(), "version".to_string()],
        };
        let action = Action::Assert {
            field,
            expected: Value::String("15.1".to_string()),
        };
        assert_eq!(action.to_string(), "ASSERT node.version IS \"15.1\"");
    }

    #[test]
    fn test_policy_rule_display() {
        let condition = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        };
        let action = Action::Assert {
            field: FieldRef {
                path: vec!["node".to_string(), "version".to_string()],
            },
            expected: Value::String("15.1".to_string()),
        };
        let rule = PolicyRule { id: None, condition, action };
        assert_eq!(rule.to_string(), "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"");
    }
}