//! Template quality analysis and validation tools

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use super::{
    HeaderParser, TemplateComplexity, TemplateEngine, TemplateEnvironment, TemplateHeader,
    TemplateValidator,
};

/// Comprehensive template quality analyzer
#[derive(Debug)]
pub struct TemplateQualityAnalyzer {
    /// Template engine for validation
    engine: TemplateEngine,
    /// Template validator for syntax checking
    validator: TemplateValidator,
    /// Header parser for metadata extraction  
    header_parser: HeaderParser,
    /// Template environment for advanced analysis
    environment: TemplateEnvironment,
    /// Configuration for quality analysis
    config: QualityAnalysisConfig,
}

/// Configuration for template quality analysis
#[derive(Debug, Clone)]
pub struct QualityAnalysisConfig {
    /// Enable template linting
    pub enable_linting: bool,
    /// Enable performance analysis
    pub enable_performance_analysis: bool,
    /// Enable security scanning
    pub enable_security_scanning: bool,
    /// Enable documentation generation
    pub enable_documentation_generation: bool,
    /// Maximum template complexity score allowed
    pub max_complexity_score: u32,
    /// Maximum template size in bytes
    pub max_template_size: usize,
    /// Timeout for individual template analysis
    pub analysis_timeout: Duration,
    /// Include detailed explanations in results
    pub verbose_output: bool,
}

impl Default for QualityAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_linting: true,
            enable_performance_analysis: true,
            enable_security_scanning: true,
            enable_documentation_generation: true,
            max_complexity_score: 100,
            max_template_size: 1024 * 1024, // 1MB
            analysis_timeout: Duration::from_secs(30),
            verbose_output: false,
        }
    }
}

impl TemplateQualityAnalyzer {
    /// Create a new template quality analyzer
    pub fn new() -> Result<Self> {
        Self::with_config(QualityAnalysisConfig::default())
    }

    /// Create a new template quality analyzer with custom configuration
    pub fn with_config(config: QualityAnalysisConfig) -> Result<Self> {
        let engine = TemplateEngine::with_timeout(config.analysis_timeout)?;
        let validator = TemplateValidator::new();
        let header_parser = HeaderParser::new();
        let environment = TemplateEnvironment::new()?;

        Ok(Self {
            engine,
            validator,
            header_parser,
            environment,
            config,
        })
    }

    /// Analyze a single template file
    pub async fn analyze_template<P: AsRef<Path>>(
        &self,
        template_path: P,
    ) -> Result<TemplateQualityReport> {
        let template_path = template_path.as_ref();
        let start_time = Instant::now();

        info!("Analyzing template: {}", template_path.display());

        // Read template content
        let template_content = fs::read_to_string(template_path)
            .with_context(|| format!("Failed to read template: {}", template_path.display()))?;

        // Check template size
        if template_content.len() > self.config.max_template_size {
            return Err(anyhow!(
                "Template size {} bytes exceeds maximum allowed size {} bytes",
                template_content.len(),
                self.config.max_template_size
            ));
        }

        let mut report = TemplateQualityReport::new(template_path.to_path_buf());

        // Template linting
        if self.config.enable_linting {
            debug!("Running template linting");
            report.linting_results = Some(self.lint_template(&template_content).await?);
        }

        // Performance analysis
        if self.config.enable_performance_analysis {
            debug!("Running performance analysis");
            report.performance_analysis = Some(self.analyze_performance(&template_content).await?);
        }

        // Security scanning
        if self.config.enable_security_scanning {
            debug!("Running security scanning");
            report.security_scan = Some(self.scan_security(&template_content).await?);
        }

        // Documentation generation
        if self.config.enable_documentation_generation {
            debug!("Generating documentation");
            report.documentation = Some(self.generate_documentation(&template_content).await?);
        }

        report.analysis_duration = start_time.elapsed();
        report.calculate_overall_score();

        info!(
            "Template analysis completed in {:?} - Overall score: {}/100",
            report.analysis_duration, report.overall_score
        );

        Ok(report)
    }

    /// Analyze multiple templates in a directory
    pub async fn analyze_directory<P: AsRef<Path>>(
        &self,
        dir_path: P,
    ) -> Result<DirectoryQualityReport> {
        let dir_path = dir_path.as_ref();
        let start_time = Instant::now();

        info!("Analyzing templates in directory: {}", dir_path.display());

        let mut directory_report = DirectoryQualityReport::new(dir_path.to_path_buf());

        // Find all template files
        let template_files = self.find_template_files(dir_path)?;

        info!("Found {} template files to analyze", template_files.len());

        // Analyze each template
        for template_file in template_files {
            match self.analyze_template(&template_file).await {
                Ok(report) => {
                    directory_report.template_reports.push(report);
                }
                Err(e) => {
                    warn!("Failed to analyze {}: {}", template_file.display(), e);
                    directory_report
                        .failed_analyses
                        .push((template_file, e.to_string()));
                }
            }
        }

        directory_report.analysis_duration = start_time.elapsed();
        directory_report.calculate_summary();

        info!(
            "Directory analysis completed in {:?} - {} templates analyzed, {} failed",
            directory_report.analysis_duration,
            directory_report.template_reports.len(),
            directory_report.failed_analyses.len()
        );

        Ok(directory_report)
    }

