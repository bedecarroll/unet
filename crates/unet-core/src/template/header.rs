//! Template header parsing and matching system
//!
//! This module provides functionality for parsing and matching template headers
//! that specify which configuration sections a template should apply to.

use anyhow::{Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template match header specification
///
/// Headers define which configuration sections a template should apply to
/// using various matching strategies including regex, hierarchical paths, and glob patterns.
///
/// # Examples
///
/// ```
/// # Template header examples:
/// # template-match: interface.ethernet.*
/// # template-match: /^vlan\.\d+$/
/// # template-match: bgp.neighbors.**.ipv4
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateHeader {
    /// The raw header specification string
    pub raw: String,
    /// Parsed match pattern
    pub pattern: MatchPattern,
    /// Optional priority for conflict resolution (higher = more priority)
    pub priority: Option<u32>,
    /// Optional scope restrictions
    pub scope: Option<TemplateScope>,
}

/// Different types of matching patterns supported
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchPattern {
    /// Exact string match
    Exact(String),
    /// Regular expression pattern
    Regex(String),
    /// Hierarchical path with wildcards (dot-separated)
    HierarchicalPath(String),
    /// Glob pattern with * and ** support
    Glob(String),
}

/// Template scope restrictions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateScope {
    /// Device types this template applies to
    pub device_types: Option<Vec<String>>,
    /// Vendor restrictions
    pub vendors: Option<Vec<String>>,
    /// Configuration contexts (e.g., "global", "interface", "routing")
    pub contexts: Option<Vec<String>>,
}

/// Template header parser
#[derive(Debug, Clone)]
pub struct HeaderParser {
    /// Compiled regex patterns for validation
    regex_cache: HashMap<String, Regex>,
}

