//! Template testing framework for comprehensive validation and regression testing

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use super::{
    RenderingResult, TemplateContext, TemplateEngine, TemplateRenderer, TemplateValidator,
};
use crate::models::Node;

/// Comprehensive template testing framework
#[derive(Debug)]
pub struct TemplateTestFramework {
    /// Template engine for rendering
    engine: TemplateEngine,
    /// Template renderer for full pipeline testing
    renderer: TemplateRenderer,
    /// Template validator for syntax and security
    validator: TemplateValidator,
    /// Configuration for test execution
    config: TestFrameworkConfig,
    /// Registry of test templates
    test_registry: TestRegistry,
}

/// Configuration for the template testing framework
#[derive(Debug, Clone)]
pub struct TestFrameworkConfig {
    /// Maximum time allowed for individual template tests
    pub test_timeout: Duration,
    /// Maximum time allowed for test suite execution
    pub suite_timeout: Duration,
    /// Whether to run integration tests
    pub run_integration_tests: bool,
    /// Whether to run regression tests
    pub run_regression_tests: bool,
    /// Directory containing test templates
    pub test_templates_dir: Option<PathBuf>,
    /// Directory for storing test artifacts
    pub test_artifacts_dir: Option<PathBuf>,
    /// Whether to enable verbose test output
    pub verbose: bool,
}

impl Default for TestFrameworkConfig {
    fn default() -> Self {
        Self {
            test_timeout: Duration::from_secs(30),
            suite_timeout: Duration::from_secs(300), // 5 minutes
            run_integration_tests: true,
            run_regression_tests: true,
            test_templates_dir: None,
            test_artifacts_dir: None,
            verbose: false,
        }
    }
}

/// Registry for managing test templates and cases
#[derive(Debug, Clone)]
pub struct TestRegistry {
    /// Unit test cases
    unit_tests: Vec<TemplateUnitTest>,
    /// Integration test cases
    integration_tests: Vec<TemplateIntegrationTest>,
    /// Regression test cases  
    regression_tests: Vec<TemplateRegressionTest>,
    /// Performance benchmark cases
    performance_tests: Vec<TemplatePerformanceTest>,
}

impl Default for TestRegistry {
    fn default() -> Self {
        Self {
            unit_tests: Vec::new(),
            integration_tests: Vec::new(),
            regression_tests: Vec::new(),
            performance_tests: Vec::new(),
        }
    }
}

impl TemplateTestFramework {
    /// Create a new template testing framework
    pub fn new() -> Result<Self> {
        Self::with_config(TestFrameworkConfig::default())
    }

    /// Create a new template testing framework with custom configuration
    pub fn with_config(config: TestFrameworkConfig) -> Result<Self> {
        let engine = TemplateEngine::with_timeout(config.test_timeout)?;
        let renderer = TemplateRenderer::new()?;
        let validator = TemplateValidator::new();
        let test_registry = TestRegistry::default();

        Ok(Self {
            engine,
            renderer,
            validator,
            config,
            test_registry,
        })
    }

    /// Register a unit test case
    pub fn register_unit_test(&mut self, test: TemplateUnitTest) {
        self.test_registry.unit_tests.push(test);
    }

    /// Register an integration test case
    pub fn register_integration_test(&mut self, test: TemplateIntegrationTest) {
        self.test_registry.integration_tests.push(test);
    }

    /// Register a regression test case
    pub fn register_regression_test(&mut self, test: TemplateRegressionTest) {
        self.test_registry.regression_tests.push(test);
    }

    /// Register a performance test case
    pub fn register_performance_test(&mut self, test: TemplatePerformanceTest) {
        self.test_registry.performance_tests.push(test);
    }