    /// Lint a template for best practices and common issues
    async fn lint_template(&self, template_content: &str) -> Result<TemplateLintingResults> {
        let mut results = TemplateLintingResults::new();

        // Syntax validation
        match self.validator.validate_syntax(template_content) {
            Ok(_) => results.syntax_valid = true,
            Err(e) => {
                results.syntax_valid = false;
                results.errors.push(LintError::new(
                    LintErrorSeverity::Error,
                    "syntax_error".to_string(),
                    e.to_string(),
                    None,
                ));
            }
        }

        // Template complexity analysis
        let complexity = self.validator.analyze_complexity(template_content);
        results.complexity_score = Self::complexity_to_score(&complexity);

        if results.complexity_score > self.config.max_complexity_score {
            results.warnings.push(LintWarning::new(
                "high_complexity".to_string(),
                format!(
                    "Template complexity score {} exceeds recommended maximum {}",
                    results.complexity_score, self.config.max_complexity_score
                ),
                Some(
                    "Consider breaking down complex templates into smaller, reusable components"
                        .to_string(),
                ),
            ));
        }

        // Best practices checks
        self.check_best_practices(template_content, &mut results)?;

        // Naming conventions
        self.check_naming_conventions(template_content, &mut results)?;

        // Performance patterns
        self.check_performance_patterns(template_content, &mut results)?;

        results.total_issues = results.errors.len() + results.warnings.len();

        Ok(results)
    }

    /// Analyze template performance characteristics
    async fn analyze_performance(
        &self,
        template_content: &str,
    ) -> Result<TemplatePerformanceAnalysis> {
        let mut analysis = TemplatePerformanceAnalysis::new();

        // Template size metrics
        analysis.template_size_bytes = template_content.len();
        analysis.line_count = template_content.lines().count();

        // Compilation time measurement
        let compile_start = Instant::now();
        match self.engine.compile_template("perf_test", template_content) {
            Ok(_) => {
                analysis.compilation_time = Some(compile_start.elapsed());
                analysis.compilation_successful = true;
            }
            Err(e) => {
                analysis.compilation_successful = false;
                analysis.compilation_error = Some(e.to_string());
            }
        }

        // Rendering performance estimation
        if analysis.compilation_successful {
            analysis.rendering_estimates =
                Some(self.estimate_rendering_performance(template_content)?);
        }

        // Memory usage estimation
        analysis.estimated_memory_usage = self.estimate_memory_usage(template_content);

        // Optimization suggestions
        analysis.optimization_suggestions =
            self.generate_optimization_suggestions(template_content);

        Ok(analysis)
    }

    /// Scan template for security issues
    async fn scan_security(&self, template_content: &str) -> Result<TemplateSecurityScan> {
        let mut scan = TemplateSecurityScan::new();

        // Check for dangerous patterns
        scan.dangerous_patterns = self.check_dangerous_patterns(template_content)?;

        // Validate template security
        match self.validator.validate_security(template_content) {
            Ok(_) => scan.security_validation_passed = true,
            Err(e) => {
                scan.security_validation_passed = false;
                scan.security_issues.push(SecurityIssue::new(
                    SecuritySeverity::High,
                    "security_validation_failed".to_string(),
                    e.to_string(),
                    Some("Template failed security validation checks".to_string()),
                ));
            }
        }

        // Check for sensitive data exposure
        scan.sensitive_data_risks = self.check_sensitive_data_exposure(template_content)?;

        // Input validation checks
        scan.input_validation_issues = self.check_input_validation(template_content)?;

        scan.total_security_issues = scan.security_issues.len()
            + scan.dangerous_patterns.len()
            + scan.sensitive_data_risks.len()
            + scan.input_validation_issues.len();

        Ok(scan)
    }

    /// Generate documentation for a template
    async fn generate_documentation(
        &self,
        template_content: &str,
    ) -> Result<TemplateDocumentation> {
        let mut documentation = TemplateDocumentation::new();

        // Parse template header for metadata
        // Extract header from template content
        if let Some(header_line) = self.extract_header_line(template_content) {
            let mut parser = HeaderParser::new();
            if let Ok(header) = parser.parse(&header_line) {
                documentation.metadata = Some(header);
            }
        }

        // Extract variables and their types
        documentation.variables = self.extract_template_variables(template_content)?;

        // Generate usage examples
        documentation.usage_examples = self.generate_usage_examples(template_content)?;

        // Extract template structure
        documentation.template_structure = self.analyze_template_structure(template_content)?;

        // Generate markdown documentation
        documentation.markdown_content = self.generate_markdown_documentation(&documentation)?;

        Ok(documentation)
    }

    /// Check template best practices
    fn check_best_practices(
        &self,
        template_content: &str,
        results: &mut TemplateLintingResults,
    ) -> Result<()> {
        // Check for hardcoded values
        let hardcoded_ip_regex = Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}")?;
        if hardcoded_ip_regex.is_match(template_content) {
            results.warnings.push(LintWarning::new(
                "hardcoded_ip".to_string(),
                "Template contains hardcoded IP addresses".to_string(),
                Some(
                    "Consider using variables for IP addresses to improve template reusability"
                        .to_string(),
                ),
            ));
        }

