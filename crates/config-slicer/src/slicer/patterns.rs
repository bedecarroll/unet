//! Pattern matching implementations for configuration slicing
//!
//! This module provides various pattern matching capabilities including glob patterns,
//! regex patterns, and hierarchical patterns for extracting configuration slices.

use crate::error::{Error, Result};
use crate::slicer::{ContextMatcher, HierarchicalPattern};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Different types of patterns supported for slice extraction
#[derive(Debug, Clone, PartialEq)]
pub enum SlicePattern {
    /// Glob-style pattern matching (e.g., "interface*", "vlan?0?")
    Glob(GlobPattern),
    /// Regular expression pattern matching
    Regex(RegexPattern),
    /// Hierarchical path pattern matching
    Hierarchical(HierarchicalPattern),
    /// Context-based pattern matching
    Context(ContextMatcher),
}

/// Glob pattern for simple wildcard matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobPattern {
    /// The glob pattern string
    pub pattern: String,
    /// Whether matching should be case-sensitive
    pub case_sensitive: bool,
    /// Compiled regex for efficient matching
    #[serde(skip)]
    regex: Option<Regex>,
}

impl PartialEq for GlobPattern {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern && self.case_sensitive == other.case_sensitive
    }
}

impl GlobPattern {
    /// Create a new glob pattern
    pub fn new(pattern: &str) -> Result<Self> {
        let mut glob = Self {
            pattern: pattern.to_string(),
            case_sensitive: true,
            regex: None,
        };
        glob.compile()?;
        Ok(glob)
    }

    /// Create a case-insensitive glob pattern
    pub fn new_case_insensitive(pattern: &str) -> Result<Self> {
        let mut glob = Self {
            pattern: pattern.to_string(),
            case_sensitive: false,
            regex: None,
        };
        glob.compile()?;
        Ok(glob)
    }

    /// Compile the glob pattern to a regex
    fn compile(&mut self) -> Result<()> {
        let regex_pattern = glob_to_regex(&self.pattern, self.case_sensitive)?;
        self.regex = Some(
            Regex::new(&regex_pattern)
                .map_err(|e| Error::InvalidPattern(format!("Invalid glob pattern: {e}")))?,
        );
        Ok(())
    }

    /// Check if the pattern matches the given text
    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        if let Some(ref regex) = self.regex {
            regex.is_match(text)
        } else {
            false
        }
    }

    /// Get the original pattern string
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Check if the pattern is case-sensitive
    #[must_use]
    pub const fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }
}

/// Regular expression pattern for complex matching
#[derive(Debug, Clone)]
pub struct RegexPattern {
    /// The regex pattern string
    pub pattern: String,
    /// Compiled regex for efficient matching
    regex: Regex,
}

impl PartialEq for RegexPattern {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl RegexPattern {
    /// Create a new regex pattern
    pub fn new(pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| Error::InvalidPattern(format!("Invalid regex pattern: {e}")))?;

        Ok(Self {
            pattern: pattern.to_string(),
            regex,
        })
    }

    /// Check if the pattern matches the given text
    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    /// Get the original pattern string
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Get capture groups from a match
    #[must_use]
    pub fn captures<'a>(&self, text: &'a str) -> Option<regex::Captures<'a>> {
        self.regex.captures(text)
    }

    /// Find all matches in the text
    #[must_use]
    pub fn find_all(&self, text: &str) -> Vec<String> {
        self.regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

/// Generic pattern matching trait
pub trait PatternMatcher {
    /// Check if the pattern matches the given text
    fn matches(&self, text: &str) -> bool;

    /// Get a description of the pattern
    fn description(&self) -> String;
}

impl PatternMatcher for GlobPattern {
    fn matches(&self, text: &str) -> bool {
        self.matches(text)
    }

    fn description(&self) -> String {
        format!("Glob pattern: '{}'", self.pattern)
    }
}

impl PatternMatcher for RegexPattern {
    fn matches(&self, text: &str) -> bool {
        self.matches(text)
    }

    fn description(&self) -> String {
        format!("Regex pattern: '{}'", self.pattern)
    }
}

/// Convert a glob pattern to a regular expression
fn glob_to_regex(pattern: &str, case_sensitive: bool) -> Result<String> {
    let mut regex_pattern = String::new();
    let mut chars = pattern.chars().peekable();

    regex_pattern.push('^');

    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                // Match any sequence of characters
                regex_pattern.push_str(".*");
            }
            '?' => {
                // Match any single character
                regex_pattern.push('.');
            }
            '[' => {
                // Character class - copy until closing bracket
                regex_pattern.push('[');
                for class_ch in chars.by_ref() {
                    regex_pattern.push(class_ch);
                    if class_ch == ']' {
                        break;
                    }
                }
            }
            '\\' => {
                // Escape character
                if let Some(escaped) = chars.next() {
                    regex_pattern.push('\\');
                    regex_pattern.push(escaped);
                } else {
                    regex_pattern.push('\\');
                }
            }
            // Escape regex special characters
            '^' | '$' | '.' | '+' | '(' | ')' | '{' | '}' | '|' => {
                regex_pattern.push('\\');
                regex_pattern.push(ch);
            }
            _ => {
                regex_pattern.push(ch);
            }
        }
    }

    regex_pattern.push('$');

    if !case_sensitive {
        regex_pattern = format!("(?i){regex_pattern}");
    }

    Ok(regex_pattern)
}