    /// Load test cases from a directory
    pub async fn load_tests_from_directory<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Err(anyhow!("Test directory does not exist: {}", dir.display()));
        }

        info!("Loading test cases from directory: {}", dir.display());

        // Load unit tests
        let unit_tests_dir = dir.join("unit");
        if unit_tests_dir.exists() {
            self.load_unit_tests(&unit_tests_dir).await?;
        }

        // Load integration tests
        let integration_tests_dir = dir.join("integration");
        if integration_tests_dir.exists() {
            self.load_integration_tests(&integration_tests_dir).await?;
        }

        // Load regression tests
        let regression_tests_dir = dir.join("regression");
        if regression_tests_dir.exists() {
            self.load_regression_tests(&regression_tests_dir).await?;
        }

        // Load performance tests
        let performance_tests_dir = dir.join("performance");
        if performance_tests_dir.exists() {
            self.load_performance_tests(&performance_tests_dir).await?;
        }

        info!(
            "Loaded {} unit tests, {} integration tests, {} regression tests, {} performance tests",
            self.test_registry.unit_tests.len(),
            self.test_registry.integration_tests.len(),
            self.test_registry.regression_tests.len(),
            self.test_registry.performance_tests.len()
        );

        Ok(())
    }

    /// Run all registered tests
    pub async fn run_all_tests(&self) -> Result<TestSuiteResult> {
        let start_time = Instant::now();
        info!("Starting template test suite execution");

        let mut suite_result = TestSuiteResult::new();

        // Run unit tests
        if !self.test_registry.unit_tests.is_empty() {
            info!("Running {} unit tests", self.test_registry.unit_tests.len());
            let unit_results = self.run_unit_tests().await?;
            suite_result.unit_test_results = unit_results;
        }

        // Run integration tests if enabled
        if self.config.run_integration_tests && !self.test_registry.integration_tests.is_empty() {
            info!(
                "Running {} integration tests",
                self.test_registry.integration_tests.len()
            );
            let integration_results = self.run_integration_tests().await?;
            suite_result.integration_test_results = integration_results;
        }

        // Run regression tests if enabled
        if self.config.run_regression_tests && !self.test_registry.regression_tests.is_empty() {
            info!(
                "Running {} regression tests",
                self.test_registry.regression_tests.len()
            );
            let regression_results = self.run_regression_tests().await?;
            suite_result.regression_test_results = regression_results;
        }

        // Run performance tests
        if !self.test_registry.performance_tests.is_empty() {
            info!(
                "Running {} performance tests",
                self.test_registry.performance_tests.len()
            );
            let performance_results = self.run_performance_tests().await?;
            suite_result.performance_test_results = performance_results;
        }

        suite_result.total_duration = start_time.elapsed();
        suite_result.calculate_summary();

        info!(
            "Template test suite completed in {:?} - {} passed, {} failed, {} skipped",
            suite_result.total_duration,
            suite_result.summary.total_passed,
            suite_result.summary.total_failed,
            suite_result.summary.total_skipped
        );

        Ok(suite_result)
    }

    /// Run unit tests
    async fn run_unit_tests(&self) -> Result<Vec<UnitTestResult>> {
        let mut results = Vec::new();

        for test in &self.test_registry.unit_tests {
            let start_time = Instant::now();
            let result = self.run_single_unit_test(test).await;
            let duration = start_time.elapsed();

            let test_result = UnitTestResult {
                test_name: test.name.clone(),
                passed: result.is_ok(),
                duration,
                error: result.err().map(|e| e.to_string()),
            };

            if self.config.verbose {
                info!(
                    "Unit test '{}' {} in {:?}",
                    test.name,
                    if test_result.passed {
                        "PASSED"
                    } else {
                        "FAILED"
                    },
                    duration
                );
            }

            results.push(test_result);
        }

        Ok(results)
    }

    /// Run a single unit test
    async fn run_single_unit_test(&self, test: &TemplateUnitTest) -> Result<()> {
        debug!("Running unit test: {}", test.name);

        // Validate template syntax
        self.validator.validate_syntax(&test.template_content)?;

        // Test template rendering with test context
        let context_value =
            serde_json::to_value(&test.test_context).context("Failed to serialize test context")?;

        let output = self.engine.render(&test.name, &context_value).await?;

        // Verify expected output if provided
        if let Some(expected) = &test.expected_output {
            if &output != expected {
                return Err(anyhow!(
                    "Template output mismatch:\nExpected: {}\nActual: {}",
                    expected,
                    output
                ));
            }
        }

        // Verify expected patterns if provided
        if let Some(patterns) = &test.expected_patterns {
            for pattern in patterns {
                let regex = regex::Regex::new(pattern)
                    .with_context(|| format!("Invalid regex pattern: {}", pattern))?;

                if !regex.is_match(&output) {
                    return Err(anyhow!(
                        "Template output does not match pattern '{}': {}",
                        pattern,
                        output
                    ));
                }
            }
        }

        // Check that forbidden patterns are not present
        if let Some(forbidden) = &test.forbidden_patterns {
            for pattern in forbidden {
                let regex = regex::Regex::new(pattern)
                    .with_context(|| format!("Invalid regex pattern: {}", pattern))?;

                if regex.is_match(&output) {
                    return Err(anyhow!(
                        "Template output contains forbidden pattern '{}': {}",
                        pattern,
                        output
                    ));
                }
            }
        }

        debug!("Unit test '{}' completed successfully", test.name);
        Ok(())
    }

    /// Run integration tests
    async fn run_integration_tests(&self) -> Result<Vec<IntegrationTestResult>> {
        let mut results = Vec::new();

        for test in &self.test_registry.integration_tests {
            let start_time = Instant::now();
            let result = self.run_single_integration_test(test).await;
            let duration = start_time.elapsed();

            let test_result = IntegrationTestResult {
                test_name: test.name.clone(),
                passed: result.is_ok(),
                duration,
                error: result.err().map(|e| e.to_string()),
                rendering_result: None, // Could store successful rendering results
            };

            if self.config.verbose {
                info!(
                    "Integration test '{}' {} in {:?}",
                    test.name,
                    if test_result.passed {
                        "PASSED"
                    } else {
                        "FAILED"
                    },
                    duration
                );
            }

            results.push(test_result);
        }

        Ok(results)
    }

    /// Run a single integration test
    async fn run_single_integration_test(&self, test: &TemplateIntegrationTest) -> Result<()> {
        debug!("Running integration test: {}", test.name);

        // Create template context using the provided node
        let context = TemplateContext::new(test.test_node.clone())
            .with_variables(test.context_variables.clone());

        // Render template using full pipeline
        let rendering_result = self
            .renderer
            .render_template(&test.template_name, context)
            .await?;

        // Validate the output
        if !rendering_result.is_valid() {
            return Err(anyhow!(
                "Template output validation failed: {:?}",
                rendering_result.validation_result
            ));
        }

        // Check vendor-specific validation if specified
        if let Some(expected_vendor) = &test.expected_vendor {
            let output_vendor = &rendering_result
                .context
                .node
                .vendor
                .to_string()
                .to_lowercase();
            if output_vendor != expected_vendor {
                return Err(anyhow!(
                    "Expected vendor '{}' but got '{}'",
                    expected_vendor,
                    output_vendor
                ));
            }
        }

        // Verify output meets minimum requirements
        if rendering_result.output.trim().is_empty() {
            return Err(anyhow!("Template rendered empty output"));
        }

        // Check performance requirements
        if let Some(max_duration) = test.max_render_time {
            if rendering_result.duration > max_duration {
                return Err(anyhow!(
                    "Template rendering took {:?}, exceeding maximum of {:?}",
                    rendering_result.duration,
                    max_duration
                ));
            }
        }

        debug!("Integration test '{}' completed successfully", test.name);
        Ok(())
    }

    /// Run regression tests
    async fn run_regression_tests(&self) -> Result<Vec<RegressionTestResult>> {
        let mut results = Vec::new();

        for test in &self.test_registry.regression_tests {
            let start_time = Instant::now();
            let result = self.run_single_regression_test(test).await;
            let duration = start_time.elapsed();

            let passed = result.is_ok();
            let error = result.err().map(|e| e.to_string());

            let test_result = RegressionTestResult {
                test_name: test.name.clone(),
                passed,
                duration,
                error,
                baseline_match: passed,
            };

            if self.config.verbose {
                info!(
                    "Regression test '{}' {} in {:?}",
                    test.name,
                    if test_result.passed {
                        "PASSED"
                    } else {
                        "FAILED"
                    },
                    duration
                );
            }

            results.push(test_result);
        }

        Ok(results)
    }

    /// Run a single regression test
    async fn run_single_regression_test(&self, test: &TemplateRegressionTest) -> Result<()> {
        debug!("Running regression test: {}", test.name);

        // Create context for regression test
        let context = TemplateContext::new(test.baseline_node.clone())
            .with_variables(test.baseline_context.clone());

        // Render template
        let rendering_result = self
            .renderer
            .render_template(&test.template_name, context)
            .await?;

        // Compare with baseline output
        let actual_output = rendering_result.output.trim();
        let expected_output = test.baseline_output.trim();

        if actual_output != expected_output {
            // Allow for some flexibility with whitespace differences
            let normalized_actual = normalize_config_output(actual_output);
            let normalized_expected = normalize_config_output(expected_output);

            if normalized_actual != normalized_expected {
                return Err(anyhow!(
                    "Regression test failed - output differs from baseline:\nExpected:\n{}\nActual:\n{}",
                    expected_output,
                    actual_output
                ));
            }
        }

        // Verify rendering time hasn't regressed significantly
        if let Some(baseline_duration) = test.baseline_duration {
            let regression_threshold = baseline_duration.mul_f64(1.5); // 50% slower is considered regression
            if rendering_result.duration > regression_threshold {
                warn!(
                    "Performance regression detected in '{}': {:?} vs baseline {:?}",
                    test.name, rendering_result.duration, baseline_duration
                );
                // Note: Performance regressions are warnings, not failures for now
            }
        }

        debug!("Regression test '{}' completed successfully", test.name);
        Ok(())
    }

    /// Run performance tests
    async fn run_performance_tests(&self) -> Result<Vec<PerformanceTestResult>> {
        let mut results = Vec::new();

        for test in &self.test_registry.performance_tests {
            let result = self.run_single_performance_test(test).await?;

            if self.config.verbose {
                info!(
                    "Performance test '{}' - {} iterations, avg: {:?}, max: {:?}",
                    test.name, result.iterations, result.average_duration, result.max_duration
                );
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Run a single performance test
    async fn run_single_performance_test(
        &self,
        test: &TemplatePerformanceTest,
    ) -> Result<PerformanceTestResult> {
        debug!("Running performance test: {}", test.name);

        let mut durations = Vec::new();
        let context = TemplateContext::new(test.test_node.clone())
            .with_variables(test.context_variables.clone());

        // Run multiple iterations to get reliable performance data
        for _ in 0..test.iterations {
            let start_time = Instant::now();
            let _result = self
                .renderer
                .render_template(&test.template_name, context.clone())
                .await?;
            durations.push(start_time.elapsed());
        }

        // Calculate statistics
        let total_duration: Duration = durations.iter().sum();
        let average_duration = total_duration / durations.len() as u32;
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();

        // Check performance requirements
        let mut passed = true;
        let mut error = None;

        if let Some(max_avg) = test.max_average_duration {
            if average_duration > max_avg {
                passed = false;
                error = Some(format!(
                    "Average duration {:?} exceeds maximum {:?}",
                    average_duration, max_avg
                ));
            }
        }

        if let Some(max_single) = test.max_single_duration {
            if max_duration > max_single {
                passed = false;
                error = Some(format!(
                    "Maximum duration {:?} exceeds limit {:?}",
                    max_duration, max_single
                ));
            }
        }

        Ok(PerformanceTestResult {
            test_name: test.name.clone(),
            passed,
            iterations: test.iterations,
            total_duration,
            average_duration,
            min_duration,
            max_duration,
            error,
        })
    }

    /// Load unit tests from directory
    async fn load_unit_tests(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                let content = fs::read_to_string(&path)?;
                let test: TemplateUnitTest = serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse unit test: {}", path.display()))?;
                self.register_unit_test(test);
            }
        }
        Ok(())
    }

    /// Load integration tests from directory
    async fn load_integration_tests(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                let content = fs::read_to_string(&path)?;
                let test: TemplateIntegrationTest =
                    serde_yaml::from_str(&content).with_context(|| {
                        format!("Failed to parse integration test: {}", path.display())
                    })?;
                self.register_integration_test(test);
            }
        }
        Ok(())
    }

    /// Load regression tests from directory
    async fn load_regression_tests(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                let content = fs::read_to_string(&path)?;
                let test: TemplateRegressionTest =
                    serde_yaml::from_str(&content).with_context(|| {
                        format!("Failed to parse regression test: {}", path.display())
                    })?;
                self.register_regression_test(test);
            }
        }
        Ok(())
    }

    /// Load performance tests from directory
    async fn load_performance_tests(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                let content = fs::read_to_string(&path)?;
                let test: TemplatePerformanceTest =
                    serde_yaml::from_str(&content).with_context(|| {
                        format!("Failed to parse performance test: {}", path.display())
                    })?;
                self.register_performance_test(test);
            }
        }
        Ok(())
    }

    /// Generate a test report
    pub fn generate_report(&self, result: &TestSuiteResult) -> Result<String> {
        let mut report = String::new();

        report.push_str("# Template Test Suite Report\n\n");
        report.push_str(&format!(
            "**Execution Time:** {:?}\n",
            result.total_duration
        ));
        report.push_str(&format!(
            "**Total Tests:** {}\n",
            result.summary.total_tests
        ));
        report.push_str(&format!("**Passed:** {}\n", result.summary.total_passed));
        report.push_str(&format!("**Failed:** {}\n", result.summary.total_failed));
        report.push_str(&format!(
            "**Skipped:** {}\n\n",
            result.summary.total_skipped
        ));

        // Unit tests section
        if !result.unit_test_results.is_empty() {
            report.push_str("## Unit Tests\n\n");
            for test in &result.unit_test_results {
                report.push_str(&format!(
                    "- **{}**: {} ({:?})\n",
                    test.test_name,
                    if test.passed {
                        "✅ PASSED"
                    } else {
                        "❌ FAILED"
                    },
                    test.duration
                ));
                if let Some(error) = &test.error {
                    report.push_str(&format!("  Error: {}\n", error));
                }
            }
            report.push('\n');
        }

        // Integration tests section
        if !result.integration_test_results.is_empty() {
            report.push_str("## Integration Tests\n\n");
            for test in &result.integration_test_results {
                report.push_str(&format!(
                    "- **{}**: {} ({:?})\n",
                    test.test_name,
                    if test.passed {
                        "✅ PASSED"
                    } else {
                        "❌ FAILED"
                    },
                    test.duration
                ));
                if let Some(error) = &test.error {
                    report.push_str(&format!("  Error: {}\n", error));
                }
            }
            report.push('\n');
        }

        // Regression tests section
        if !result.regression_test_results.is_empty() {
            report.push_str("## Regression Tests\n\n");
            for test in &result.regression_test_results {
                report.push_str(&format!(
                    "- **{}**: {} ({:?})\n",
                    test.test_name,
                    if test.passed {
                        "✅ PASSED"
                    } else {
                        "❌ FAILED"
                    },
                    test.duration
                ));
                if let Some(error) = &test.error {
                    report.push_str(&format!("  Error: {}\n", error));
                }
            }
            report.push('\n');
        }

        // Performance tests section
        if !result.performance_test_results.is_empty() {
            report.push_str("## Performance Tests\n\n");
            for test in &result.performance_test_results {
                report.push_str(&format!(
                    "- **{}**: {} - {} iterations, avg: {:?}\n",
                    test.test_name,
                    if test.passed {
                        "✅ PASSED"
                    } else {
                        "❌ FAILED"
                    },
                    test.iterations,
                    test.average_duration
                ));
                if let Some(error) = &test.error {
                    report.push_str(&format!("  Error: {}\n", error));
                }
            }
        }

        Ok(report)
    }
}

