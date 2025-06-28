//! Diff analysis and reporting module
//!
//! This module provides comprehensive analysis capabilities for configuration diffs,
//! including detailed statistics, impact analysis, change categorization, and
//! summarization utilities.

use crate::diff::types::{
    ChangeComplexity, ChangeSeverity, DiffResult, DiffType, FunctionalChangeType, HierarchicalDiff,
    ImpactAnalysis, RiskLevel,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Comprehensive analysis of a diff result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffAnalysis {
    /// Detailed statistics about the diff
    pub statistics: DiffStatistics,
    /// Impact analysis of changes
    pub impact_analysis: Vec<ImpactAnalysis>,
    /// Change categorization
    pub categorization: ChangeCategorization,
    /// Summary and recommendations
    pub summary: AnalysisSummary,
}

/// Detailed statistics about a configuration diff
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffStatistics {
    /// Basic change counts
    pub change_counts: ChangeCounts,
    /// Line-level statistics
    pub line_statistics: LineStatistics,
    /// Section-level statistics
    pub section_statistics: SectionStatistics,
    /// Severity distribution
    pub severity_distribution: HashMap<ChangeSeverity, usize>,
    /// Risk distribution
    pub risk_distribution: HashMap<RiskLevel, usize>,
    /// Change type distribution
    pub change_type_distribution: HashMap<DiffType, usize>,
    /// Functional change distribution
    pub functional_change_distribution: HashMap<FunctionalChangeType, usize>,
}

/// Basic change counts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeCounts {
    /// Total number of changes
    pub total: usize,
    /// Number of additions
    pub additions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Number of modifications
    pub modifications: usize,
    /// Number of unchanged lines (for context)
    pub unchanged: usize,
}

/// Line-level statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineStatistics {
    /// Total lines in old configuration
    pub old_total_lines: usize,
    /// Total lines in new configuration
    pub new_total_lines: usize,
    /// Lines affected by changes
    pub affected_lines: usize,
    /// Percentage of configuration changed
    pub change_percentage: f64,
    /// Average lines per change
    pub average_lines_per_change: f64,
}

/// Section-level statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionStatistics {
    /// Total sections in old configuration
    pub old_total_sections: usize,
    /// Total sections in new configuration
    pub new_total_sections: usize,
    /// Sections added
    pub sections_added: usize,
    /// Sections removed
    pub sections_removed: usize,
    /// Sections modified
    pub sections_modified: usize,
    /// Sections unchanged
    pub sections_unchanged: usize,
    /// Section change percentage
    pub section_change_percentage: f64,
}

/// Change categorization by different dimensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ChangeCategorization {
    /// Changes by network function
    pub by_function: HashMap<NetworkFunction, Vec<String>>,
    /// Changes by configuration section
    pub by_section: HashMap<ConfigSection, Vec<String>>,
    /// Changes by vendor-specific features
    pub by_vendor_feature: HashMap<VendorFeature, Vec<String>>,
    /// Changes by operational impact
    pub by_impact: HashMap<OperationalImpact, Vec<String>>,
}

/// Network function categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkFunction {
    /// Layer 2 switching functionality
    Switching,
    /// Layer 3 routing functionality
    Routing,
    /// Security and access control
    Security,
    /// Quality of service
    QualityOfService,
    /// Network management
    Management,
    /// Monitoring and diagnostics
    Monitoring,
    /// Other network function
    Other(String),
}

/// Configuration section categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigSection {
    /// Interface configurations
    Interfaces,
    /// VLAN configurations
    Vlans,
    /// Routing protocol configurations
    Routing,
    /// Access control lists
    AccessLists,
    /// System-level configurations
    System,
    /// Other configuration section
    Other(String),
}

/// Vendor-specific feature categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VendorFeature {
    /// Cisco-specific features
    Cisco(String),
    /// Juniper-specific features
    Juniper(String),
    /// Arista-specific features
    Arista(String),
    /// Generic/standard features
    Standard(String),
}

