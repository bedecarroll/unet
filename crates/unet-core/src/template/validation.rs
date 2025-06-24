//! Template validation and security checking

use anyhow::{Context, Result, anyhow};
use minijinja::{Environment, UndefinedBehavior};
use regex::Regex;
use std::collections::HashSet;
use tracing::{debug, warn};

/// Template validator for syntax and security validation
#[derive(Debug)]
pub struct TemplateValidator {
    forbidden_patterns: Vec<Regex>,
    max_template_size: usize,
    max_render_time_ms: u64,
}

impl TemplateValidator {
    /// Create a new template validator with default settings
    pub fn new() -> Self {
        let forbidden_patterns = vec![
            // Prevent file system access
            Regex::new(r"__import__").unwrap(),
            Regex::new(r"open\s*\(").unwrap(),
            Regex::new(r"file\s*\(").unwrap(),
            Regex::new(r"exec\s*\(").unwrap(),
            Regex::new(r"eval\s*\(").unwrap(),
            // Prevent system command execution
            Regex::new(r"system\s*\(").unwrap(),
            Regex::new(r"subprocess").unwrap(),
            Regex::new(r"os\.").unwrap(),
            // Prevent network access
            Regex::new(r"urllib").unwrap(),
            Regex::new(r"requests").unwrap(),
            Regex::new(r"socket").unwrap(),
            // Prevent dangerous built-ins and modules
            Regex::new(r"globals\s*\(").unwrap(),
            Regex::new(r"locals\s*\(").unwrap(),
            Regex::new(r"vars\s*\(").unwrap(),
            Regex::new(r"dir\s*\(").unwrap(),
            Regex::new(r"getattr\s*\(").unwrap(),
            Regex::new(r"setattr\s*\(").unwrap(),
            Regex::new(r"delattr\s*\(").unwrap(),
            Regex::new(r"hasattr\s*\(").unwrap(),
            Regex::new(r"compile\s*\(").unwrap(),
            // Prevent class and module introspection
            Regex::new(r"__class__").unwrap(),
            Regex::new(r"__bases__").unwrap(),
            Regex::new(r"__subclasses__").unwrap(),
            Regex::new(r"__module__").unwrap(),
            Regex::new(r"__globals__").unwrap(),
            Regex::new(r"__dict__").unwrap(),
            // Prevent template injection attempts
            Regex::new(r"\{\{.*\{%").unwrap(),
            Regex::new(r"\{%.*\{\{").unwrap(),
        ];

        Self {
            forbidden_patterns,
            max_template_size: 1024 * 1024, // 1MB
            max_render_time_ms: 5000,       // 5 seconds
        }
    }

    /// Create a validator with custom settings
    pub fn with_settings(max_template_size: usize, max_render_time_ms: u64) -> Self {
        let mut validator = Self::new();
        validator.max_template_size = max_template_size;
        validator.max_render_time_ms = max_render_time_ms;
        validator
    }

    /// Validate template syntax using MiniJinja parser
    pub fn validate_syntax(&self, template_content: &str) -> Result<()> {
        // Check template size
        if template_content.len() > self.max_template_size {
            return Err(anyhow!(
                "Template size ({} bytes) exceeds maximum allowed size ({} bytes)",
                template_content.len(),
                self.max_template_size
            ));
        }

        // Check for forbidden patterns
        self.validate_security(template_content)?;

        // Validate syntax with MiniJinja
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);

        env.template_from_str(template_content)
            .context("Template syntax validation failed")?;