/// Template unit test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateUnitTest {
    /// Test name
    pub name: String,
    /// Template content to test
    pub template_content: String,
    /// Test context for rendering
    pub test_context: HashMap<String, Value>,
    /// Expected exact output (optional)
    pub expected_output: Option<String>,
    /// Expected output patterns (regex)
    pub expected_patterns: Option<Vec<String>>,
    /// Forbidden patterns that should not appear
    pub forbidden_patterns: Option<Vec<String>>,
    /// Description of what this test validates
    pub description: Option<String>,
}

/// Template integration test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateIntegrationTest {
    /// Test name
    pub name: String,
    /// Template name to render
    pub template_name: String,
    /// Test node for context
    pub test_node: Node,
    /// Additional context variables
    pub context_variables: HashMap<String, Value>,
    /// Expected vendor for validation
    pub expected_vendor: Option<String>,
    /// Maximum allowed render time
    pub max_render_time: Option<Duration>,
    /// Description of what this test validates
    pub description: Option<String>,
}

/// Template regression test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRegressionTest {
    /// Test name
    pub name: String,
    /// Template name to test
    pub template_name: String,
    /// Baseline node configuration
    pub baseline_node: Node,
    /// Baseline context variables
    pub baseline_context: HashMap<String, Value>,
    /// Expected baseline output
    pub baseline_output: String,
    /// Baseline rendering duration for performance tracking
    pub baseline_duration: Option<Duration>,
    /// Description of what this regression test protects
    pub description: Option<String>,
}