/// Pattern builder for creating complex patterns
pub struct PatternBuilder {
    pattern_type: PatternType,
    pattern_string: String,
    case_sensitive: bool,
}

enum PatternType {
    Glob,
    Regex,
}

impl PatternBuilder {
    /// Start building a glob pattern
    #[must_use]
    pub fn glob(pattern: &str) -> Self {
        Self {
            pattern_type: PatternType::Glob,
            pattern_string: pattern.to_string(),
            case_sensitive: true,
        }
    }

    /// Start building a regex pattern
    #[must_use]
    pub fn regex(pattern: &str) -> Self {
        Self {
            pattern_type: PatternType::Regex,
            pattern_string: pattern.to_string(),
            case_sensitive: true,
        }
    }

    /// Set case sensitivity
    #[must_use]
    pub const fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Build the pattern
    pub fn build(self) -> Result<SlicePattern> {
        match self.pattern_type {
            PatternType::Glob => {
                let pattern = if self.case_sensitive {
                    GlobPattern::new(&self.pattern_string)?
                } else {
                    GlobPattern::new_case_insensitive(&self.pattern_string)?
                };
                Ok(SlicePattern::Glob(pattern))
            }
            PatternType::Regex => {
                let mut pattern_str = self.pattern_string;
                if !self.case_sensitive {
                    pattern_str = format!("(?i){pattern_str}");
                }
                let pattern = RegexPattern::new(&pattern_str)?;
                Ok(SlicePattern::Regex(pattern))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_pattern_basic() {
        let pattern = GlobPattern::new("interface*").unwrap();

        assert!(pattern.matches("interface GigabitEthernet0/1"));
        assert!(pattern.matches("interface"));
        assert!(!pattern.matches("hostname test"));
    }

    #[test]
    fn test_glob_pattern_question_mark() {
        let pattern = GlobPattern::new("vlan?0?").unwrap();

        assert!(pattern.matches("vlan101"));
        assert!(pattern.matches("vlan202"));
        assert!(!pattern.matches("vlan1"));
        assert!(!pattern.matches("vlan1000"));
    }

    #[test]
    fn test_glob_pattern_case_insensitive() {
        let pattern = GlobPattern::new_case_insensitive("INTERFACE*").unwrap();

        assert!(pattern.matches("interface GigabitEthernet0/1"));
        assert!(pattern.matches("Interface FastEthernet0/1"));
        assert!(pattern.matches("INTERFACE TenGigE0/1"));
    }

    #[test]
    fn test_regex_pattern() {
        let pattern = RegexPattern::new(r"^interface\s+\w+\d+/\d+$").unwrap();

        assert!(pattern.matches("interface GigabitEthernet0/1"));
        assert!(pattern.matches("interface FastEthernet1/2"));
        assert!(!pattern.matches("interface"));
        assert!(!pattern.matches("hostname test"));
    }

    #[test]
    fn test_regex_pattern_captures() {
        let pattern = RegexPattern::new(r"^interface\s+(\w+)(\d+/\d+)$").unwrap();

        let captures = pattern.captures("interface GigabitEthernet0/1").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "GigabitEthernet");
        assert_eq!(captures.get(2).unwrap().as_str(), "0/1");
    }

    #[test]
    fn test_pattern_builder_glob() {
        let pattern = PatternBuilder::glob("interface*")
            .case_sensitive(false)
            .build()
            .unwrap();

        if let SlicePattern::Glob(glob) = pattern {
            assert!(!glob.is_case_sensitive());
            assert!(glob.matches("INTERFACE test"));
        } else {
            panic!("Expected glob pattern");
        }
    }

    #[test]
    fn test_pattern_builder_regex() {
        let pattern = PatternBuilder::regex(r"interface\s+\w+")
            .case_sensitive(false)
            .build()
            .unwrap();

        if let SlicePattern::Regex(regex) = pattern {
            assert!(regex.matches("INTERFACE test"));
            assert!(regex.matches("interface TEST"));
        } else {
            panic!("Expected regex pattern");
        }
    }

    #[test]
    fn test_glob_to_regex_conversion() {
        assert_eq!(glob_to_regex("*", true).unwrap(), "^.*$");
        assert_eq!(glob_to_regex("?", true).unwrap(), "^.$");
        assert_eq!(glob_to_regex("test*", true).unwrap(), "^test.*$");
        assert_eq!(glob_to_regex("test?", true).unwrap(), "^test.$");
        assert_eq!(glob_to_regex("test.", true).unwrap(), r"^test\.$");
    }

    #[test]
    fn test_pattern_matcher_trait() {
        let glob: Box<dyn PatternMatcher> = Box::new(GlobPattern::new("test*").unwrap());
        let regex: Box<dyn PatternMatcher> = Box::new(RegexPattern::new("test.*").unwrap());

        assert!(glob.matches("test123"));
        assert!(regex.matches("test123"));

        assert!(glob.description().contains("Glob pattern"));
        assert!(regex.description().contains("Regex pattern"));
    }
}
