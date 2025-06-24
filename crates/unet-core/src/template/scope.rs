//! Template scope management and resolution system
//!
//! This module provides functionality for resolving template scope conflicts,
//! managing template priorities, and composing templates for configuration generation.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use tracing::{debug, warn};

use super::header::{HeaderParser, TemplateHeader, TemplateScope};

/// Template scope context for resolution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopeContext {
    /// Device type (e.g., "router", "switch", "firewall")
    pub device_type: Option<String>,
    /// Vendor (e.g., "cisco", "juniper", "arista")
    pub vendor: Option<String>,
    /// Configuration context (e.g., "global", "interface", "routing")
    pub context: String,
    /// Additional metadata for scope matching
    pub metadata: HashMap<String, String>,
}

/// Template with resolved scope information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopedTemplate {
    /// Template identifier
    pub id: String,
    /// Template content
    pub content: String,
    /// Parsed header information
    pub header: TemplateHeader,
    /// Resolved scope context
    pub scope_context: ScopeContext,
    /// Calculated priority (higher = more priority)
    pub resolved_priority: u32,
}

/// Template scope resolver
#[derive(Debug, Clone)]
pub struct ScopeResolver {
    /// Default priority for templates without explicit priority
    default_priority: u32,
    /// Priority boost for vendor-specific templates
    vendor_priority_boost: u32,
    /// Priority boost for device-type-specific templates
    device_type_priority_boost: u32,
}

impl ScopeResolver {
    /// Create a new scope resolver with default configuration
    pub fn new() -> Self {
        Self {
            default_priority: 100,
            vendor_priority_boost: 50,
            device_type_priority_boost: 25,
        }
    }

    /// Create a scope resolver with custom priority configuration
    pub fn with_priorities(default: u32, vendor_boost: u32, device_type_boost: u32) -> Self {
        Self {
            default_priority: default,
            vendor_priority_boost: vendor_boost,
            device_type_priority_boost: device_type_boost,
        }
    }

    /// Resolve template scope and calculate priority
    ///
    /// # Arguments
    ///
    /// * `template_id` - Unique identifier for the template
    /// * `template_content` - The template content
    /// * `header` - Parsed template header
    /// * `target_context` - Target scope context for resolution
    ///
    /// # Returns
    ///
    /// A `ScopedTemplate` with resolved scope and priority, or an error if the template
    /// doesn't match the target context.
    pub fn resolve_scope(
        &self,
        template_id: &str,
        template_content: &str,
        header: &TemplateHeader,
        target_context: &ScopeContext,
    ) -> Result<Option<ScopedTemplate>> {
        // Check if template scope matches the target context
        if let Some(template_scope) = &header.scope {
            if !self.scope_matches(template_scope, target_context)? {
                debug!(
                    "Template '{}' scope doesn't match target context",
                    template_id
                );
                return Ok(None);
            }
        }

        // Calculate resolved priority
        let resolved_priority = self.calculate_priority(header, target_context);

        Ok(Some(ScopedTemplate {
            id: template_id.to_string(),
            content: template_content.to_string(),
            header: header.clone(),
            scope_context: target_context.clone(),
            resolved_priority,
        }))
    }