        // Check for proper indentation
        let lines: Vec<&str> = template_content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if line.starts_with('\t') && line.contains(' ') {
                results.warnings.push(LintWarning::new(
                    "mixed_indentation".to_string(),
                    format!(
                        "Line {} uses mixed tabs and spaces for indentation",
                        line_num + 1
                    ),
                    Some(
                        "Use consistent indentation (either tabs or spaces, not both)".to_string(),
                    ),
                ));
            }
        }

        // Check for template header presence
        if !template_content.contains("template-match:") {
            results.warnings.push(LintWarning::new(
                "missing_header".to_string(),
                "Template is missing template-match header".to_string(),
                Some(
                    "Add template-match header to specify template scope and matching criteria"
                        .to_string(),
                ),
            ));
        }

        Ok(())
    }

    /// Check naming conventions
    fn check_naming_conventions(
        &self,
        template_content: &str,
        results: &mut TemplateLintingResults,
    ) -> Result<()> {
        let variable_regex = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}")?;

        for captures in variable_regex.captures_iter(template_content) {
            if let Some(var_name) = captures.get(1) {
                let name = var_name.as_str();

                // Check for camelCase (should be snake_case)
                if name.chars().any(|c| c.is_uppercase()) {
                    results.warnings.push(LintWarning::new(
                        "camel_case_variable".to_string(),
                        format!("Variable '{}' uses camelCase, prefer snake_case", name),
                        Some("Use snake_case for variable names (e.g., 'my_variable' instead of 'myVariable')".to_string()),
                    ));
                }

                // Check for single letter variables
                if name.len() == 1 {
                    results.warnings.push(LintWarning::new(
                        "single_char_variable".to_string(),
                        format!("Variable '{}' is a single character", name),
                        Some(
                            "Use descriptive variable names instead of single characters"
                                .to_string(),
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check for performance anti-patterns
    fn check_performance_patterns(
        &self,
        template_content: &str,
        results: &mut TemplateLintingResults,
    ) -> Result<()> {
        // Check for nested loops
        let loop_count = template_content.matches("{% for").count();
        if loop_count > 3 {
            results.warnings.push(LintWarning::new(
                "excessive_loops".to_string(),
                format!("Template contains {} nested loops, which may impact performance", loop_count),
                Some("Consider simplifying template logic or pre-processing data to reduce loop nesting".to_string()),
            ));
        }

        // Check for complex filters in loops
        if template_content.contains("{% for") && template_content.contains("|") {
            let filter_in_loop_regex =
                Regex::new(r"\{\%\s*for.*\n[\s\S]*?\{\{.*\|.*\}\}[\s\S]*?\{\%\s*endfor")?;
            if filter_in_loop_regex.is_match(template_content) {
                results.warnings.push(LintWarning::new(
                    "filter_in_loop".to_string(),
                    "Complex filters used inside loops may impact performance".to_string(),
                    Some(
                        "Consider pre-processing data with filters before the template loop"
                            .to_string(),
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Check for dangerous patterns that could cause security issues
    fn check_dangerous_patterns(&self, template_content: &str) -> Result<Vec<DangerousPattern>> {
        let mut patterns = Vec::new();

        // Check for potential command injection patterns
        let command_patterns = [
            r"\|\s*sh\b",
            r"\|\s*bash\b",
            r"\|\s*exec\b",
            r"system\s*\(",
            r"eval\s*\(",
        ];

        for pattern in &command_patterns {
            let regex = Regex::new(pattern)?;
            if regex.is_match(template_content) {
                patterns.push(DangerousPattern::new(
                    "command_injection_risk".to_string(),
                    format!(
                        "Template contains potentially dangerous pattern: {}",
                        pattern
                    ),
                    SecuritySeverity::High,
                ));
            }
        }

        // Check for file system access patterns
        let file_patterns = [
            r"file\s*\(",
            r"open\s*\(",
            r"read\s*\(",
            r"write\s*\(",
            r"/etc/",
            r"/var/",
            r"\.\./",
        ];

        for pattern in &file_patterns {
            let regex = Regex::new(pattern)?;
            if regex.is_match(template_content) {
                patterns.push(DangerousPattern::new(
                    "file_system_access".to_string(),
                    format!("Template may attempt file system access: {}", pattern),
                    SecuritySeverity::Medium,
                ));
            }
        }

        Ok(patterns)
    }

    /// Check for sensitive data exposure risks
    fn check_sensitive_data_exposure(
        &self,
        template_content: &str,
    ) -> Result<Vec<SensitiveDataRisk>> {
        let mut risks = Vec::new();

        // Check for potential credential patterns
        let credential_patterns = [
            (
                r"(?i)password\s*[=:]\s*['\x22]?[^'\x22\s]+",
                "password_in_template",
            ),
            (
                r"(?i)secret\s*[=:]\s*['\x22]?[^'\x22\s]+",
                "secret_in_template",
            ),
            (r"(?i)key\s*[=:]\s*['\x22]?[^'\x22\s]+", "key_in_template"),
            (
                r"(?i)token\s*[=:]\s*['\x22]?[^'\x22\s]+",
                "token_in_template",
            ),
        ];

        for (pattern, risk_type) in &credential_patterns {
            let regex = Regex::new(pattern)?;
            if regex.is_match(template_content) {
                risks.push(SensitiveDataRisk::new(
                    risk_type.to_string(),
                    format!(
                        "Template may contain hardcoded credentials matching pattern: {}",
                        pattern
                    ),
                    SecuritySeverity::High,
                ));
            }
        }

        Ok(risks)
    }

    /// Check input validation patterns
    fn check_input_validation(&self, template_content: &str) -> Result<Vec<InputValidationIssue>> {
        let mut issues = Vec::new();

        // Check for unescaped output
        let unescaped_regex = Regex::new(r"\{\{\s*[^}|]*\s*\}\}")?;
        let escaped_regex = Regex::new(r"\{\{\s*[^}]*\|\s*escape\s*\}\}")?;

        let unescaped_count = unescaped_regex.find_iter(template_content).count();
        let escaped_count = escaped_regex.find_iter(template_content).count();

        if unescaped_count > escaped_count * 2 {
            issues.push(InputValidationIssue::new(
                "unescaped_output".to_string(),
                "Template has many unescaped output expressions".to_string(),
                SecuritySeverity::Medium,
                Some("Consider using the 'escape' filter for user-provided data".to_string()),
            ));
        }

        Ok(issues)
    }

    /// Estimate rendering performance characteristics
    fn estimate_rendering_performance(
        &self,
        template_content: &str,
    ) -> Result<RenderingPerformanceEstimate> {
        let mut estimate = RenderingPerformanceEstimate::new();

        // Count template constructs that affect performance
        estimate.variable_count = Regex::new(r"\{\{[^}]+\}\}")?
            .find_iter(template_content)
            .count();
        estimate.loop_count = template_content.matches("{% for").count();
        estimate.condition_count = template_content.matches("{% if").count();
        estimate.filter_count = template_content.matches('|').count();

        // Estimate based on template complexity
        let base_time = Duration::from_micros(100); // Base rendering time
        let variable_factor = estimate.variable_count as u64 * 10;
        let loop_factor = estimate.loop_count as u64 * 1000;
        let condition_factor = estimate.condition_count as u64 * 50;
        let filter_factor = estimate.filter_count as u64 * 100;

        estimate.estimated_render_time = base_time
            + Duration::from_micros(
                variable_factor + loop_factor + condition_factor + filter_factor,
            );

        // Performance category
        estimate.performance_category = if estimate.estimated_render_time < Duration::from_millis(1)
        {
            "fast".to_string()
        } else if estimate.estimated_render_time < Duration::from_millis(10) {
            "moderate".to_string()
        } else {
            "slow".to_string()
        };

        Ok(estimate)
    }

    /// Estimate memory usage for template
    fn estimate_memory_usage(&self, template_content: &str) -> usize {
        // Base memory for template storage
        let base_memory = template_content.len();

        // Additional memory for parsed AST (rough estimate)
        let ast_memory = template_content.len() / 2;

        // Memory for runtime context (rough estimate based on variables)
        let variable_count = Regex::new(r"\{\{[^}]+\}\}")
            .unwrap()
            .find_iter(template_content)
            .count();
        let context_memory = variable_count * 64; // Rough estimate per variable

        base_memory + ast_memory + context_memory
    }

    /// Generate optimization suggestions
    fn generate_optimization_suggestions(
        &self,
        template_content: &str,
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Check for repeated patterns
        let lines: Vec<&str> = template_content.lines().collect();
        let mut line_counts: HashMap<&str, usize> = HashMap::new();

        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                *line_counts.entry(trimmed).or_insert(0) += 1;
            }
        }

        for (line, count) in line_counts {
            if count > 3 {
                suggestions.push(OptimizationSuggestion::new(
                    "repeated_code".to_string(),
                    format!("Line '{}' is repeated {} times", line, count),
                    "Consider extracting repeated code into a macro or include".to_string(),
                ));
            }
        }

        // Check for large templates
        if template_content.len() > 10000 {
            suggestions.push(OptimizationSuggestion::new(
                "large_template".to_string(),
                "Template is very large".to_string(),
                "Consider breaking down large templates into smaller, reusable components"
                    .to_string(),
            ));
        }

        suggestions
    }

    /// Extract template variables and their usage
    fn extract_template_variables(&self, template_content: &str) -> Result<Vec<TemplateVariable>> {
        let mut variables = Vec::new();
        let mut seen_variables = HashSet::new();

        let variable_regex = Regex::new(
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*)\s*(?:\|[^}]*)?\s*\}\}",
        )?;

        for captures in variable_regex.captures_iter(template_content) {
            if let Some(var_match) = captures.get(1) {
                let var_name = var_match.as_str();

                if !seen_variables.contains(var_name) {
                    seen_variables.insert(var_name.to_string());

                    variables.push(TemplateVariable {
                        name: var_name.to_string(),
                        variable_type: self.infer_variable_type(var_name),
                        description: self.generate_variable_description(var_name),
                        required: true, // Assume all variables are required unless specified otherwise
                        default_value: None,
                    });
                }
            }
        }

        Ok(variables)
    }

    /// Generate usage examples for template
    fn generate_usage_examples(&self, template_content: &str) -> Result<Vec<UsageExample>> {
        let mut examples = Vec::new();

        // Generate basic example
        let variables = self.extract_template_variables(template_content)?;
        let mut example_context = HashMap::new();

        for var in &variables {
            let example_value = match var.variable_type.as_str() {
                "string" => Value::String("example_value".to_string()),
                "number" => Value::Number(serde_json::Number::from(42)),
                "boolean" => Value::Bool(true),
                "array" => Value::Array(vec![
                    Value::String("item1".to_string()),
                    Value::String("item2".to_string()),
                ]),
                _ => Value::String("example".to_string()),
            };
            example_context.insert(var.name.clone(), example_value);
        }

        examples.push(UsageExample {
            title: "Basic Usage".to_string(),
            description: "Basic template rendering example".to_string(),
            context: example_context,
            expected_output: None, // Would need actual rendering to generate this
        });

        Ok(examples)
    }

    /// Analyze template structure
    fn analyze_template_structure(&self, template_content: &str) -> Result<TemplateStructure> {
        let mut structure = TemplateStructure::new();

        // Count different template elements
        structure.total_lines = template_content.lines().count();
        structure.template_blocks = template_content.matches("{% ").count();
        structure.output_expressions = template_content.matches("{{ ").count();
        structure.comments = template_content.matches("{# ").count();

        // Identify template sections
        structure.sections = self.identify_template_sections(template_content)?;

        Ok(structure)
    }

    /// Generate markdown documentation
    fn generate_markdown_documentation(
        &self,
        documentation: &TemplateDocumentation,
    ) -> Result<String> {
        let mut markdown = String::new();

        markdown.push_str("# Template Documentation\n\n");

        // Metadata section
        if let Some(metadata) = &documentation.metadata {
            markdown.push_str("## Template Metadata\n\n");
            markdown.push_str(&format!("- **Match Pattern**: {:?}\n", metadata.pattern));
            // TemplateHeader doesn't have a description field, using scope instead
            if let Some(scope) = &metadata.scope {
                markdown.push_str(&format!("- **Scope**: {:?}\n", scope));
            }
            markdown.push_str("\n");
        }

        // Variables section
        if !documentation.variables.is_empty() {
            markdown.push_str("## Template Variables\n\n");
            markdown.push_str("| Name | Type | Required | Description |\n");
            markdown.push_str("|------|------|----------|-------------|\n");

            for var in &documentation.variables {
                markdown.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    var.name,
                    var.variable_type,
                    if var.required { "Yes" } else { "No" },
                    var.description.as_deref().unwrap_or("No description")
                ));
            }
            markdown.push_str("\n");
        }

        // Structure section
        markdown.push_str("## Template Structure\n\n");
        markdown.push_str(&format!(
            "- **Total Lines**: {}\n",
            documentation.template_structure.total_lines
        ));
        markdown.push_str(&format!(
            "- **Template Blocks**: {}\n",
            documentation.template_structure.template_blocks
        ));
        markdown.push_str(&format!(
            "- **Output Expressions**: {}\n",
            documentation.template_structure.output_expressions
        ));
        markdown.push_str(&format!(
            "- **Comments**: {}\n",
            documentation.template_structure.comments
        ));
        markdown.push_str("\n");

        // Usage examples
        if !documentation.usage_examples.is_empty() {
            markdown.push_str("## Usage Examples\n\n");
            for (_i, example) in documentation.usage_examples.iter().enumerate() {
                markdown.push_str(&format!("### {}\n\n", example.title));
                if !example.description.is_empty() {
                    markdown.push_str(&format!("{}\n\n", example.description));
                }
                markdown.push_str("```json\n");
                markdown.push_str(&serde_json::to_string_pretty(&example.context)?);
                markdown.push_str("\n```\n\n");
            }
        }

        Ok(markdown)
    }

    /// Find template files in directory
    fn find_template_files(&self, dir_path: &Path) -> Result<Vec<PathBuf>> {
        let mut template_files = Vec::new();

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "j2" || extension == "jinja" || extension == "template" {
                        template_files.push(path);
                    }
                }
            } else if path.is_dir() {
                // Recursively search subdirectories
                let mut sub_files = self.find_template_files(&path)?;
                template_files.append(&mut sub_files);
            }
        }

        Ok(template_files)
    }

    /// Infer variable type from name
    fn infer_variable_type(&self, var_name: &str) -> String {
        let name_lower = var_name.to_lowercase();

        if name_lower.contains("count") || name_lower.contains("num") || name_lower.contains("port")
        {
            "number".to_string()
        } else if name_lower.contains("enable")
            || name_lower.contains("disable")
            || name_lower.contains("is_")
        {
            "boolean".to_string()
        } else if name_lower.contains("list")
            || name_lower.contains("items")
            || name_lower.ends_with('s')
        {
            "array".to_string()
        } else {
            "string".to_string()
        }
    }

    /// Generate description for variable
    fn generate_variable_description(&self, var_name: &str) -> Option<String> {
        let name_lower = var_name.to_lowercase();

        if name_lower.contains("interface") {
            Some("Network interface name or identifier".to_string())
        } else if name_lower.contains("ip") || name_lower.contains("address") {
            Some("IP address or network address".to_string())
        } else if name_lower.contains("vlan") {
            Some("VLAN identifier or configuration".to_string())
        } else if name_lower.contains("description") {
            Some("Descriptive text or comment".to_string())
        } else {
            None
        }
    }

    /// Identify template sections
    fn identify_template_sections(&self, template_content: &str) -> Result<Vec<TemplateSection>> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = template_content.lines().collect();

        let mut current_section = None;
        let mut section_start = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Look for section headers (comments with section indicators)
            if trimmed.starts_with("{#")
                && (trimmed.contains("section") || trimmed.contains("SECTION"))
            {
                // End previous section
                if let Some(section_name) = current_section.take() {
                    sections.push(TemplateSection {
                        name: section_name,
                        start_line: section_start,
                        end_line: line_num,
                        line_count: line_num - section_start,
                    });
                }

                // Start new section
                current_section = Some(
                    trimmed
                        .replace("{#", "")
                        .replace("#}", "")
                        .trim()
                        .to_string(),
                );
                section_start = line_num;
            }
        }

        // End final section
        if let Some(section_name) = current_section {
            sections.push(TemplateSection {
                name: section_name,
                start_line: section_start,
                end_line: lines.len(),
                line_count: lines.len() - section_start,
            });
        }

        Ok(sections)
    }

    /// Extract template header line from template content
    fn extract_header_line(&self, template_content: &str) -> Option<String> {
        for line in template_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("{#") && trimmed.contains("template-match:") {
                // Extract the header line content
                let content = trimmed
                    .replace("{#", "")
                    .replace("#}", "")
                    .trim()
                    .to_string();
                return Some(content);
            }
        }
        None
    }

    /// Convert TemplateComplexity enum to numeric score
    fn complexity_to_score(complexity: &TemplateComplexity) -> u32 {
        match complexity {
            TemplateComplexity::Low => 25,
            TemplateComplexity::Medium => 60,
            TemplateComplexity::High => 100,
        }
    }
}

