//! Abstract Syntax Tree definitions for policy rules
//!
//! This module defines the data structures that represent parsed policy rules
//! and their components.

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

/// A complete policy rule with condition and action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Optional identifier for the rule
    pub id: Option<String>,
    /// Condition that must be met for the action to be executed
    pub condition: Condition,
    /// Action to execute when condition is satisfied
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
        /// Field reference to compare
        field: FieldRef,
        /// Comparison operator to use
        operator: ComparisonOperator,
        /// Value to compare against
        value: Value,
    },
    /// Check if field exists/is null
    Existence {
        /// Field reference to check
        field: FieldRef,
        /// Whether to check for null (true) or non-null (false)
        is_null: bool,
    },
    /// Always true condition
    True,
    /// Always false condition
    False,
}

/// Comparison operators for conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Equality comparison (==)
    Equal,
    /// Inequality comparison (!=)
    NotEqual,
    /// Less than comparison (<)
    LessThan,
    /// Less than or equal comparison (<=)
    LessThanOrEqual,
    /// Greater than comparison (>)
    GreaterThan,
    /// Greater than or equal comparison (>=)
    GreaterThanOrEqual,
    /// String contains operation
    Contains,
    /// Regex pattern matching
    Matches,
}

/// Actions that can be executed when conditions are met
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Assert that a field has a specific value (compliance checking)
    Assert {
        /// Field reference to check
        field: FieldRef,
        /// Expected value for the field
        expected: Value,
    },
    /// Set a field to a specific value (mutation)
    Set {
        /// Field reference to set
        field: FieldRef,
        /// Value to set the field to
        value: Value,
    },
    /// Apply a template to generate configuration
    ApplyTemplate {
        /// Path to the template file
        template_path: String,
    },
}

/// Reference to a field in the data model (dot notation)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldRef {
    /// Path components for field access (e.g., `["interface", "status"]`)
    pub path: Vec<String>,
}

/// Values that can be used in conditions and actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// String literal value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Null value
    Null,
    /// Regex pattern as string
    Regex(String),
    /// Reference to another field
    FieldRef(FieldRef),
    /// Array of values
    Array(Vec<Value>),
    /// Object with key-value pairs  
    Object(HashMap<String, Value>),
}

impl fmt::Display for FieldRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.join("."))
    }
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::LessThan => write!(f, "<"),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanOrEqual => write!(f, ">="),
            Self::Contains => write!(f, "CONTAINS"),
            Self::Matches => write!(f, "MATCHES"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Number(n) => write!(f, "{n}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Null => write!(f, "null"),
            Self::Regex(r) => write!(f, "/{r}/"),
            Self::FieldRef(field) => write!(f, "{field}"),
            Self::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, "]")
            }
            Self::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{key}\": {val}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::And(left, right) => write!(f, "({left} AND {right})"),
            Self::Or(left, right) => write!(f, "({left} OR {right})"),
            Self::Not(cond) => write!(f, "NOT {cond}"),
            Self::Comparison {
                field,
                operator,
                value,
            } => {
                write!(f, "{field} {operator} {value}")
            }
            Self::Existence { field, is_null } => {
                if *is_null {
                    write!(f, "{field} IS NULL")
                } else {
                    write!(f, "{field} IS NOT NULL")
                }
            }
            Self::True => write!(f, "TRUE"),
            Self::False => write!(f, "FALSE"),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assert { field, expected } => {
                write!(f, "ASSERT {field} IS {expected}")
            }
            Self::Set { field, value } => {
                write!(f, "SET {field} TO {value}")
            }
            Self::ApplyTemplate { template_path } => {
                write!(f, "APPLY \"{template_path}\"")
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
        assert_eq!(
            Value::Regex("^dist-\\d+$".to_string()).to_string(),
            "/^dist-\\d+$/"
        );
        assert_eq!(
            Value::FieldRef(FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()]
            })
            .to_string(),
            "node.vendor"
        );
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
        let rule = PolicyRule {
            id: None,
            condition,
            action,
        };
        assert_eq!(
            rule.to_string(),
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        );
    }
}
