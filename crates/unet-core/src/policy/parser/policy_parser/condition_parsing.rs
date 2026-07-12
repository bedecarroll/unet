//! Condition parsing logic for policy rules

use super::super::error::ParseError;
use super::super::utils::next_pair;
use super::value_parsing;
use crate::policy::ast::Condition;
use crate::policy::grammar::Rule;
use pest::iterators::Pair;

/// Parse a condition from a rule pair
pub fn parse_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    let inner_pair = next_pair(pair.into_inner(), "condition inner")?;
    parse_or_condition(inner_pair)
}

/// Parse an OR condition (handles multiple OR operations)
pub fn parse_or_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = parse_and_condition(next_pair(inner.by_ref(), "or condition left side")?)?;

    for right_pair in inner {
        let right = parse_and_condition(right_pair)?;
        left = Condition::Or(Box::new(left), Box::new(right));
    }

    Ok(left)
}

/// Parse an AND condition (handles multiple AND operations)
pub fn parse_and_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = parse_not_condition(next_pair(inner.by_ref(), "and condition left side")?)?;

    for right_pair in inner {
        let right = parse_not_condition(right_pair)?;
        left = Condition::And(Box::new(left), Box::new(right));
    }

    Ok(left)
}

/// Parse a NOT condition (negation)
pub fn parse_not_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    let condition_text = pair.as_str().trim_start();
    let condition =
        parse_primary_condition(next_pair(pair.into_inner(), "not condition expression")?)?;
    let is_negated = condition_text.strip_prefix("NOT").is_some_and(|rest| {
        rest.chars()
            .next()
            .is_none_or(|character| character.is_whitespace() || character == '(')
    });

    if is_negated {
        Ok(Condition::Not(Box::new(condition)))
    } else {
        Ok(condition)
    }
}

/// Parse a primary condition (comparison, existence check, or nested condition)
pub fn parse_primary_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    match pair.as_rule() {
        Rule::condition => parse_condition(pair),
        Rule::comparison => parse_comparison(pair),
        Rule::existence_check => parse_existence_check(pair),
        Rule::primary_condition => {
            let inner = next_pair(pair.into_inner(), "primary condition expression")?;
            parse_primary_condition(inner)
        }
        _ => Err(ParseError {
            message: format!("Unexpected rule in primary condition: {:?}", pair.as_rule()),
            location: None,
        }),
    }
}

/// Parse a comparison condition (field operator value)
pub fn parse_comparison(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    let mut inner = pair.into_inner();
    let field =
        value_parsing::parse_field_ref(&next_pair(inner.by_ref(), "comparison field reference")?);
    let operator =
        value_parsing::parse_operator(&next_pair(inner.by_ref(), "comparison operator")?)?;
    let value = value_parsing::parse_value(next_pair(inner, "comparison value")?)?;

    Ok(Condition::Comparison {
        field,
        operator,
        value,
    })
}

/// Parse an existence check condition (field IS [NOT] NULL)
pub fn parse_existence_check(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    let existence_text = pair.as_str().trim();
    let field_pair = next_pair(pair.into_inner(), "existence field reference")?;
    let field = value_parsing::parse_field_ref(&field_pair);
    let suffix = existence_text
        .strip_prefix(field_pair.as_str())
        .map(str::trim)
        .ok_or_else(|| ParseError {
            message: format!(
                "Expected NULL or NOT NULL after field reference, got: {existence_text}"
            ),
            location: None,
        })?;
    let is_null = match suffix {
        "IS NULL" => true,
        "IS NOT NULL" => false,
        _ => {
            return Err(ParseError {
                message: format!("Expected NULL or NOT NULL after field reference, got: {suffix}"),
                location: None,
            });
        }
    };

    Ok(Condition::Existence { field, is_null })
}