/// Complete quality analysis report for a single template
#[derive(Debug, Clone, Serialize)]
pub struct TemplateQualityReport {
    /// Path to the analyzed template
    pub template_path: PathBuf,
    /// Overall quality score (0-100)
    pub overall_score: u32,
    /// Time taken for analysis
    pub analysis_duration: Duration,
    /// Linting results
    pub linting_results: Option<TemplateLintingResults>,
    /// Performance analysis results
    pub performance_analysis: Option<TemplatePerformanceAnalysis>,
    /// Security scan results
    pub security_scan: Option<TemplateSecurityScan>,
    /// Generated documentation
    pub documentation: Option<TemplateDocumentation>,
}

impl TemplateQualityReport {
    pub fn new(template_path: PathBuf) -> Self {
        Self {
            template_path,
            overall_score: 0,
            analysis_duration: Duration::default(),
            linting_results: None,
            performance_analysis: None,
            security_scan: None,
            documentation: None,
        }
    }

    /// Calculate overall quality score based on all analysis results
    pub fn calculate_overall_score(&mut self) {
        let mut score = 100u32;
        let _factors = 0u32;

        // Linting score contribution
        if let Some(linting) = &self.linting_results {
            if !linting.syntax_valid {
                score = score.saturating_sub(30);
            }
            // Deduct points for errors and warnings
            score = score.saturating_sub((linting.errors.len() as u32) * 10);
            score = score.saturating_sub((linting.warnings.len() as u32) * 5);
        }

        // Performance score contribution
        if let Some(performance) = &self.performance_analysis {
            if !performance.compilation_successful {
                score = score.saturating_sub(25);
            }
            if let Some(estimates) = &performance.rendering_estimates {
                match estimates.performance_category.as_str() {
                    "slow" => score = score.saturating_sub(15),
                    "moderate" => score = score.saturating_sub(5),
                    _ => {} // No deduction for fast templates
                }
            }
        }

        // Security score contribution
        if let Some(security) = &self.security_scan {
            if !security.security_validation_passed {
                score = score.saturating_sub(40);
            }
            // Deduct points for security issues
            score = score.saturating_sub((security.total_security_issues as u32) * 8);
        }

        // Documentation completeness (bonus points for good documentation)
        if let Some(docs) = &self.documentation {
            if docs.metadata.is_some() {
                score = score.saturating_add(5);
            }
            if !docs.variables.is_empty() {
                score = score.saturating_add(5);
            }
        }

        self.overall_score = score.min(100);
    }
}