impl HeaderParser {
    /// Create a new header parser
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }

    /// Parse a template header specification
    ///
    /// # Arguments
    ///
    /// * `header_line` - The template header line to parse
    ///
    /// # Returns
    ///
    /// A parsed `TemplateHeader` or an error if parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use unet_core::template::header::HeaderParser;
    ///
    /// let parser = HeaderParser::new();
    ///
    /// // Parse different header types
    /// let exact = parser.parse("template-match: interface.ethernet.eth0")?;
    /// let regex = parser.parse("template-match: /^vlan\\.\\d+$/")?;
    /// let glob = parser.parse("template-match: bgp.neighbors.*.ipv4")?;
    /// let hierarchical = parser.parse("template-match: ospf.area.**.networks")?;
    /// ```
    pub fn parse(&mut self, header_line: &str) -> Result<TemplateHeader> {
        let trimmed = header_line.trim();

        // Extract the header directive and value
        if !trimmed.starts_with("template-match:") {
            return Err(anyhow!(
                "Invalid template header: must start with 'template-match:'"
            ));
        }

        let value = trimmed.strip_prefix("template-match:").unwrap().trim();

        if value.is_empty() {
            return Err(anyhow!("Empty template match pattern"));
        }

        // Parse the pattern based on its format
        let pattern = self.parse_pattern(value)?;

        Ok(TemplateHeader {
            raw: header_line.to_string(),
            pattern,
            priority: None,
            scope: None,
        })
    }

    /// Parse a pattern string into a MatchPattern
    fn parse_pattern(&mut self, pattern: &str) -> Result<MatchPattern> {
        // Check for regex pattern (enclosed in forward slashes)
        if pattern.starts_with('/') && pattern.ends_with('/') && pattern.len() > 2 {
            let regex_str = &pattern[1..pattern.len() - 1];
            // Validate the regex
            let regex = Regex::new(regex_str)
                .map_err(|e| anyhow!("Invalid regex pattern '{}': {}", regex_str, e))?;
            self.regex_cache.insert(regex_str.to_string(), regex);
            return Ok(MatchPattern::Regex(regex_str.to_string()));
        }

        // Check for glob patterns (contains * or **)
        if pattern.contains('*') {
            self.validate_glob_pattern(pattern)?;
            return Ok(MatchPattern::Glob(pattern.to_string()));
        }

        // Check for hierarchical path patterns (contains dots and potentially wildcards)
        if pattern.contains('.') {
            return Ok(MatchPattern::HierarchicalPath(pattern.to_string()));
        }

        // Default to exact match
        Ok(MatchPattern::Exact(pattern.to_string()))
    }

    /// Validate a glob pattern syntax
    fn validate_glob_pattern(&self, pattern: &str) -> Result<()> {
        // Basic validation for glob patterns
        // Ensure ** is not followed by non-separator characters
        if pattern.contains("***") {
            return Err(anyhow!("Invalid glob pattern: triple asterisk not allowed"));
        }

        // Validate that ** is properly separated
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() > 1 {
            for (i, part) in parts.iter().enumerate() {
                if i > 0 && !part.is_empty() && !part.starts_with('.') && !part.starts_with('/') {
                    return Err(anyhow!(
                        "Invalid glob pattern: ** must be followed by separator"
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if a configuration path matches a template header
    pub fn matches(&mut self, header: &TemplateHeader, config_path: &str) -> Result<bool> {
        match &header.pattern {
            MatchPattern::Exact(pattern) => Ok(pattern == config_path),
            MatchPattern::Regex(pattern) => {
                let regex = if let Some(compiled) = self.regex_cache.get(pattern) {
                    compiled
                } else {
                    let compiled = Regex::new(pattern)
                        .map_err(|e| anyhow!("Failed to compile regex '{}': {}", pattern, e))?;
                    self.regex_cache.insert(pattern.clone(), compiled);
                    self.regex_cache.get(pattern).unwrap()
                };
                Ok(regex.is_match(config_path))
            }
            MatchPattern::HierarchicalPath(pattern) => {
                self.matches_hierarchical(pattern, config_path)
            }
            MatchPattern::Glob(pattern) => self.matches_glob(pattern, config_path),
        }
    }

    /// Match hierarchical path patterns
    fn matches_hierarchical(&self, pattern: &str, path: &str) -> Result<bool> {
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let path_parts: Vec<&str> = path.split('.').collect();

        self.match_parts(&pattern_parts, &path_parts, 0, 0)
    }

    /// Match glob patterns
    fn matches_glob(&self, pattern: &str, path: &str) -> Result<bool> {
        // Convert glob pattern to regex
        let regex_pattern = self.glob_to_regex(pattern)?;
        let regex = Regex::new(&regex_pattern)
            .map_err(|e| anyhow!("Failed to compile glob regex: {}", e))?;
        Ok(regex.is_match(path))
    }

    /// Convert glob pattern to regex
    fn glob_to_regex(&self, pattern: &str) -> Result<String> {
        let mut regex = String::new();
        regex.push('^');

        let mut chars = pattern.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next(); // consume second *
                        regex.push_str(".*"); // ** matches anything including separators
                    } else {
                        regex.push_str("[^.]*"); // * matches anything except separators
                    }
                }
                '.' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '+' | '?' | '\\' => {
                    regex.push('\\');
                    regex.push(c);
                }
                _ => regex.push(c),
            }
        }

        regex.push('$');
        Ok(regex)
    }

    /// Recursive part matching for hierarchical patterns
    fn match_parts(
        &self,
        pattern_parts: &[&str],
        path_parts: &[&str],
        p_idx: usize,
        path_idx: usize,
    ) -> Result<bool> {
        // If we've consumed all pattern parts
        if p_idx >= pattern_parts.len() {
            return Ok(path_idx >= path_parts.len());
        }

        // If we've consumed all path parts but have pattern parts left
        if path_idx >= path_parts.len() {
            return Ok(false);
        }

        let pattern_part = pattern_parts[p_idx];

        match pattern_part {
            "*" => {
                // Single wildcard matches exactly one part
                self.match_parts(pattern_parts, path_parts, p_idx + 1, path_idx + 1)
            }
            "**" => {
                // Double wildcard matches zero or more parts
                // Try matching zero parts
                if self.match_parts(pattern_parts, path_parts, p_idx + 1, path_idx)? {
                    return Ok(true);
                }
                // Try matching one or more parts
                for i in path_idx..path_parts.len() {
                    if self.match_parts(pattern_parts, path_parts, p_idx + 1, i + 1)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => {
                // Exact match required
                if pattern_part == path_parts[path_idx] {
                    self.match_parts(pattern_parts, path_parts, p_idx + 1, path_idx + 1)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

impl Default for HeaderParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exact_match() {
        let mut parser = HeaderParser::new();
        let header = parser
            .parse("template-match: interface.ethernet.eth0")
            .unwrap();

        assert_eq!(header.raw, "template-match: interface.ethernet.eth0");
        assert_eq!(
            header.pattern,
            MatchPattern::HierarchicalPath("interface.ethernet.eth0".to_string())
        );
    }

    #[test]
    fn test_parse_regex_pattern() {
        let mut parser = HeaderParser::new();
        let header = parser.parse("template-match: /^vlan\\.\\d+$/").unwrap();

        assert_eq!(
            header.pattern,
            MatchPattern::Regex("^vlan\\.\\d+$".to_string())
        );
    }

    #[test]
    fn test_parse_glob_pattern() {
        let mut parser = HeaderParser::new();
        let header = parser
            .parse("template-match: bgp.neighbors.*.ipv4")
            .unwrap();

        assert_eq!(
            header.pattern,
            MatchPattern::Glob("bgp.neighbors.*.ipv4".to_string())
        );
    }

    #[test]
    fn test_parse_hierarchical_pattern() {
        let mut parser = HeaderParser::new();
        let header = parser
            .parse("template-match: ospf.area.**.networks")
            .unwrap();

        assert_eq!(
            header.pattern,
            MatchPattern::Glob("ospf.area.**.networks".to_string())
        );
    }

    #[test]
    fn test_exact_match() {
        let mut parser = HeaderParser::new();
        let header = TemplateHeader {
            raw: "test".to_string(),
            pattern: MatchPattern::Exact("interface.eth0".to_string()),
            priority: None,
            scope: None,
        };

        assert!(parser.matches(&header, "interface.eth0").unwrap());
        assert!(!parser.matches(&header, "interface.eth1").unwrap());
    }

    #[test]
    fn test_regex_match() {
        let mut parser = HeaderParser::new();
        let header = TemplateHeader {
            raw: "test".to_string(),
            pattern: MatchPattern::Regex("^vlan\\.\\d+$".to_string()),
            priority: None,
            scope: None,
        };

        assert!(parser.matches(&header, "vlan.100").unwrap());
        assert!(parser.matches(&header, "vlan.1").unwrap());
        assert!(!parser.matches(&header, "vlan.abc").unwrap());
        assert!(!parser.matches(&header, "interface.vlan.100").unwrap());
    }

    #[test]
    fn test_hierarchical_match() {
        let mut parser = HeaderParser::new();
        let header = TemplateHeader {
            raw: "test".to_string(),
            pattern: MatchPattern::HierarchicalPath("interface.*.config".to_string()),
            priority: None,
            scope: None,
        };

        assert!(parser.matches(&header, "interface.eth0.config").unwrap());
        assert!(parser.matches(&header, "interface.vlan100.config").unwrap());
        assert!(!parser.matches(&header, "interface.eth0.status").unwrap());
        assert!(
            !parser
                .matches(&header, "routing.interface.eth0.config")
                .unwrap()
        );
    }

    #[test]
    fn test_glob_match() {
        let mut parser = HeaderParser::new();
        let header = TemplateHeader {
            raw: "test".to_string(),
            pattern: MatchPattern::Glob("bgp.neighbors.**.ipv4".to_string()),
            priority: None,
            scope: None,
        };

        assert!(
            parser
                .matches(&header, "bgp.neighbors.192.168.1.1.ipv4")
                .unwrap()
        );
        assert!(parser.matches(&header, "bgp.neighbors.peer1.ipv4").unwrap());
        assert!(
            parser
                .matches(&header, "bgp.neighbors.group.peer1.ipv4")
                .unwrap()
        );
        assert!(!parser.matches(&header, "bgp.neighbors.peer1.ipv6").unwrap());
    }

    #[test]
    fn test_invalid_patterns() {
        let mut parser = HeaderParser::new();

        // Invalid regex
        assert!(parser.parse("template-match: /[invalid/").is_err());

        // Invalid glob
        assert!(parser.parse("template-match: invalid***pattern").is_err());

        // Empty pattern
        assert!(parser.parse("template-match: ").is_err());

        // Missing directive
        assert!(parser.parse("invalid-header: pattern").is_err());
    }
}
