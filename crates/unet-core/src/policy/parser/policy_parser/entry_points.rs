//! Main parsing entry points for policy rules and files

use super::super::error::ParseError;
use super::{action_parsing, condition_parsing};
use crate::policy::ast::PolicyRule;
use crate::policy::grammar::{PolicyGrammar, Rule};
use pest::{Parser, iterators::Pair};

/// Parse a single policy rule from text input
pub fn parse_rule_from_input(input: &str) -> Result<PolicyRule, ParseError> {
    let pairs = PolicyGrammar::parse(Rule::rule, input).map_err(|e| ParseError {
        message: e.to_string(),
        location: None,
    })?;

    let rule_pair = pairs.into_iter().next().ok_or_else(|| ParseError {
        message: "No rule found in input".to_string(),
        location: None,
    })?;

    parse_rule_pair(rule_pair)
}

/// Parse multiple policy rules from a policy file
pub fn parse_file_from_input(input: &str) -> Result<Vec<PolicyRule>, ParseError> {
    let pairs = PolicyGrammar::parse(Rule::policy_file, input).map_err(|e| ParseError {
        message: e.to_string(),
        location: None,
    })?;

    let mut rules = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::policy_file {
            for inner_pair in pair.into_inner() {
                if inner_pair.as_rule() == Rule::rule {
                    rules.push(parse_rule_pair(inner_pair)?);
                }
            }
        }
    }

    Ok(rules)
}

/// Parse a rule pair into a `PolicyRule`
pub fn parse_rule_pair(pair: Pair<Rule>) -> Result<PolicyRule, ParseError> {
    let mut condition = None;
    let mut action = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::condition => {
                condition = Some(condition_parsing::parse_condition(inner_pair)?);
            }
            Rule::action => {
                action = Some(action_parsing::parse_action(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(PolicyRule {
        id: None, // Parser doesn't set IDs - they can be set by loader
        condition: condition.ok_or_else(|| ParseError {
            message: "Missing condition in rule".to_string(),
            location: None,
        })?,
        action: action.ok_or_else(|| ParseError {
            message: "Missing action in rule".to_string(),
            location: None,
        })?,
    })
}