    /// Check if a template scope matches the target context
    fn scope_matches(
        &self,
        template_scope: &TemplateScope,
        target_context: &ScopeContext,
    ) -> Result<bool> {
        // Check device type match
        if let Some(allowed_device_types) = &template_scope.device_types {
            if let Some(target_device_type) = &target_context.device_type {
                if !allowed_device_types.contains(target_device_type) {
                    return Ok(false);
                }
            } else {
                // Template requires device type but target doesn't have one
                return Ok(false);
            }
        }

        // Check vendor match
        if let Some(allowed_vendors) = &template_scope.vendors {
            if let Some(target_vendor) = &target_context.vendor {
                if !allowed_vendors.contains(target_vendor) {
                    return Ok(false);
                }
            } else {
                // Template requires vendor but target doesn't have one
                return Ok(false);
            }
        }

        // Check context match
        if let Some(allowed_contexts) = &template_scope.contexts {
            if !allowed_contexts.contains(&target_context.context) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Calculate resolved priority for a template in the given context
    fn calculate_priority(&self, header: &TemplateHeader, target_context: &ScopeContext) -> u32 {
        let mut priority = header.priority.unwrap_or(self.default_priority);

        // Apply scope-based priority boosts
        if let Some(template_scope) = &header.scope {
            // Boost priority for vendor-specific templates
            if let Some(allowed_vendors) = &template_scope.vendors {
                if let Some(target_vendor) = &target_context.vendor {
                    if allowed_vendors.len() == 1 && allowed_vendors.contains(target_vendor) {
                        priority += self.vendor_priority_boost;
                    }
                }
            }

            // Boost priority for device-type-specific templates
            if let Some(allowed_device_types) = &template_scope.device_types {
                if let Some(target_device_type) = &target_context.device_type {
                    if allowed_device_types.len() == 1
                        && allowed_device_types.contains(target_device_type)
                    {
                        priority += self.device_type_priority_boost;
                    }
                }
            }
        }

        priority
    }

    /// Resolve multiple templates for a given context and return them in priority order
    ///
    /// # Arguments
    ///
    /// * `templates` - Map of template ID to (content, header) pairs
    /// * `target_context` - Target scope context for resolution
    ///
    /// # Returns
    ///
    /// A vector of `ScopedTemplate` objects sorted by priority (highest first)
    pub fn resolve_multiple(
        &self,
        templates: &HashMap<String, (String, TemplateHeader)>,
        target_context: &ScopeContext,
    ) -> Result<Vec<ScopedTemplate>> {
        let mut resolved_templates = Vec::new();

        for (template_id, (content, header)) in templates {
            if let Some(scoped_template) =
                self.resolve_scope(template_id, content, header, target_context)?
            {
                resolved_templates.push(scoped_template);
            }
        }

        // Sort by priority (highest first), then by template ID for deterministic ordering
        resolved_templates.sort_by(|a, b| match b.resolved_priority.cmp(&a.resolved_priority) {
            Ordering::Equal => a.id.cmp(&b.id),
            other => other,
        });

        debug!(
            "Resolved {} templates for context: {:?}",
            resolved_templates.len(),
            target_context
        );

        Ok(resolved_templates)
    }
}

impl Default for ScopeResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Template conflict detection system
#[derive(Debug, Clone)]
pub struct ConflictDetector {
    /// Resolver for scope management
    resolver: ScopeResolver,
}

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new() -> Self {
        Self {
            resolver: ScopeResolver::new(),
        }
    }

    /// Create a conflict detector with custom resolver
    pub fn with_resolver(resolver: ScopeResolver) -> Self {
        Self { resolver }
    }

    /// Detect potential conflicts between templates
    ///
    /// # Arguments
    ///
    /// * `templates` - Map of template ID to (content, header) pairs
    /// * `target_context` - Target scope context
    ///
    /// # Returns
    ///
    /// A vector of conflict reports
    pub fn detect_conflicts(
        &self,
        templates: &HashMap<String, (String, TemplateHeader)>,
        target_context: &ScopeContext,
    ) -> Result<Vec<TemplateConflict>> {
        let resolved_templates = self.resolver.resolve_multiple(templates, target_context)?;
        let mut conflicts = Vec::new();

        // Group templates by configuration path pattern
        let mut pattern_groups: HashMap<String, Vec<&ScopedTemplate>> = HashMap::new();

        for template in &resolved_templates {
            let pattern_key = self.get_pattern_key(&template.header)?;
            pattern_groups
                .entry(pattern_key)
                .or_default()
                .push(template);
        }

        // Check for conflicts within each pattern group
        for (pattern, group_templates) in pattern_groups {
            if group_templates.len() > 1 {
                // Sort by priority to identify the winning template
                let mut sorted_group = group_templates;
                sorted_group.sort_by(|a, b| b.resolved_priority.cmp(&a.resolved_priority));

                let winning_template = sorted_group[0];
                let conflicting_templates: Vec<String> =
                    sorted_group[1..].iter().map(|t| t.id.clone()).collect();

                conflicts.push(TemplateConflict {
                    pattern: pattern.clone(),
                    winning_template: winning_template.id.clone(),
                    conflicting_templates,
                    resolution_reason: self
                        .get_resolution_reason(winning_template, &sorted_group[1..]),
                });
            }
        }

        if !conflicts.is_empty() {
            warn!(
                "Detected {} template conflicts for context: {:?}",
                conflicts.len(),
                target_context
            );
        }

        Ok(conflicts)
    }

    /// Get a pattern key for grouping templates that might conflict
    fn get_pattern_key(&self, header: &TemplateHeader) -> Result<String> {
        use super::header::MatchPattern;

        match &header.pattern {
            MatchPattern::Exact(pattern) => Ok(pattern.clone()),
            MatchPattern::Regex(pattern) => Ok(format!("regex:{}", pattern)),
            MatchPattern::HierarchicalPath(pattern) => Ok(format!("hierarchical:{}", pattern)),
            MatchPattern::Glob(pattern) => Ok(format!("glob:{}", pattern)),
        }
    }

    /// Get the reason why one template won over others
    fn get_resolution_reason(&self, winner: &ScopedTemplate, losers: &[&ScopedTemplate]) -> String {
        if winner.resolved_priority > losers[0].resolved_priority {
            format!(
                "Higher priority ({} vs {})",
                winner.resolved_priority, losers[0].resolved_priority
            )
        } else {
            "Lexicographic ordering by template ID".to_string()
        }
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Template conflict report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateConflict {
    /// Configuration pattern that has conflicts
    pub pattern: String,
    /// ID of the winning template
    pub winning_template: String,
    /// IDs of templates that were overridden
    pub conflicting_templates: Vec<String>,
    /// Reason for the conflict resolution
    pub resolution_reason: String,
}

/// Template composition system for combining multiple templates
#[derive(Debug, Clone)]
pub struct TemplateComposer {
    /// Resolver for scope management
    resolver: ScopeResolver,
    /// Conflict detector for identifying template conflicts
    conflict_detector: ConflictDetector,
}

impl TemplateComposer {
    /// Create a new template composer
    pub fn new() -> Self {
        let resolver = ScopeResolver::new();
        let conflict_detector = ConflictDetector::with_resolver(resolver.clone());

        Self {
            resolver,
            conflict_detector,
        }
    }

    /// Create a composer with custom resolver
    pub fn with_resolver(resolver: ScopeResolver) -> Self {
        let conflict_detector = ConflictDetector::with_resolver(resolver.clone());

        Self {
            resolver,
            conflict_detector,
        }
    }

    /// Compose templates for a given context, resolving conflicts and applying priority ordering
    ///
    /// # Arguments
    ///
    /// * `templates` - Map of template ID to (content, header) pairs
    /// * `target_context` - Target scope context for composition
    /// * `config_path` - Configuration path to match against template headers
    ///
    /// # Returns
    ///
    /// A `CompositionResult` containing the selected templates and any conflicts detected
    pub async fn compose(
        &self,
        templates: &HashMap<String, (String, TemplateHeader)>,
        target_context: &ScopeContext,
        config_path: &str,
    ) -> Result<CompositionResult> {
        // First, resolve all templates for the given context
        let resolved_templates = self.resolver.resolve_multiple(templates, target_context)?;

        // Filter templates that match the specific configuration path
        let mut matching_templates = Vec::new();
        let mut header_parser = HeaderParser::new();

        for scoped_template in resolved_templates {
            if header_parser.matches(&scoped_template.header, config_path)? {
                matching_templates.push(scoped_template);
            }
        }

        // Detect conflicts among matching templates
        let matching_templates_map: HashMap<String, (String, TemplateHeader)> = matching_templates
            .iter()
            .map(|st| (st.id.clone(), (st.content.clone(), st.header.clone())))
            .collect();

        let conflicts = self
            .conflict_detector
            .detect_conflicts(&matching_templates_map, target_context)?;

        // Build final template selection, excluding conflicted templates
        let mut final_templates = Vec::new();
        let conflicted_template_ids: std::collections::HashSet<String> = conflicts
            .iter()
            .flat_map(|c| c.conflicting_templates.iter().cloned())
            .collect();

        for template in matching_templates {
            if !conflicted_template_ids.contains(&template.id) {
                final_templates.push(template);
            }
        }

        // Sort final templates by priority (highest first)
        final_templates.sort_by(|a, b| match b.resolved_priority.cmp(&a.resolved_priority) {
            Ordering::Equal => a.id.cmp(&b.id),
            other => other,
        });

        debug!(
            "Composed {} templates for config path '{}' in context: {:?}",
            final_templates.len(),
            config_path,
            target_context
        );

        Ok(CompositionResult {
            selected_templates: final_templates,
            conflicts,
            config_path: config_path.to_string(),
            context: target_context.clone(),
        })
    }

    /// Compose templates for multiple configuration paths
    ///
    /// # Arguments
    ///
    /// * `templates` - Map of template ID to (content, header) pairs
    /// * `target_context` - Target scope context for composition
    /// * `config_paths` - List of configuration paths to process
    ///
    /// # Returns
    ///
    /// A map of configuration path to `CompositionResult`
    pub async fn compose_multiple(
        &self,
        templates: &HashMap<String, (String, TemplateHeader)>,
        target_context: &ScopeContext,
        config_paths: &[String],
    ) -> Result<HashMap<String, CompositionResult>> {
        let mut results = HashMap::new();

        for config_path in config_paths {
            let result = self.compose(templates, target_context, config_path).await?;
            results.insert(config_path.clone(), result);
        }

        Ok(results)
    }

    /// Get template composition summary for reporting
    pub async fn get_composition_summary(
        &self,
        templates: &HashMap<String, (String, TemplateHeader)>,
        target_context: &ScopeContext,
        config_paths: &[String],
    ) -> Result<CompositionSummary> {
        let results = self
            .compose_multiple(templates, target_context, config_paths)
            .await?;

        let total_templates = templates.len();
        let total_conflicts: usize = results.values().map(|r| r.conflicts.len()).sum();
        let total_selected: usize = results.values().map(|r| r.selected_templates.len()).sum();

        let conflict_patterns: Vec<String> = results
            .values()
            .flat_map(|r| r.conflicts.iter().map(|c| c.pattern.clone()))
            .collect();

        Ok(CompositionSummary {
            total_templates_available: total_templates,
            total_templates_selected: total_selected,
            total_conflicts_detected: total_conflicts,
            conflict_patterns,
            composition_results: results,
        })
    }
}

impl Default for TemplateComposer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of template composition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionResult {
    /// Templates selected for this configuration path
    pub selected_templates: Vec<ScopedTemplate>,
    /// Conflicts detected during composition
    pub conflicts: Vec<TemplateConflict>,
    /// Configuration path this result applies to
    pub config_path: String,
    /// Scope context used for composition
    pub context: ScopeContext,
}

/// Summary of template composition across multiple paths
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionSummary {
    /// Total number of templates available for composition
    pub total_templates_available: usize,
    /// Total number of templates selected across all paths
    pub total_templates_selected: usize,
    /// Total number of conflicts detected
    pub total_conflicts_detected: usize,
    /// List of configuration patterns that had conflicts
    pub conflict_patterns: Vec<String>,
    /// Detailed composition results per configuration path
    pub composition_results: HashMap<String, CompositionResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::header::{MatchPattern, TemplateScope};

    fn create_test_context() -> ScopeContext {
        ScopeContext {
            device_type: Some("router".to_string()),
            vendor: Some("cisco".to_string()),
            context: "interface".to_string(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_header_with_scope(scope: Option<TemplateScope>) -> TemplateHeader {
        TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: None,
            scope,
        }
    }

    #[test]
    fn test_scope_resolver_creation() {
        let resolver = ScopeResolver::new();
        assert_eq!(resolver.default_priority, 100);
        assert_eq!(resolver.vendor_priority_boost, 50);
        assert_eq!(resolver.device_type_priority_boost, 25);
    }

    #[test]
    fn test_scope_matching_device_type() {
        let resolver = ScopeResolver::new();
        let target_context = create_test_context();

        // Matching device type
        let scope = TemplateScope {
            device_types: Some(vec!["router".to_string(), "switch".to_string()]),
            vendors: None,
            contexts: None,
        };
        assert!(resolver.scope_matches(&scope, &target_context).unwrap());

        // Non-matching device type
        let scope = TemplateScope {
            device_types: Some(vec!["switch".to_string()]),
            vendors: None,
            contexts: None,
        };
        assert!(!resolver.scope_matches(&scope, &target_context).unwrap());
    }

    #[test]
    fn test_scope_matching_vendor() {
        let resolver = ScopeResolver::new();
        let target_context = create_test_context();

        // Matching vendor
        let scope = TemplateScope {
            device_types: None,
            vendors: Some(vec!["cisco".to_string(), "juniper".to_string()]),
            contexts: None,
        };
        assert!(resolver.scope_matches(&scope, &target_context).unwrap());

        // Non-matching vendor
        let scope = TemplateScope {
            device_types: None,
            vendors: Some(vec!["juniper".to_string()]),
            contexts: None,
        };
        assert!(!resolver.scope_matches(&scope, &target_context).unwrap());
    }

    #[test]
    fn test_scope_matching_context() {
        let resolver = ScopeResolver::new();
        let target_context = create_test_context();

        // Matching context
        let scope = TemplateScope {
            device_types: None,
            vendors: None,
            contexts: Some(vec!["interface".to_string(), "routing".to_string()]),
        };
        assert!(resolver.scope_matches(&scope, &target_context).unwrap());

        // Non-matching context
        let scope = TemplateScope {
            device_types: None,
            vendors: None,
            contexts: Some(vec!["routing".to_string()]),
        };
        assert!(!resolver.scope_matches(&scope, &target_context).unwrap());
    }

    #[test]
    fn test_priority_calculation() {
        let resolver = ScopeResolver::new();
        let target_context = create_test_context();

        // Template with no specific scope
        let header = create_test_header_with_scope(None);
        assert_eq!(resolver.calculate_priority(&header, &target_context), 100);

        // Template with vendor-specific scope
        let scope = TemplateScope {
            device_types: None,
            vendors: Some(vec!["cisco".to_string()]),
            contexts: None,
        };
        let header = create_test_header_with_scope(Some(scope));
        assert_eq!(resolver.calculate_priority(&header, &target_context), 150); // 100 + 50

        // Template with both vendor and device type specific scope
        let scope = TemplateScope {
            device_types: Some(vec!["router".to_string()]),
            vendors: Some(vec!["cisco".to_string()]),
            contexts: None,
        };
        let header = create_test_header_with_scope(Some(scope));
        assert_eq!(resolver.calculate_priority(&header, &target_context), 175); // 100 + 50 + 25
    }

    #[test]
    fn test_resolve_scope_matching() {
        let resolver = ScopeResolver::new();
        let target_context = create_test_context();

        let scope = TemplateScope {
            device_types: Some(vec!["router".to_string()]),
            vendors: Some(vec!["cisco".to_string()]),
            contexts: Some(vec!["interface".to_string()]),
        };
        let header = create_test_header_with_scope(Some(scope));

        let result = resolver
            .resolve_scope(
                "test_template",
                "template content",
                &header,
                &target_context,
            )
            .unwrap();

        assert!(result.is_some());
        let scoped_template = result.unwrap();
        assert_eq!(scoped_template.id, "test_template");
        assert_eq!(scoped_template.content, "template content");
        assert_eq!(scoped_template.resolved_priority, 175);
    }

    #[test]
    fn test_resolve_scope_non_matching() {
        let resolver = ScopeResolver::new();
        let target_context = create_test_context();

        let scope = TemplateScope {
            device_types: Some(vec!["switch".to_string()]), // Non-matching device type
            vendors: Some(vec!["cisco".to_string()]),
            contexts: Some(vec!["interface".to_string()]),
        };
        let header = create_test_header_with_scope(Some(scope));

        let result = resolver
            .resolve_scope(
                "test_template",
                "template content",
                &header,
                &target_context,
            )
            .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_conflict_detection() {
        let detector = ConflictDetector::new();
        let target_context = create_test_context();

        let mut templates = HashMap::new();

        // Two templates with same pattern but different priorities
        let header1 = TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: Some(200),
            scope: None,
        };
        let header2 = TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: Some(100),
            scope: None,
        };

        templates.insert(
            "high_priority".to_string(),
            ("template 1".to_string(), header1),
        );
        templates.insert(
            "low_priority".to_string(),
            ("template 2".to_string(), header2),
        );

        let conflicts = detector
            .detect_conflicts(&templates, &target_context)
            .unwrap();

        assert_eq!(conflicts.len(), 1);
        let conflict = &conflicts[0];
        assert_eq!(conflict.winning_template, "high_priority");
        assert_eq!(conflict.conflicting_templates, vec!["low_priority"]);
        assert!(conflict.resolution_reason.contains("Higher priority"));
    }

    #[tokio::test]
    async fn test_template_composer_creation() {
        let composer = TemplateComposer::new();
        // Basic creation test - ensure composer is properly initialized
        assert_eq!(composer.resolver.default_priority, 100);
    }

    #[tokio::test]
    async fn test_template_composition_basic() {
        let composer = TemplateComposer::new();
        let target_context = create_test_context();

        let mut templates = HashMap::new();
        let header = TemplateHeader {
            raw: "template-match: interface.eth0".to_string(),
            pattern: MatchPattern::Exact("interface.eth0".to_string()),
            priority: Some(100),
            scope: None,
        };

        templates.insert(
            "test_template".to_string(),
            ("interface config".to_string(), header),
        );

        let result = composer
            .compose(&templates, &target_context, "interface.eth0")
            .await
            .unwrap();

        assert_eq!(result.selected_templates.len(), 1);
        assert_eq!(result.selected_templates[0].id, "test_template");
        assert_eq!(result.conflicts.len(), 0);
        assert_eq!(result.config_path, "interface.eth0");
    }

    #[tokio::test]
    async fn test_template_composition_with_conflicts() {
        let composer = TemplateComposer::new();
        let target_context = create_test_context();

        let mut templates = HashMap::new();

        // Two templates with same exact pattern but different priorities
        let header1 = TemplateHeader {
            raw: "template-match: interface.eth0".to_string(),
            pattern: MatchPattern::Exact("interface.eth0".to_string()),
            priority: Some(200),
            scope: None,
        };
        let header2 = TemplateHeader {
            raw: "template-match: interface.eth0".to_string(),
            pattern: MatchPattern::Exact("interface.eth0".to_string()),
            priority: Some(100),
            scope: None,
        };

        templates.insert(
            "high_priority".to_string(),
            ("high priority config".to_string(), header1),
        );
        templates.insert(
            "low_priority".to_string(),
            ("low priority config".to_string(), header2),
        );

        let result = composer
            .compose(&templates, &target_context, "interface.eth0")
            .await
            .unwrap();

        // Should select only the high priority template
        assert_eq!(result.selected_templates.len(), 1);
        assert_eq!(result.selected_templates[0].id, "high_priority");
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].winning_template, "high_priority");
        assert_eq!(
            result.conflicts[0].conflicting_templates,
            vec!["low_priority"]
        );
    }

    #[tokio::test]
    async fn test_template_composition_no_match() {
        let composer = TemplateComposer::new();
        let target_context = create_test_context();

        let mut templates = HashMap::new();
        let header = TemplateHeader {
            raw: "template-match: interface.eth1".to_string(),
            pattern: MatchPattern::Exact("interface.eth1".to_string()),
            priority: Some(100),
            scope: None,
        };

        templates.insert(
            "test_template".to_string(),
            ("interface config".to_string(), header),
        );

        let result = composer
            .compose(&templates, &target_context, "interface.eth0")
            .await
            .unwrap();

        // No templates should match
        assert_eq!(result.selected_templates.len(), 0);
        assert_eq!(result.conflicts.len(), 0);
    }

    #[tokio::test]
    async fn test_template_composition_scope_filtering() {
        let composer = TemplateComposer::new();
        let target_context = create_test_context(); // cisco router

        let mut templates = HashMap::new();

        // Template that matches scope
        let cisco_scope = TemplateScope {
            device_types: Some(vec!["router".to_string()]),
            vendors: Some(vec!["cisco".to_string()]),
            contexts: None,
        };
        let header1 = TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: Some(100),
            scope: Some(cisco_scope),
        };

        // Template that doesn't match vendor
        let juniper_scope = TemplateScope {
            device_types: Some(vec!["router".to_string()]),
            vendors: Some(vec!["juniper".to_string()]),
            contexts: None,
        };
        let header2 = TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: Some(100),
            scope: Some(juniper_scope),
        };

        templates.insert(
            "cisco_template".to_string(),
            ("cisco config".to_string(), header1),
        );
        templates.insert(
            "juniper_template".to_string(),
            ("juniper config".to_string(), header2),
        );

        let result = composer
            .compose(&templates, &target_context, "interface.eth0")
            .await
            .unwrap();

        // Only cisco template should be selected
        assert_eq!(result.selected_templates.len(), 1);
        assert_eq!(result.selected_templates[0].id, "cisco_template");
        assert_eq!(result.conflicts.len(), 0);
    }