/// Operational impact categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationalImpact {
    /// Changes affecting network connectivity
    Connectivity,
    /// Changes affecting network performance
    Performance,
    /// Changes affecting security posture
    Security,
    /// Changes affecting monitoring/visibility
    Monitoring,
    /// Configuration changes with minimal impact
    Cosmetic,
}

/// Analysis summary with recommendations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Overall assessment of the changes
    pub overall_assessment: OverallAssessment,
    /// Key findings from the analysis
    pub key_findings: Vec<String>,
    /// Recommendations for deployment
    pub recommendations: Vec<Recommendation>,
    /// Required validation steps
    pub validation_steps: Vec<ValidationStep>,
    /// Estimated deployment time
    pub estimated_deployment_time: DeploymentTime,
}

/// Overall assessment of changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallAssessment {
    /// Low-risk, routine changes
    Routine,
    /// Moderate changes requiring review
    Moderate,
    /// Significant changes requiring careful planning
    Significant,
    /// High-risk changes requiring extensive validation
    HighRisk,
}

/// Deployment recommendation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Recommendation description
    pub description: String,
    /// Priority level
    pub priority: Priority,
    /// Rationale for the recommendation
    pub rationale: String,
}

/// Recommendation categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    /// Testing recommendations
    Testing,
    /// Deployment approach recommendations
    Deployment,
    /// Rollback planning recommendations
    RollbackPlanning,
    /// Monitoring recommendations
    Monitoring,
    /// Documentation recommendations
    Documentation,
}

/// Priority levels for recommendations
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Validation step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationStep {
    /// Step description
    pub description: String,
    /// Step type
    pub step_type: ValidationStepType,
    /// Estimated time to complete
    pub estimated_time: String,
    /// Required tools or access
    pub requirements: Vec<String>,
}

/// Types of validation steps
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStepType {
    /// Pre-deployment validation
    PreDeployment,
    /// During deployment validation
    DuringDeployment,
    /// Post-deployment validation
    PostDeployment,
    /// Rollback validation
    RollbackValidation,
}

/// Deployment time estimation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentTime {
    /// Estimated preparation time
    pub preparation: String,
    /// Estimated deployment time
    pub deployment: String,
    /// Estimated validation time
    pub validation: String,
    /// Total estimated time
    pub total: String,
}

/// Main analyzer for diff results
pub struct DiffAnalyzer {
    /// Enable detailed analysis
    enable_detailed: bool,
    /// Enable impact analysis
    enable_impact: bool,
    /// Enable categorization
    enable_categorization: bool,
}

