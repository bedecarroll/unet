//! Main policy parser implementation
//!
//! This module has been split into focused submodules:
//! - `entry_points`: Main parsing entry points for rules and files
//! - `condition_parsing`: Condition parsing logic (or, and, not, comparison)
//! - `value_parsing`: Value, field ref, and operator parsing
//! - `action_parsing`: Action parsing logic (assert, set, apply template)

use super::super::error::ParseError;
use super::entry_points;
use crate::policy::ast::PolicyRule;

/// Parser for policy rules
pub struct PolicyParser;

impl PolicyParser {
    /// Parse a single policy rule from text
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - The input contains invalid syntax
    /// - The rule structure is malformed
    /// - Required components are missing
    pub fn parse_rule(input: &str) -> Result<PolicyRule, ParseError> {
        entry_points::parse_rule_from_input(input)
    }

    /// Parse multiple policy rules from a policy file
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - The file contains invalid syntax
    /// - Any rule structure is malformed
    /// - Required components are missing
    pub fn parse_file(input: &str) -> Result<Vec<PolicyRule>, ParseError> {
        entry_points::parse_file_from_input(input)
    }
}