    #[tokio::test]
    async fn test_template_compose_multiple() {
        let composer = TemplateComposer::new();
        let target_context = create_test_context();

        let mut templates = HashMap::new();
        let header1 = TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: Some(100),
            scope: None,
        };
        let header2 = TemplateHeader {
            raw: "template-match: routing.*".to_string(),
            pattern: MatchPattern::Glob("routing.*".to_string()),
            priority: Some(100),
            scope: None,
        };

        templates.insert(
            "interface_template".to_string(),
            ("interface config".to_string(), header1),
        );
        templates.insert(
            "routing_template".to_string(),
            ("routing config".to_string(), header2),
        );

        let config_paths = vec!["interface.eth0".to_string(), "routing.ospf".to_string()];
        let results = composer
            .compose_multiple(&templates, &target_context, &config_paths)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.contains_key("interface.eth0"));
        assert!(results.contains_key("routing.ospf"));

        let interface_result = &results["interface.eth0"];
        let routing_result = &results["routing.ospf"];

        assert_eq!(interface_result.selected_templates.len(), 1);
        assert_eq!(
            interface_result.selected_templates[0].id,
            "interface_template"
        );

        assert_eq!(routing_result.selected_templates.len(), 1);
        assert_eq!(routing_result.selected_templates[0].id, "routing_template");
    }

    #[tokio::test]
    async fn test_composition_summary() {
        let composer = TemplateComposer::new();
        let target_context = create_test_context();

        let mut templates = HashMap::new();
        let header1 = TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: Some(200),
            scope: None,
        };
        let header2 = TemplateHeader {
            raw: "template-match: interface.*".to_string(),
            pattern: MatchPattern::Glob("interface.*".to_string()),
            priority: Some(100),
            scope: None,
        };

        templates.insert(
            "high_priority".to_string(),
            ("high priority config".to_string(), header1),
        );
        templates.insert(
            "low_priority".to_string(),
            ("low priority config".to_string(), header2),
        );

        let config_paths = vec!["interface.eth0".to_string()];
        let summary = composer
            .get_composition_summary(&templates, &target_context, &config_paths)
            .await
            .unwrap();

        assert_eq!(summary.total_templates_available, 2);
        assert_eq!(summary.total_templates_selected, 1);
        assert_eq!(summary.total_conflicts_detected, 1);
        assert_eq!(summary.conflict_patterns.len(), 1);
        assert!(summary.conflict_patterns[0].contains("glob:interface.*"));
    }
}