        debug!("Template syntax validation passed");
        Ok(())
    }

    /// Validate template security (check for dangerous patterns)
    pub fn validate_security(&self, template_content: &str) -> Result<()> {
        // Check for forbidden patterns
        for pattern in &self.forbidden_patterns {
            if pattern.is_match(template_content) {
                return Err(anyhow!(
                    "Template contains forbidden pattern: {}",
                    pattern.as_str()
                ));
            }
        }

        // Additional security checks
        self.validate_loop_safety(template_content)?;
        self.validate_recursion_safety(template_content)?;
        self.validate_variable_safety(template_content)?;

        debug!("Template security validation passed");
        Ok(())
    }

    /// Validate that loops are safe and won't cause infinite loops
    fn validate_loop_safety(&self, template_content: &str) -> Result<()> {
        let loop_regex =
            Regex::new(r"\{%\s*for\s+\w+\s+in\s+range\s*\(\s*(\d+)\s*(?:,\s*(\d+))?\s*\)")
                .context("Failed to compile loop regex")?;

        for cap in loop_regex.captures_iter(template_content) {
            let start: u64 = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
            let end: u64 = cap
                .get(2)
                .map(|m| m.as_str().parse().unwrap_or(0))
                .unwrap_or(start);

            let iterations = if end > start { end - start } else { start };

            // Prevent excessive loop iterations (max 10,000)
            if iterations > 10_000 {
                return Err(anyhow!(
                    "Template contains loop with too many iterations: {}",
                    iterations
                ));
            }
        }

        Ok(())
    }

    /// Validate against recursive template includes
    fn validate_recursion_safety(&self, template_content: &str) -> Result<()> {
        // Count nested blocks to prevent deep recursion
        let mut block_depth: i32 = 0;
        let mut max_depth: i32 = 0;

        // Process character by character to handle multiple blocks on the same line
        let chars: Vec<char> = template_content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '%' {
                // Found start of block, check if it's an end block
                let mut j = i + 2;
                while j < chars.len() && chars[j] != '%' {
                    j += 1;
                }
                if j + 1 < chars.len() && chars[j] == '%' && chars[j + 1] == '}' {
                    let block_content: String = chars[i + 2..j].iter().collect();
                    if block_content.trim().starts_with("end") {
                        block_depth = block_depth.saturating_sub(1);
                    } else {
                        block_depth += 1;
                        max_depth = max_depth.max(block_depth);
                    }
                    i = j + 2;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // Limit maximum nesting depth to prevent stack overflow
        if max_depth > 20 {
            return Err(anyhow!(
                "Template has excessive nesting depth: {} (max allowed: 20)",
                max_depth
            ));
        }

        Ok(())
    }

    /// Validate variable access safety
    fn validate_variable_safety(&self, template_content: &str) -> Result<()> {
        // Check for suspicious variable access patterns
        if template_content.contains("__") {
            warn!("Template contains double underscore variables which may be dangerous");
        }

        // Check for attempts to access request/response objects
        let unsafe_vars = ["request", "response", "session", "config", "app"];
        for var in unsafe_vars {
            if template_content.contains(var) {
                warn!(
                    "Template contains potentially unsafe variable access: {}",
                    var
                );
            }
        }

        Ok(())
    }

    /// Validate template with comprehensive checks
    pub fn validate_comprehensive(&self, template_content: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();

        // Basic syntax validation
        match self.validate_syntax(template_content) {
            Ok(_) => result.syntax_valid = true,
            Err(e) => {
                result.syntax_valid = false;
                result.errors.push(format!("Syntax error: {}", e));
            }
        }

        // Security validation
        match self.validate_security(template_content) {
            Ok(_) => result.security_valid = true,
            Err(e) => {
                result.security_valid = false;
                result.errors.push(format!("Security error: {}", e));
            }
        }

        // Analyze template complexity
        result.complexity = self.analyze_complexity(template_content);

        // Extract used variables
        result.variables = self.extract_variables(template_content)?;

        // Extract used filters
        result.filters = self.extract_filters(template_content)?;

        result.overall_valid = result.syntax_valid && result.security_valid;

        Ok(result)
    }

    /// Analyze template complexity
    pub fn analyze_complexity(&self, template_content: &str) -> TemplateComplexity {
        let lines = template_content.lines().count();
        let blocks = template_content.matches("{% ").count();
        let variables = template_content.matches("{{ ").count();
        let filters = template_content.matches(" | ").count();

        let complexity_score = lines + (blocks * 2) + variables + (filters / 2);

        if complexity_score > 200 {
            TemplateComplexity::High
        } else if complexity_score > 50 {
            TemplateComplexity::Medium
        } else {
            TemplateComplexity::Low
        }
    }

    /// Extract variables used in template
    fn extract_variables(&self, template_content: &str) -> Result<Vec<String>> {
        let var_regex = Regex::new(r"\{\{\s*([^}|]+)(?:\s*\|[^}]*)?\s*\}\}")
            .context("Failed to compile variable regex")?;

        let mut variables = HashSet::new();

        for cap in var_regex.captures_iter(template_content) {
            if let Some(var) = cap.get(1) {
                let var_name = var.as_str().trim();
                // Extract just the base variable name (ignore attribute access)
                let base_name = var_name.split('.').next().unwrap_or(var_name);
                variables.insert(base_name.to_string());
            }
        }

        let mut result: Vec<String> = variables.into_iter().collect();
        result.sort();
        Ok(result)
    }

    /// Extract filters used in template
    fn extract_filters(&self, template_content: &str) -> Result<Vec<String>> {
        let filter_regex = Regex::new(r"\|\s*([a-zA-Z_][a-zA-Z0-9_]*)")
            .context("Failed to compile filter regex")?;

        let mut filters = HashSet::new();

        for cap in filter_regex.captures_iter(template_content) {
            if let Some(filter) = cap.get(1) {
                filters.insert(filter.as_str().to_string());
            }
        }

        let mut result: Vec<String> = filters.into_iter().collect();
        result.sort();
        Ok(result)
    }
}

impl Default for TemplateValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Template validation result
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Whether template syntax is valid
    pub syntax_valid: bool,
    /// Whether template passes security checks
    pub security_valid: bool,
    /// Overall validation status
    pub overall_valid: bool,
    /// List of validation errors
    pub errors: Vec<String>,
    /// Template complexity assessment
    pub complexity: TemplateComplexity,
    /// Variables used in template
    pub variables: Vec<String>,
    /// Filters used in template
    pub filters: Vec<String>,
}