/// Template performance test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePerformanceTest {
    /// Test name
    pub name: String,
    /// Template name to test
    pub template_name: String,
    /// Test node for context
    pub test_node: Node,
    /// Context variables
    pub context_variables: HashMap<String, Value>,
    /// Number of iterations to run
    pub iterations: usize,
    /// Maximum allowed average duration
    pub max_average_duration: Option<Duration>,
    /// Maximum allowed single iteration duration
    pub max_single_duration: Option<Duration>,
    /// Description of what this performance test measures
    pub description: Option<String>,
}

/// Result of a unit test
#[derive(Debug, Clone)]
pub struct UnitTestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

/// Result of an integration test
#[derive(Debug, Clone)]
pub struct IntegrationTestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub rendering_result: Option<RenderingResult>,
}

/// Result of a regression test
#[derive(Debug, Clone)]
pub struct RegressionTestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub baseline_match: bool,
}

/// Result of a performance test
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub passed: bool,
    pub iterations: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub error: Option<String>,
}

/// Complete test suite results
#[derive(Debug, Clone)]
pub struct TestSuiteResult {
    pub unit_test_results: Vec<UnitTestResult>,
    pub integration_test_results: Vec<IntegrationTestResult>,
    pub regression_test_results: Vec<RegressionTestResult>,
    pub performance_test_results: Vec<PerformanceTestResult>,
    pub total_duration: Duration,
    pub summary: TestSummary,
}