/// Quality analysis report for a directory of templates
#[derive(Debug, Clone, Serialize)]
pub struct DirectoryQualityReport {
    /// Directory path analyzed
    pub directory_path: PathBuf,
    /// Individual template reports
    pub template_reports: Vec<TemplateQualityReport>,
    /// Templates that failed analysis
    pub failed_analyses: Vec<(PathBuf, String)>,
    /// Total analysis duration
    pub analysis_duration: Duration,
    /// Summary statistics
    pub summary: DirectoryQualitySummary,
}

impl DirectoryQualityReport {
    pub fn new(directory_path: PathBuf) -> Self {
        Self {
            directory_path,
            template_reports: Vec::new(),
            failed_analyses: Vec::new(),
            analysis_duration: Duration::default(),
            summary: DirectoryQualitySummary::default(),
        }
    }

    /// Calculate summary statistics
    pub fn calculate_summary(&mut self) {
        self.summary.total_templates = self.template_reports.len();
        self.summary.failed_templates = self.failed_analyses.len();

        if !self.template_reports.is_empty() {
            let total_score: u32 = self.template_reports.iter().map(|r| r.overall_score).sum();
            self.summary.average_score = total_score / self.template_reports.len() as u32;

            self.summary.highest_score = self
                .template_reports
                .iter()
                .map(|r| r.overall_score)
                .max()
                .unwrap_or(0);
            self.summary.lowest_score = self
                .template_reports
                .iter()
                .map(|r| r.overall_score)
                .min()
                .unwrap_or(0);
        }

        // Count templates by score ranges
        for report in &self.template_reports {
            match report.overall_score {
                90..=100 => self.summary.excellent_templates += 1,
                75..=89 => self.summary.good_templates += 1,
                50..=74 => self.summary.fair_templates += 1,
                _ => self.summary.poor_templates += 1,
            }
        }
    }
}

