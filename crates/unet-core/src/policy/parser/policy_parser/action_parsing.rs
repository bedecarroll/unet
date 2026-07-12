//! Action parsing logic for policy rules

use super::super::error::ParseError;
use super::super::utils::next_pair;
use super::value_parsing;
use crate::policy::ast::Action;
use crate::policy::grammar::Rule;
use pest::iterators::Pair;

/// Parse an action from a rule pair
pub fn parse_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
    let inner_pair = next_pair(pair.into_inner(), "action inner")?;
    match inner_pair.as_rule() {
        Rule::assert_action => parse_assert_action(inner_pair),
        Rule::set_action => parse_set_action(inner_pair),
        Rule::apply_template_action => parse_apply_template_action(inner_pair),
        _ => Err(ParseError {
            message: format!("Unexpected action rule: {:?}", inner_pair.as_rule()),
            location: None,
        }),
    }
}

/// Parse an assert action (ASSERT field IS value)
pub fn parse_assert_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
    let inner = pair.into_inner();

    // Find the field_ref and value pairs, skipping literal tokens
    let mut field_pair = None;
    let mut expected_pair = None;

    for inner_pair in inner {
        match inner_pair.as_rule() {
            Rule::field_ref => field_pair = Some(inner_pair),
            Rule::value => expected_pair = Some(inner_pair),
            _ => {} // Skip literal tokens like "ASSERT" and "IS"
        }
    }

    let field_pair = field_pair.ok_or_else(|| ParseError {
        message: "Missing field in assert action".to_string(),
        location: None,
    })?;
    let field = value_parsing::parse_field_ref(&field_pair);

    let expected_pair = expected_pair.ok_or_else(|| ParseError {
        message: "Missing expected value in assert action".to_string(),
        location: None,
    })?;
    let expected = value_parsing::parse_value(expected_pair)?;

    Ok(Action::Assert { field, expected })
}

/// Parse a set action (SET field TO value)
pub fn parse_set_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
    let inner = pair.into_inner();

    // Find the field_ref and value pairs, skipping literal tokens
    let mut field_pair = None;
    let mut value_pair = None;

    for inner_pair in inner {
        match inner_pair.as_rule() {
            Rule::field_ref => field_pair = Some(inner_pair),
            Rule::value => value_pair = Some(inner_pair),
            _ => {} // Skip literal tokens like "SET" and "TO"
        }
    }

    let field_pair = field_pair.ok_or_else(|| ParseError {
        message: "Missing field in set action".to_string(),
        location: None,
    })?;
    let field = value_parsing::parse_field_ref(&field_pair);

    let value_pair = value_pair.ok_or_else(|| ParseError {
        message: "Missing value in set action".to_string(),
        location: None,
    })?;
    let value = value_parsing::parse_value(value_pair)?;

    Ok(Action::Set { field, value })
}

/// Parse an apply template action (APPLY `template_path`)
pub fn parse_apply_template_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
    let mut inner = pair.into_inner();

    // Find the string_literal, skipping the "APPLY" token
    let template_pair = inner
        .find(|p| p.as_rule() == Rule::string_literal)
        .ok_or_else(|| ParseError {
            message: "Missing template path in apply action".to_string(),
            location: None,
        })?;

    let template_path_value = value_parsing::parse_value(template_pair)?;

    // Extract string from value
    let crate::policy::ast::Value::String(template_path) = template_path_value else {
        return Err(ParseError {
            message: "Template path must be a string".to_string(),
            location: None,
        });
    };

    Ok(Action::ApplyTemplate { template_path })
}