impl DiffAnalyzer {
    /// Create a new analyzer with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enable_detailed: true,
            enable_impact: true,
            enable_categorization: true,
        }
    }

    /// Create a new analyzer with minimal analysis
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            enable_detailed: false,
            enable_impact: false,
            enable_categorization: false,
        }
    }

    /// Enable or disable detailed analysis
    #[must_use]
    pub const fn with_detailed_analysis(mut self, enable: bool) -> Self {
        self.enable_detailed = enable;
        self
    }

    /// Enable or disable impact analysis
    #[must_use]
    pub const fn with_impact_analysis(mut self, enable: bool) -> Self {
        self.enable_impact = enable;
        self
    }

    /// Enable or disable categorization
    #[must_use]
    pub const fn with_categorization(mut self, enable: bool) -> Self {
        self.enable_categorization = enable;
        self
    }

    /// Perform comprehensive analysis of a diff result
    pub fn analyze(&self, diff_result: &DiffResult) -> Result<DiffAnalysis> {
        debug!("Starting comprehensive diff analysis");

        let statistics = self.calculate_statistics(diff_result)?;
        let impact_analysis = if self.enable_impact {
            self.perform_impact_analysis(diff_result)?
        } else {
            Vec::new()
        };
        let categorization = if self.enable_categorization {
            self.categorize_changes(diff_result)?
        } else {
            ChangeCategorization::default()
        };
        let summary = self.generate_summary(diff_result, &statistics, &categorization)?;

        Ok(DiffAnalysis {
            statistics,
            impact_analysis,
            categorization,
            summary,
        })
    }

    /// Calculate detailed statistics for the diff
    fn calculate_statistics(&self, diff_result: &DiffResult) -> Result<DiffStatistics> {
        debug!("Calculating diff statistics");

        let text_diff = &diff_result.text_diff;
        let hierarchical_diff = &diff_result.hierarchical_diff;
        let semantic_diff = &diff_result.semantic_diff;

        // Basic change counts
        let change_counts = ChangeCounts {
            total: text_diff.additions + text_diff.deletions + text_diff.modifications,
            additions: text_diff.additions,
            deletions: text_diff.deletions,
            modifications: text_diff.modifications,
            unchanged: text_diff
                .changes
                .iter()
                .filter(|c| c.change_type == DiffType::Unchanged)
                .count(),
        };

        // Line statistics
        let old_total_lines = text_diff
            .changes
            .iter()
            .filter_map(|c| c.old_line_number)
            .max()
            .unwrap_or(0);
        let new_total_lines = text_diff
            .changes
            .iter()
            .filter_map(|c| c.new_line_number)
            .max()
            .unwrap_or(0);
        let affected_lines = text_diff
            .changes
            .iter()
            .filter(|c| c.change_type != DiffType::Unchanged)
            .count();
        let change_percentage = if old_total_lines > 0 {
            (affected_lines as f64 / old_total_lines as f64) * 100.0
        } else {
            0.0
        };
        let average_lines_per_change = if change_counts.total > 0 {
            affected_lines as f64 / change_counts.total as f64
        } else {
            0.0
        };

        let line_statistics = LineStatistics {
            old_total_lines,
            new_total_lines,
            affected_lines,
            change_percentage,
            average_lines_per_change,
        };

        // Section statistics
        let sections_added = hierarchical_diff
            .sections
            .iter()
            .filter(|s| s.change_type == DiffType::Addition)
            .count();
        let sections_removed = hierarchical_diff
            .sections
            .iter()
            .filter(|s| s.change_type == DiffType::Deletion)
            .count();
        let sections_modified = hierarchical_diff
            .sections
            .iter()
            .filter(|s| s.change_type == DiffType::Modification)
            .count();
        let sections_unchanged = hierarchical_diff
            .sections
            .iter()
            .filter(|s| s.change_type == DiffType::Unchanged)
            .count();
        let total_sections =
            sections_added + sections_removed + sections_modified + sections_unchanged;
        let section_change_percentage = if total_sections > 0 {
            ((sections_added + sections_removed + sections_modified) as f64 / total_sections as f64)
                * 100.0
        } else {
            0.0
        };

        let section_statistics = SectionStatistics {
            old_total_sections: sections_removed + sections_modified + sections_unchanged,
            new_total_sections: sections_added + sections_modified + sections_unchanged,
            sections_added,
            sections_removed,
            sections_modified,
            sections_unchanged,
            section_change_percentage,
        };

        // Distribution statistics
        let mut severity_distribution = HashMap::new();
        let mut risk_distribution = HashMap::new();
        let mut change_type_distribution = HashMap::new();
        let mut functional_change_distribution = HashMap::new();

        // Count severity distribution from structure changes
        for change in &hierarchical_diff.structure_changes {
            *severity_distribution
                .entry(change.severity.clone())
                .or_insert(0) += 1;
        }

        // Count risk distribution from impact analysis
        for impact in &semantic_diff.impact_analysis {
            *risk_distribution
                .entry(impact.risk_level.clone())
                .or_insert(0) += 1;
        }

        // Count change type distribution
        for change in &text_diff.changes {
            *change_type_distribution
                .entry(change.change_type.clone())
                .or_insert(0) += 1;
        }

        // Count functional change distribution
        for change in &semantic_diff.functional_changes {
            *functional_change_distribution
                .entry(change.change_type.clone())
                .or_insert(0) += 1;
        }

        Ok(DiffStatistics {
            change_counts,
            line_statistics,
            section_statistics,
            severity_distribution,
            risk_distribution,
            change_type_distribution,
            functional_change_distribution,
        })
    }

    /// Perform enhanced impact analysis
    fn perform_impact_analysis(&self, diff_result: &DiffResult) -> Result<Vec<ImpactAnalysis>> {
        debug!("Performing enhanced impact analysis");

        let mut analysis = diff_result.semantic_diff.impact_analysis.clone();

        // Add additional impact analysis based on change patterns
        if self.has_interface_changes(&diff_result.hierarchical_diff) {
            analysis.push(ImpactAnalysis {
                affected_components: vec!["Network Interfaces".to_string()],
                risk_level: RiskLevel::Medium,
                impact_description:
                    "Interface configuration changes may affect network connectivity".to_string(),
                validation_steps: vec![
                    "Verify interface status after changes".to_string(),
                    "Test connectivity to connected devices".to_string(),
                    "Monitor interface error counters".to_string(),
                ],
            });
        }

        if self.has_routing_changes(diff_result) {
            analysis.push(ImpactAnalysis {
                affected_components: vec![
                    "Routing Tables".to_string(),
                    "Network Reachability".to_string(),
                ],
                risk_level: RiskLevel::High,
                impact_description:
                    "Routing changes may affect network reachability and traffic flow".to_string(),
                validation_steps: vec![
                    "Verify routing table convergence".to_string(),
                    "Test end-to-end connectivity".to_string(),
                    "Monitor routing protocol adjacencies".to_string(),
                ],
            });
        }

        if self.has_security_changes(diff_result) {
            analysis.push(ImpactAnalysis {
                affected_components: vec![
                    "Security Policies".to_string(),
                    "Access Control".to_string(),
                ],
                risk_level: RiskLevel::High,
                impact_description:
                    "Security configuration changes may affect access control and compliance"
                        .to_string(),
                validation_steps: vec![
                    "Verify access control lists are working correctly".to_string(),
                    "Test security policy enforcement".to_string(),
                    "Review security audit logs".to_string(),
                ],
            });
        }

        Ok(analysis)
    }

    /// Categorize changes by different dimensions
    fn categorize_changes(&self, diff_result: &DiffResult) -> Result<ChangeCategorization> {
        debug!("Categorizing changes");

        let mut by_function = HashMap::new();
        let mut by_section = HashMap::new();
        let mut by_vendor_feature = HashMap::new();
        let mut by_impact = HashMap::new();

        // Categorize hierarchical changes
        for section in &diff_result.hierarchical_diff.sections {
            let path = &section.path;

            // Categorize by function
            let function = self.classify_network_function(path);
            by_function
                .entry(function)
                .or_insert_with(Vec::new)
                .push(path.clone());

            // Categorize by section
            let section_type = self.classify_config_section(path);
            by_section
                .entry(section_type)
                .or_insert_with(Vec::new)
                .push(path.clone());

            // Categorize by vendor feature
            let vendor_feature = self.classify_vendor_feature(path);
            by_vendor_feature
                .entry(vendor_feature)
                .or_insert_with(Vec::new)
                .push(path.clone());

            // Categorize by impact
            let impact = self.classify_operational_impact(path, &section.change_type);
            by_impact
                .entry(impact)
                .or_insert_with(Vec::new)
                .push(path.clone());
        }

        Ok(ChangeCategorization {
            by_function,
            by_section,
            by_vendor_feature,
            by_impact,
        })
    }

    /// Generate analysis summary with recommendations
    fn generate_summary(
        &self,
        diff_result: &DiffResult,
        statistics: &DiffStatistics,
        categorization: &ChangeCategorization,
    ) -> Result<AnalysisSummary> {
        debug!("Generating analysis summary");

        let overall_assessment = self.assess_overall_risk(diff_result, statistics);
        let key_findings = self.generate_key_findings(statistics, categorization);
        let recommendations = self.generate_recommendations(&overall_assessment, statistics);
        let validation_steps = self.generate_validation_steps(&overall_assessment, categorization);
        let estimated_deployment_time =
            self.estimate_deployment_time(&overall_assessment, statistics);

        Ok(AnalysisSummary {
            overall_assessment,
            key_findings,
            recommendations,
            validation_steps,
            estimated_deployment_time,
        })
    }

    // Helper methods for analysis

    fn has_interface_changes(&self, hierarchical_diff: &HierarchicalDiff) -> bool {
        hierarchical_diff
            .sections
            .iter()
            .any(|s| s.path.starts_with("interface") || s.path.contains("interface"))
    }

    fn has_routing_changes(&self, diff_result: &DiffResult) -> bool {
        diff_result
            .semantic_diff
            .functional_changes
            .iter()
            .any(|c| matches!(c.change_type, FunctionalChangeType::Routing))
    }

    fn has_security_changes(&self, diff_result: &DiffResult) -> bool {
        diff_result
            .semantic_diff
            .functional_changes
            .iter()
            .any(|c| {
                matches!(
                    c.change_type,
                    FunctionalChangeType::AccessControl | FunctionalChangeType::Security
                )
            })
    }

    fn classify_network_function(&self, path: &str) -> NetworkFunction {
        if path.contains("interface") || path.contains("vlan") {
            NetworkFunction::Switching
        } else if path.contains("route") || path.contains("bgp") || path.contains("ospf") {
            NetworkFunction::Routing
        } else if path.contains("access-list") || path.contains("security") || path.contains("acl")
        {
            NetworkFunction::Security
        } else if path.contains("qos") || path.contains("class-map") || path.contains("policy-map")
        {
            NetworkFunction::QualityOfService
        } else if path.contains("snmp") || path.contains("ntp") || path.contains("logging") {
            NetworkFunction::Management
        } else {
            NetworkFunction::Other("Unknown".to_string())
        }
    }

    fn classify_config_section(&self, path: &str) -> ConfigSection {
        if path.starts_with("interface") {
            ConfigSection::Interfaces
        } else if path.starts_with("vlan") {
            ConfigSection::Vlans
        } else if path.contains("route") || path.contains("router") {
            ConfigSection::Routing
        } else if path.contains("access-list") || path.contains("acl") {
            ConfigSection::AccessLists
        } else {
            ConfigSection::Other("Unknown".to_string())
        }
    }

    fn classify_vendor_feature(&self, path: &str) -> VendorFeature {
        // This is a simplified classification - could be enhanced with vendor-specific patterns
        if path.contains("GigabitEthernet") || path.contains("FastEthernet") {
            VendorFeature::Cisco("Interface Naming".to_string())
        } else if path.contains("ge-") || path.contains("xe-") {
            VendorFeature::Juniper("Interface Naming".to_string())
        } else {
            VendorFeature::Standard("Generic".to_string())
        }
    }

    fn classify_operational_impact(&self, path: &str, change_type: &DiffType) -> OperationalImpact {
        match change_type {
            DiffType::Addition | DiffType::Deletion => {
                if path.contains("interface") {
                    OperationalImpact::Connectivity
                } else if path.contains("route") {
                    OperationalImpact::Connectivity
                } else if path.contains("security") || path.contains("access-list") {
                    OperationalImpact::Security
                } else {
                    OperationalImpact::Performance
                }
            }
            DiffType::Modification => {
                if path.contains("description") || path.contains("name") {
                    OperationalImpact::Cosmetic
                } else {
                    OperationalImpact::Performance
                }
            }
            DiffType::Unchanged => OperationalImpact::Cosmetic,
        }
    }

    const fn assess_overall_risk(
        &self,
        diff_result: &DiffResult,
        statistics: &DiffStatistics,
    ) -> OverallAssessment {
        let total_changes = statistics.change_counts.total;
        let highest_risk = &diff_result.summary.highest_risk;
        let complexity = &diff_result.summary.complexity;

        match (total_changes, highest_risk, complexity) {
            (0..=5, RiskLevel::Low, ChangeComplexity::Simple) => OverallAssessment::Routine,
            (
                6..=20,
                RiskLevel::Low | RiskLevel::Medium,
                ChangeComplexity::Simple | ChangeComplexity::Moderate,
            ) => OverallAssessment::Moderate,
            (_, RiskLevel::High, _) | (21..=50, _, ChangeComplexity::Complex) => {
                OverallAssessment::Significant
            }
            (_, RiskLevel::Critical, _) | (_, _, ChangeComplexity::VeryComplex) => {
                OverallAssessment::HighRisk
            }
            _ => OverallAssessment::Moderate,
        }
    }

    fn generate_key_findings(
        &self,
        statistics: &DiffStatistics,
        categorization: &ChangeCategorization,
    ) -> Vec<String> {
        let mut findings = Vec::new();

        findings.push(format!(
            "Total of {} changes detected ({} additions, {} deletions, {} modifications)",
            statistics.change_counts.total,
            statistics.change_counts.additions,
            statistics.change_counts.deletions,
            statistics.change_counts.modifications
        ));

        findings.push(format!(
            "{:.1}% of configuration lines affected",
            statistics.line_statistics.change_percentage
        ));

        if statistics.section_statistics.sections_added > 0 {
            findings.push(format!(
                "{} new configuration sections added",
                statistics.section_statistics.sections_added
            ));
        }

        if statistics.section_statistics.sections_removed > 0 {
            findings.push(format!(
                "{} configuration sections removed",
                statistics.section_statistics.sections_removed
            ));
        }

        // Add findings based on categorization
        for (function, paths) in &categorization.by_function {
            if !paths.is_empty() {
                findings.push(format!(
                    "{} changes affect {:?} functionality",
                    paths.len(),
                    function
                ));
            }
        }

        findings
    }

    fn generate_recommendations(
        &self,
        assessment: &OverallAssessment,
        statistics: &DiffStatistics,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        match assessment {
            OverallAssessment::Routine => {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Testing,
                    description: "Basic functional testing sufficient".to_string(),
                    priority: Priority::Low,
                    rationale: "Low-risk routine changes with minimal impact".to_string(),
                });
            }
            OverallAssessment::Moderate => {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Testing,
                    description:
                        "Perform comprehensive testing including connectivity verification"
                            .to_string(),
                    priority: Priority::Medium,
                    rationale: "Moderate changes requiring verification of functionality"
                        .to_string(),
                });
            }
            OverallAssessment::Significant => {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Testing,
                    description: "Extensive testing required including impact analysis".to_string(),
                    priority: Priority::High,
                    rationale: "Significant changes with potential for service impact".to_string(),
                });
                recommendations.push(Recommendation {
                    category: RecommendationCategory::RollbackPlanning,
                    description: "Prepare detailed rollback procedures".to_string(),
                    priority: Priority::High,
                    rationale: "High-impact changes require robust rollback capability".to_string(),
                });
            }
            OverallAssessment::HighRisk => {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Testing,
                    description: "Full regression testing and staged deployment required"
                        .to_string(),
                    priority: Priority::Critical,
                    rationale:
                        "High-risk changes with potential for significant service disruption"
                            .to_string(),
                });
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Deployment,
                    description: "Deploy during maintenance window with expert supervision"
                        .to_string(),
                    priority: Priority::Critical,
                    rationale: "Complex changes require controlled deployment environment"
                        .to_string(),
                });
            }
        }

        // Add specific recommendations based on statistics
        if statistics.section_statistics.sections_removed > 0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::RollbackPlanning,
                description: "Document removed configurations for potential restoration"
                    .to_string(),
                priority: Priority::High,
                rationale:
                    "Configuration removal can be difficult to reverse without documentation"
                        .to_string(),
            });
        }

        recommendations
    }

    fn generate_validation_steps(
        &self,
        assessment: &OverallAssessment,
        categorization: &ChangeCategorization,
    ) -> Vec<ValidationStep> {
        let mut steps = Vec::new();

        // Common pre-deployment steps
        steps.push(ValidationStep {
            description: "Backup current configuration".to_string(),
            step_type: ValidationStepType::PreDeployment,
            estimated_time: "5 minutes".to_string(),
            requirements: vec!["Configuration management access".to_string()],
        });

        // Assessment-specific steps
        match assessment {
            OverallAssessment::HighRisk | OverallAssessment::Significant => {
                steps.push(ValidationStep {
                    description: "Verify configuration syntax in test environment".to_string(),
                    step_type: ValidationStepType::PreDeployment,
                    estimated_time: "30 minutes".to_string(),
                    requirements: vec![
                        "Test environment access".to_string(),
                        "Configuration validation tools".to_string(),
                    ],
                });
            }
            _ => {}
        }

        // Function-specific validation steps
        if categorization
            .by_function
            .contains_key(&NetworkFunction::Routing)
        {
            steps.push(ValidationStep {
                description: "Verify routing table convergence".to_string(),
                step_type: ValidationStepType::PostDeployment,
                estimated_time: "10 minutes".to_string(),
                requirements: vec!["Network monitoring tools".to_string()],
            });
        }

        if categorization
            .by_function
            .contains_key(&NetworkFunction::Security)
        {
            steps.push(ValidationStep {
                description: "Test security policy enforcement".to_string(),
                step_type: ValidationStepType::PostDeployment,
                estimated_time: "20 minutes".to_string(),
                requirements: vec!["Security testing tools".to_string()],
            });
        }

        steps
    }

    fn estimate_deployment_time(
        &self,
        assessment: &OverallAssessment,
        statistics: &DiffStatistics,
    ) -> DeploymentTime {
        let base_deployment_minutes = match assessment {
            OverallAssessment::Routine => 5,
            OverallAssessment::Moderate => 15,
            OverallAssessment::Significant => 30,
            OverallAssessment::HighRisk => 60,
        };

        // Adjust based on change volume
        let change_factor = (statistics.change_counts.total as f64 / 10.0).max(1.0);
        let adjusted_deployment = (f64::from(base_deployment_minutes) * change_factor) as u32;

        let preparation_time = adjusted_deployment / 2;
        let validation_time = adjusted_deployment;
        let total_time = preparation_time + adjusted_deployment + validation_time;

        DeploymentTime {
            preparation: format!("{preparation_time} minutes"),
            deployment: format!("{adjusted_deployment} minutes"),
            validation: format!("{validation_time} minutes"),
            total: format!("{total_time} minutes"),
        }
    }
}