/// Summary statistics for directory quality analysis
#[derive(Debug, Clone, Default, Serialize)]
pub struct DirectoryQualitySummary {
    pub total_templates: usize,
    pub failed_templates: usize,
    pub average_score: u32,
    pub highest_score: u32,
    pub lowest_score: u32,
    pub excellent_templates: usize, // 90-100
    pub good_templates: usize,      // 75-89
    pub fair_templates: usize,      // 50-74
    pub poor_templates: usize,      // 0-49
}

/// Template linting results
#[derive(Debug, Clone, Serialize)]
pub struct TemplateLintingResults {
    pub syntax_valid: bool,
    pub complexity_score: u32,
    pub total_issues: usize,
    pub errors: Vec<LintError>,
    pub warnings: Vec<LintWarning>,
}

impl TemplateLintingResults {
    pub fn new() -> Self {
        Self {
            syntax_valid: false,
            complexity_score: 0,
            total_issues: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Linting error
#[derive(Debug, Clone, Serialize)]
pub struct LintError {
    pub severity: LintErrorSeverity,
    pub error_type: String,
    pub message: String,
    pub line_number: Option<usize>,
}

impl LintError {
    pub fn new(
        severity: LintErrorSeverity,
        error_type: String,
        message: String,
        line_number: Option<usize>,
    ) -> Self {
        Self {
            severity,
            error_type,
            message,
            line_number,
        }
    }
}

/// Linting warning
#[derive(Debug, Clone, Serialize)]
pub struct LintWarning {
    pub warning_type: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl LintWarning {
    pub fn new(warning_type: String, message: String, suggestion: Option<String>) -> Self {
        Self {
            warning_type,
            message,
            suggestion,
        }
    }
}

/// Linting error severity levels
#[derive(Debug, Clone, Serialize)]
pub enum LintErrorSeverity {
    Error,
    Warning,
    Info,
}

/// Template performance analysis results
#[derive(Debug, Clone, Serialize)]
pub struct TemplatePerformanceAnalysis {
    pub template_size_bytes: usize,
    pub line_count: usize,
    pub compilation_successful: bool,
    pub compilation_time: Option<Duration>,
    pub compilation_error: Option<String>,
    pub rendering_estimates: Option<RenderingPerformanceEstimate>,
    pub estimated_memory_usage: usize,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

impl TemplatePerformanceAnalysis {
    pub fn new() -> Self {
        Self {
            template_size_bytes: 0,
            line_count: 0,
            compilation_successful: false,
            compilation_time: None,
            compilation_error: None,
            rendering_estimates: None,
            estimated_memory_usage: 0,
            optimization_suggestions: Vec::new(),
        }
    }
}

/// Rendering performance estimates
#[derive(Debug, Clone, Serialize)]
pub struct RenderingPerformanceEstimate {
    pub variable_count: usize,
    pub loop_count: usize,
    pub condition_count: usize,
    pub filter_count: usize,
    pub estimated_render_time: Duration,
    pub performance_category: String, // "fast", "moderate", "slow"
}

impl RenderingPerformanceEstimate {
    pub fn new() -> Self {
        Self {
            variable_count: 0,
            loop_count: 0,
            condition_count: 0,
            filter_count: 0,
            estimated_render_time: Duration::default(),
            performance_category: "unknown".to_string(),
        }
    }
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub recommendation: String,
}

impl OptimizationSuggestion {
    pub fn new(suggestion_type: String, description: String, recommendation: String) -> Self {
        Self {
            suggestion_type,
            description,
            recommendation,
        }
    }
}

/// Template security scan results
#[derive(Debug, Clone, Serialize)]
pub struct TemplateSecurityScan {
    pub security_validation_passed: bool,
    pub total_security_issues: usize,
    pub security_issues: Vec<SecurityIssue>,
    pub dangerous_patterns: Vec<DangerousPattern>,
    pub sensitive_data_risks: Vec<SensitiveDataRisk>,
    pub input_validation_issues: Vec<InputValidationIssue>,
}

impl TemplateSecurityScan {
    pub fn new() -> Self {
        Self {
            security_validation_passed: false,
            total_security_issues: 0,
            security_issues: Vec::new(),
            dangerous_patterns: Vec::new(),
            sensitive_data_risks: Vec::new(),
            input_validation_issues: Vec::new(),
        }
    }
}

/// Security issue
#[derive(Debug, Clone, Serialize)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub issue_type: String,
    pub message: String,
    pub recommendation: Option<String>,
}

impl SecurityIssue {
    pub fn new(
        severity: SecuritySeverity,
        issue_type: String,
        message: String,
        recommendation: Option<String>,
    ) -> Self {
        Self {
            severity,
            issue_type,
            message,
            recommendation,
        }
    }
}

/// Dangerous pattern detection
#[derive(Debug, Clone, Serialize)]
pub struct DangerousPattern {
    pub pattern_type: String,
    pub description: String,
    pub severity: SecuritySeverity,
}

impl DangerousPattern {
    pub fn new(pattern_type: String, description: String, severity: SecuritySeverity) -> Self {
        Self {
            pattern_type,
            description,
            severity,
        }
    }
}

/// Sensitive data exposure risk
#[derive(Debug, Clone, Serialize)]
pub struct SensitiveDataRisk {
    pub risk_type: String,
    pub description: String,
    pub severity: SecuritySeverity,
}

impl SensitiveDataRisk {
    pub fn new(risk_type: String, description: String, severity: SecuritySeverity) -> Self {
        Self {
            risk_type,
            description,
            severity,
        }
    }
}

/// Input validation issue
#[derive(Debug, Clone, Serialize)]
pub struct InputValidationIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub recommendation: Option<String>,
}

impl InputValidationIssue {
    pub fn new(
        issue_type: String,
        description: String,
        severity: SecuritySeverity,
        recommendation: Option<String>,
    ) -> Self {
        Self {
            issue_type,
            description,
            severity,
            recommendation,
        }
    }
}

/// Security severity levels
#[derive(Debug, Clone, Serialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Template documentation
#[derive(Debug, Clone, Serialize)]
pub struct TemplateDocumentation {
    pub metadata: Option<TemplateHeader>,
    pub variables: Vec<TemplateVariable>,
    pub usage_examples: Vec<UsageExample>,
    pub template_structure: TemplateStructure,
    pub markdown_content: String,
}

impl TemplateDocumentation {
    pub fn new() -> Self {
        Self {
            metadata: None,
            variables: Vec::new(),
            usage_examples: Vec::new(),
            template_structure: TemplateStructure::new(),
            markdown_content: String::new(),
        }
    }
}

/// Template variable documentation
#[derive(Debug, Clone, Serialize)]
pub struct TemplateVariable {
    pub name: String,
    pub variable_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<Value>,
}

/// Usage example for template
#[derive(Debug, Clone, Serialize)]
pub struct UsageExample {
    pub title: String,
    pub description: String,
    pub context: HashMap<String, Value>,
    pub expected_output: Option<String>,
}

/// Template structure analysis
#[derive(Debug, Clone, Serialize)]
pub struct TemplateStructure {
    pub total_lines: usize,
    pub template_blocks: usize,
    pub output_expressions: usize,
    pub comments: usize,
    pub sections: Vec<TemplateSection>,
}

impl TemplateStructure {
    pub fn new() -> Self {
        Self {
            total_lines: 0,
            template_blocks: 0,
            output_expressions: 0,
            comments: 0,
            sections: Vec::new(),
        }
    }
}

/// Template section
#[derive(Debug, Clone, Serialize)]
pub struct TemplateSection {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub line_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_quality_analyzer_creation() {
        let analyzer = TemplateQualityAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[tokio::test]
    async fn test_quality_analyzer_with_config() {
        let config = QualityAnalysisConfig {
            enable_linting: true,
            enable_performance_analysis: false,
            max_complexity_score: 50,
            verbose_output: true,
            ..Default::default()
        };

        let analyzer = TemplateQualityAnalyzer::with_config(config);
        assert!(analyzer.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_simple_template() -> Result<()> {
        let analyzer = TemplateQualityAnalyzer::new()?;

        // Create temporary template file
        let temp_dir = TempDir::new()?;
        let template_path = temp_dir.path().join("test.j2");

        let template_content = r#"
{# template-match: vendor=cisco interface=* #}
interface {{ interface_name }}
 description {{ description }}
 no shutdown
"#;

        fs::write(&template_path, template_content)?;

        let report = analyzer.analyze_template(&template_path).await?;

        assert_eq!(report.template_path, template_path);
        assert!(report.linting_results.is_some());
        assert!(report.performance_analysis.is_some());
        assert!(report.security_scan.is_some());
        assert!(report.documentation.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_directory() -> Result<()> {
        let analyzer = TemplateQualityAnalyzer::new()?;

        // Create temporary directory with templates
        let temp_dir = TempDir::new()?;

        let template1 = temp_dir.path().join("template1.j2");
        fs::write(&template1, "interface {{ name }}")?;

        let template2 = temp_dir.path().join("template2.j2");
        fs::write(&template2, "vlan {{ vlan_id }}")?;

        let report = analyzer.analyze_directory(temp_dir.path()).await?;

        assert_eq!(report.directory_path, temp_dir.path());
        assert_eq!(report.template_reports.len(), 2);
        assert_eq!(report.failed_analyses.len(), 0);

        Ok(())
    }

    #[test]
    fn test_infer_variable_type() {
        let analyzer = TemplateQualityAnalyzer::new().unwrap();

        assert_eq!(analyzer.infer_variable_type("port_number"), "number");
        assert_eq!(analyzer.infer_variable_type("is_enabled"), "boolean");
        assert_eq!(analyzer.infer_variable_type("interfaces"), "array");
        assert_eq!(analyzer.infer_variable_type("description"), "string");
    }

    #[test]
    fn test_generate_variable_description() {
        let analyzer = TemplateQualityAnalyzer::new().unwrap();

        assert!(
            analyzer
                .generate_variable_description("interface_name")
                .is_some()
        );
        assert!(
            analyzer
                .generate_variable_description("ip_address")
                .is_some()
        );
        assert!(analyzer.generate_variable_description("vlan_id").is_some());
        assert!(
            analyzer
                .generate_variable_description("random_var")
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_template_with_security_issues() -> Result<()> {
        let analyzer = TemplateQualityAnalyzer::new()?;

        let template_with_issues = r#"
password {{ password }}
exec {{ system_command }}
"#;

        let security_scan = analyzer.scan_security(template_with_issues).await?;

        assert!(!security_scan.sensitive_data_risks.is_empty());
        assert!(security_scan.total_security_issues > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_analysis() -> Result<()> {
        let analyzer = TemplateQualityAnalyzer::new()?;

        let complex_template = r#"
{% for item in items %}
  {% for subitem in item.subitems %}
    {{ subitem | upper | trim }}
  {% endfor %}
{% endfor %}
"#;

        let analysis = analyzer.analyze_performance(complex_template).await?;

        assert!(analysis.compilation_successful);
        assert!(analysis.rendering_estimates.is_some());

        if let Some(estimates) = analysis.rendering_estimates {
            assert!(estimates.loop_count > 0);
            assert!(estimates.filter_count > 0);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_documentation_generation() -> Result<()> {
        let analyzer = TemplateQualityAnalyzer::new()?;

        let documented_template = r#"
{# template-match: vendor=cisco interface=* #}
{# This template configures Cisco interfaces #}
interface {{ interface_name }}
 description {{ description }}
 ip address {{ ip_address }} {{ subnet_mask }}
"#;

        let docs = analyzer.generate_documentation(documented_template).await?;

        assert!(docs.metadata.is_some());
        assert!(!docs.variables.is_empty());
        assert!(!docs.markdown_content.is_empty());

        // Check that variables were extracted
        let var_names: Vec<&str> = docs.variables.iter().map(|v| v.name.as_str()).collect();
        assert!(var_names.contains(&"interface_name"));
        assert!(var_names.contains(&"description"));
        assert!(var_names.contains(&"ip_address"));
        assert!(var_names.contains(&"subnet_mask"));

        Ok(())
    }
}
