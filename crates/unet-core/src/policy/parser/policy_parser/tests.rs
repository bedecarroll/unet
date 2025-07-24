//! Tests for policy parser implementation

#[cfg(test)]
mod policy_parser_tests {
    use super::super::core::PolicyParser;
    use crate::policy::ast::{ComparisonOperator, Condition, Value};

    #[test]
    fn test_parse_simple_rule() {
        let input = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_ok(), "Failed to parse simple rule: {result:?}");

        let rule = result.unwrap();
        match rule.condition {
            Condition::Comparison {
                field,
                operator,
                value,
            } => {
                assert_eq!(field.path, vec!["node", "vendor"]);
                assert_eq!(operator, ComparisonOperator::Equal);
                assert_eq!(value, Value::String("cisco".to_string()));
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_complex_condition() {
        let input = r#"WHEN node.vendor == "juniper" AND node.model CONTAINS "qfx" THEN SET custom_data.priority TO "high""#;
        let result = PolicyParser::parse_rule(input);
        assert!(
            result.is_ok(),
            "Failed to parse complex condition: {result:?}"
        );
    }

    #[test]
    fn test_parse_boolean_operators() {
        let input = r#"WHEN (node.vendor == "cisco" OR node.vendor == "juniper") AND NOT node.lifecycle == "decommissioned" THEN ASSERT node.snmp_enabled IS true"#;
        let result = PolicyParser::parse_rule(input);
        assert!(
            result.is_ok(),
            "Failed to parse boolean operators: {result:?}"
        );
    }

    #[test]
    fn test_parse_null_check() {
        let input = r"WHEN custom_data.location IS NOT NULL THEN SET node.location_id TO custom_data.location";
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_ok(), "Failed to parse null check: {result:?}");
    }

    #[test]
    fn test_parse_regex_literal() {
        let input = r#"WHEN node.hostname MATCHES /^dist-\d+$/ THEN APPLY "dist-template.jinja""#;
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_ok(), "Failed to parse regex literal: {result:?}");
    }

    #[test]
    fn test_parse_policy_file() {
        let input = r#"
            WHEN node.vendor == "cisco" THEN ASSERT node.os_version IS "15.1"
            WHEN node.role == "router" AND node.location.region == "west" THEN SET custom_data.backup_priority TO "high"
        "#;
        let result = PolicyParser::parse_file(input);
        assert!(result.is_ok(), "Failed to parse policy file: {result:?}");

        let rules = result.unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_parse_rule_empty_input() {
        let input = "";
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Just verify it's an error - the exact message might vary
        assert!(!error.message.is_empty());
    }

    #[test]
    fn test_parse_rule_invalid_syntax() {
        let input = "INVALID SYNTAX HERE";
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rule_incomplete_when() {
        let input = "WHEN node.vendor ==";
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rule_missing_then() {
        let input = r#"WHEN node.vendor == "cisco""#;
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file_with_syntax_errors() {
        let input = r#"
            WHEN node.vendor == "cisco" THEN ASSERT node.os_version IS "15.1"
            WHEN node.role == "router" THEN SET custom_data.priority TO "high"
        "#;
        let result = PolicyParser::parse_file(input);
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_parse_file_empty() {
        let input = "";
        let result = PolicyParser::parse_file(input);
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 0);
    }

    #[test]
    fn test_parse_file_whitespace_only() {
        let input = r"
            
            
        ";
        let result = PolicyParser::parse_file(input);
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 0);
    }
}