/// Template complexity levels
#[derive(Debug, Default, PartialEq, serde::Serialize)]
pub enum TemplateComplexity {
    #[default]
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = TemplateValidator::new();
        assert!(!validator.forbidden_patterns.is_empty());
        assert!(validator.max_template_size > 0);
    }

    #[test]
    fn test_valid_template_syntax() {
        let validator = TemplateValidator::new();
        let template = "Hello {{ name }}! Your interface is {{ interface | cisco_interface }}.";

        assert!(validator.validate_syntax(template).is_ok());
    }

    #[test]
    fn test_invalid_template_syntax() {
        let validator = TemplateValidator::new();
        let template = "Hello {{ name }! Unclosed variable.";

        assert!(validator.validate_syntax(template).is_err());
    }

    #[test]
    fn test_security_validation_pass() {
        let validator = TemplateValidator::new();
        let template = "interface {{ interface }}\n ip address {{ ip_address }}";

        assert!(validator.validate_security(template).is_ok());
    }

    #[test]
    fn test_security_validation_fail() {
        let validator = TemplateValidator::new();
        let template = "{{ __import__('os').system('rm -rf /') }}";

        assert!(validator.validate_security(template).is_err());
    }

    #[test]
    fn test_template_size_limit() {
        let validator = TemplateValidator::with_settings(100, 1000);
        let large_template = "x".repeat(200);

        assert!(validator.validate_syntax(&large_template).is_err());
    }

    #[test]
    fn test_extract_variables() {
        let validator = TemplateValidator::new();
        let template =
            "Hello {{ name }}! Your {{ device.type }} has IP {{ device.management_ip }}.";

        let variables = validator.extract_variables(template).unwrap();
        assert!(variables.contains(&"name".to_string()));
        assert!(variables.contains(&"device".to_string()));
        assert_eq!(variables.len(), 2);
    }

    #[test]
    fn test_extract_filters() {
        let validator = TemplateValidator::new();
        let template =
            "{{ interface | cisco_interface | upper }} has IP {{ ip | ip_network(24) }}.";

        let filters = validator.extract_filters(template).unwrap();
        assert!(filters.contains(&"cisco_interface".to_string()));
        assert!(filters.contains(&"upper".to_string()));
        assert!(filters.contains(&"ip_network".to_string()));
    }

    #[test]
    fn test_comprehensive_validation() {
        let validator = TemplateValidator::new();
        let template = "interface {{ interface | cisco_interface }}\n ip address {{ ip }}";

        let result = validator.validate_comprehensive(template).unwrap();
        assert!(result.overall_valid);
        assert!(result.syntax_valid);
        assert!(result.security_valid);
        assert!(!result.variables.is_empty());
        assert!(!result.filters.is_empty());
    }

    #[test]
    fn test_complexity_analysis() {
        let validator = TemplateValidator::new();

        // Simple template
        let simple = "Hello {{ name }}!";
        assert_eq!(
            validator.analyze_complexity(simple),
            TemplateComplexity::Low
        );

        // Complex template (increase complexity to ensure it's classified as High)
        let complex = format!(
            "{}\n",
            "{% for i in range(100) %}{{ i }}{% endfor %}".repeat(50)
        );
        assert_eq!(
            validator.analyze_complexity(&complex),
            TemplateComplexity::High
        );
    }

    #[test]
    fn test_loop_safety_validation() {
        let validator = TemplateValidator::new();

        // Safe loop
        let safe_loop = "{% for i in range(100) %}{{ i }}{% endfor %}";
        assert!(validator.validate_loop_safety(safe_loop).is_ok());

        // Unsafe loop (too many iterations)
        let unsafe_loop = "{% for i in range(100000) %}{{ i }}{% endfor %}";
        assert!(validator.validate_loop_safety(unsafe_loop).is_err());
    }

    #[test]
    fn test_recursion_safety_validation() {
        let validator = TemplateValidator::new();

        // Safe nesting
        let safe_template =
            "{% if condition %}{% for item in items %}{{ item }}{% endfor %}{% endif %}";
        assert!(validator.validate_recursion_safety(safe_template).is_ok());

        // Unsafe deep nesting (21 levels of nesting)
        let mut deep_template = String::new();
        for i in 0..22 {
            deep_template.push_str(&format!("{{% if condition{} %}}", i));
        }
        deep_template.push_str("deep content");
        for _i in 0..22 {
            deep_template.push_str("{% endif %}");
        }
        assert!(validator.validate_recursion_safety(&deep_template).is_err());
    }

    #[test]
    fn test_variable_safety_validation() {
        let validator = TemplateValidator::new();

        // Safe variables
        let safe_template = "{{ name }} {{ interface }}";
        assert!(validator.validate_variable_safety(safe_template).is_ok());

        // Potentially unsafe variables (should warn but not fail)
        let unsafe_template = "{{ request.user }} {{ __class__ }}";
        assert!(validator.validate_variable_safety(unsafe_template).is_ok());
    }

    #[test]
    fn test_enhanced_security_patterns() {
        let validator = TemplateValidator::new();

        // Test new forbidden patterns
        let dangerous_templates = vec![
            "{{ globals() }}",
            "{{ locals() }}",
            "{{ getattr(obj, 'attr') }}",
            "{{ __class__ }}",
            "{{ __globals__ }}",
            "{{ compile('code', 'file', 'exec') }}",
        ];

        for template in dangerous_templates {
            assert!(
                validator.validate_security(template).is_err(),
                "Template should be rejected: {}",
                template
            );
        }
    }
}