impl Default for DiffAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::*;

    fn create_test_diff_result() -> DiffResult {
        DiffResult {
            text_diff: TextDiff {
                changes: vec![
                    DiffChange {
                        change_type: DiffType::Addition,
                        old_line: None,
                        new_line: Some("interface GigabitEthernet0/2".to_string()),
                        old_line_number: None,
                        new_line_number: Some(5),
                        context: None,
                    },
                    DiffChange {
                        change_type: DiffType::Deletion,
                        old_line: Some("no ip address".to_string()),
                        new_line: None,
                        old_line_number: Some(3),
                        new_line_number: None,
                        context: None,
                    },
                ],
                additions: 1,
                deletions: 1,
                modifications: 0,
                context_lines: 3,
            },
            hierarchical_diff: HierarchicalDiff {
                sections: vec![DiffSection {
                    path: "interface.GigabitEthernet0/2".to_string(),
                    change_type: DiffType::Addition,
                    old_section: None,
                    new_section: None,
                    child_changes: Vec::new(),
                }],
                structure_changes: Vec::new(),
                path_changes: std::collections::HashMap::new(),
            },
            semantic_diff: SemanticDiff {
                functional_changes: vec![FunctionalChange {
                    change_type: FunctionalChangeType::InterfaceState,
                    description: "Interface added".to_string(),
                    path: "interface.GigabitEthernet0/2".to_string(),
                    old_value: None,
                    new_value: Some("enabled".to_string()),
                    severity: ChangeSeverity::Info,
                }],
                impact_analysis: vec![ImpactAnalysis {
                    affected_components: vec!["Network Interfaces".to_string()],
                    risk_level: RiskLevel::Low,
                    impact_description: "New interface added".to_string(),
                    validation_steps: vec!["Test connectivity".to_string()],
                }],
                change_groups: Vec::new(),
            },
            summary: DiffSummary {
                total_changes: 2,
                additions: 1,
                deletions: 1,
                modifications: 0,
                sections_affected: 1,
                highest_risk: RiskLevel::Low,
                complexity: ChangeComplexity::Simple,
            },
            options: DiffOptions::default(),
        }
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = DiffAnalyzer::new();
        assert!(analyzer.enable_detailed);
        assert!(analyzer.enable_impact);
        assert!(analyzer.enable_categorization);
    }

    #[test]
    fn test_minimal_analyzer() {
        let analyzer = DiffAnalyzer::minimal();
        assert!(!analyzer.enable_detailed);
        assert!(!analyzer.enable_impact);
        assert!(!analyzer.enable_categorization);
    }

    #[test]
    fn test_statistics_calculation() {
        let analyzer = DiffAnalyzer::new();
        let diff_result = create_test_diff_result();

        let statistics = analyzer.calculate_statistics(&diff_result).unwrap();
        assert_eq!(statistics.change_counts.total, 2);
        assert_eq!(statistics.change_counts.additions, 1);
        assert_eq!(statistics.change_counts.deletions, 1);
    }

    #[test]
    fn test_network_function_classification() {
        let analyzer = DiffAnalyzer::new();

        assert_eq!(
            analyzer.classify_network_function("interface.GigabitEthernet0/1"),
            NetworkFunction::Switching
        );
        assert_eq!(
            analyzer.classify_network_function("router.bgp.65001"),
            NetworkFunction::Routing
        );
        assert_eq!(
            analyzer.classify_network_function("access-list.100"),
            NetworkFunction::Security
        );
    }

    #[test]
    fn test_config_section_classification() {
        let analyzer = DiffAnalyzer::new();

        assert_eq!(
            analyzer.classify_config_section("interface.GigabitEthernet0/1"),
            ConfigSection::Interfaces
        );
        assert_eq!(
            analyzer.classify_config_section("vlan.100"),
            ConfigSection::Vlans
        );
        assert_eq!(
            analyzer.classify_config_section("router.ospf"),
            ConfigSection::Routing
        );
    }

    #[test]
    fn test_comprehensive_analysis() {
        let analyzer = DiffAnalyzer::new();
        let diff_result = create_test_diff_result();

        let analysis = analyzer.analyze(&diff_result).unwrap();
        assert!(analysis.statistics.change_counts.total > 0);
        assert!(!analysis.impact_analysis.is_empty());
        assert!(!analysis.categorization.by_function.is_empty());
        assert!(!analysis.summary.key_findings.is_empty());
    }

    #[test]
    fn test_overall_assessment() {
        let analyzer = DiffAnalyzer::new();
        let diff_result = create_test_diff_result();
        let statistics = analyzer.calculate_statistics(&diff_result).unwrap();

        let assessment = analyzer.assess_overall_risk(&diff_result, &statistics);
        assert_eq!(assessment, OverallAssessment::Routine);
    }
}
