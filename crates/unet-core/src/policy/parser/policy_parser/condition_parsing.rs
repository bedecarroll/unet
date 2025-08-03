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
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    if first.as_str() == "NOT" {
        let condition = parse_primary_condition(inner.next().unwrap())?;
        Ok(Condition::Not(Box::new(condition)))
    } else {
        parse_primary_condition(first)
    }
}

/// Parse a primary condition (comparison, existence check, or nested condition)
pub fn parse_primary_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    match pair.as_rule() {
        Rule::condition => parse_condition(pair),
        Rule::comparison => parse_comparison(pair),
        Rule::existence_check => parse_existence_check(pair),
        Rule::primary_condition => {
            // Unwrap the inner rule
            let inner = pair.into_inner().next().unwrap();
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
    let field = value_parsing::parse_field_ref(&inner.next().unwrap());
    let operator = value_parsing::parse_operator(&inner.next().unwrap())?;
    let value = value_parsing::parse_value(inner.next().unwrap())?;

    Ok(Condition::Comparison {
        field,
        operator,
        value,
    })
}

/// Parse an existence check condition (field IS [NOT] NULL)
pub fn parse_existence_check(pair: Pair<Rule>) -> Result<Condition, ParseError> {
    let mut inner = pair.into_inner();
    let field = value_parsing::parse_field_ref(&inner.next().unwrap());

    // Skip "IS" token - it's implicit in the grammar
    let mut is_null = true; // Default to IS NULL

    // Check if we have NOT NULL
    if let Some(next_token) = inner.next() {
        match next_token.as_str() {
            "NULL" => is_null = true,
            "NOT" => {
                // This should be followed by NULL
                if let Some(null_token) = inner.next() {
                    if null_token.as_str() == "NULL" {
                        is_null = false;
                    } else {
                        return Err(ParseError {
                            message: format!(
                                "Expected NULL after NOT, got: {}",
                                null_token.as_str()
                            ),
                            location: None,
                        });
                    }
                } else {
                    return Err(ParseError {
                        message: "Expected NULL after NOT".to_string(),
                        location: None,
                    });
                }
            }
            _ => {
                return Err(ParseError {
                    message: format!("Expected NULL or NOT, got: {}", next_token.as_str()),
                    location: None,
                });
            }
        }
    }

    Ok(Condition::Existence { field, is_null })
}