impl TestSuiteResult {
    pub fn new() -> Self {
        Self {
            unit_test_results: Vec::new(),
            integration_test_results: Vec::new(),
            regression_test_results: Vec::new(),
            performance_test_results: Vec::new(),
            total_duration: Duration::default(),
            summary: TestSummary::default(),
        }
    }

    pub fn calculate_summary(&mut self) {
        let mut total_tests = 0;
        let mut total_passed = 0;
        let mut total_failed = 0;

        // Count unit tests
        total_tests += self.unit_test_results.len();
        total_passed += self.unit_test_results.iter().filter(|t| t.passed).count();
        total_failed += self.unit_test_results.iter().filter(|t| !t.passed).count();

        // Count integration tests
        total_tests += self.integration_test_results.len();
        total_passed += self
            .integration_test_results
            .iter()
            .filter(|t| t.passed)
            .count();
        total_failed += self
            .integration_test_results
            .iter()
            .filter(|t| !t.passed)
            .count();

        // Count regression tests
        total_tests += self.regression_test_results.len();
        total_passed += self
            .regression_test_results
            .iter()
            .filter(|t| t.passed)
            .count();
        total_failed += self
            .regression_test_results
            .iter()
            .filter(|t| !t.passed)
            .count();

        // Count performance tests
        total_tests += self.performance_test_results.len();
        total_passed += self
            .performance_test_results
            .iter()
            .filter(|t| t.passed)
            .count();
        total_failed += self
            .performance_test_results
            .iter()
            .filter(|t| !t.passed)
            .count();

        self.summary = TestSummary {
            total_tests,
            total_passed,
            total_failed,
            total_skipped: 0, // No skipped tests in this implementation
        };
    }
}

/// Summary of test execution
#[derive(Debug, Clone, Default)]
pub struct TestSummary {
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_skipped: usize,
}

/// Normalize configuration output for comparison (removes minor whitespace differences)
fn normalize_config_output(output: &str) -> String {
    output
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DeviceRole, NodeBuilder, Vendor};
    use std::time::Duration;

    #[tokio::test]
    async fn test_framework_creation() {
        let framework = TemplateTestFramework::new();
        assert!(framework.is_ok());
    }

    #[tokio::test]
    async fn test_framework_with_config() {
        let config = TestFrameworkConfig {
            test_timeout: Duration::from_secs(10),
            run_integration_tests: false,
            verbose: true,
            ..Default::default()
        };

        let framework = TemplateTestFramework::with_config(config);
        assert!(framework.is_ok());
    }

    #[tokio::test]
    async fn test_unit_test_registration() {
        let mut framework = TemplateTestFramework::new().unwrap();

        let test = TemplateUnitTest {
            name: "test_basic_template".to_string(),
            template_content: "Hello {{ name }}!".to_string(),
            test_context: {
                let mut ctx = HashMap::new();
                ctx.insert("name".to_string(), Value::String("World".to_string()));
                ctx
            },
            expected_output: Some("Hello World!".to_string()),
            expected_patterns: None,
            forbidden_patterns: None,
            description: Some("Basic template rendering test".to_string()),
        };

        framework.register_unit_test(test);
        assert_eq!(framework.test_registry.unit_tests.len(), 1);
    }

    #[tokio::test]
    async fn test_integration_test_registration() {
        let mut framework = TemplateTestFramework::new().unwrap();

        let test_node = NodeBuilder::new()
            .name("test-router")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4331")
            .role(DeviceRole::Router)
            .build()
            .unwrap();

        let test = TemplateIntegrationTest {
            name: "test_cisco_interface".to_string(),
            template_name: "cisco_interface.j2".to_string(),
            test_node,
            context_variables: HashMap::new(),
            expected_vendor: Some("cisco".to_string()),
            max_render_time: Some(Duration::from_secs(5)),
            description: Some("Cisco interface template integration test".to_string()),
        };

        framework.register_integration_test(test);
        assert_eq!(framework.test_registry.integration_tests.len(), 1);
    }

    #[test]
    fn test_normalize_config_output() {
        let input = "  interface GigabitEthernet0/0  \n  \n   description test    \n\n";
        let expected = "interface GigabitEthernet0/0\ndescription test";
        let result = normalize_config_output(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_test_suite_result_summary() {
        let mut suite_result = TestSuiteResult::new();

        // Add some mock results
        suite_result.unit_test_results.push(UnitTestResult {
            test_name: "test1".to_string(),
            passed: true,
            duration: Duration::from_millis(100),
            error: None,
        });

        suite_result.unit_test_results.push(UnitTestResult {
            test_name: "test2".to_string(),
            passed: false,
            duration: Duration::from_millis(200),
            error: Some("Test failed".to_string()),
        });

        suite_result.calculate_summary();

        assert_eq!(suite_result.summary.total_tests, 2);
        assert_eq!(suite_result.summary.total_passed, 1);
        assert_eq!(suite_result.summary.total_failed, 1);
        assert_eq!(suite_result.summary.total_skipped, 0);
    }

    #[tokio::test]
    async fn test_run_empty_test_suite() {
        let framework = TemplateTestFramework::new().unwrap();
        let result = framework.run_all_tests().await;
        assert!(result.is_ok());

        let suite_result = result.unwrap();
        assert_eq!(suite_result.summary.total_tests, 0);
        assert_eq!(suite_result.summary.total_passed, 0);
        assert_eq!(suite_result.summary.total_failed, 0);
    }
}
