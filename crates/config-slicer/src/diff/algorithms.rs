//! Diff algorithm implementations
//!
//! This module provides different types of diff algorithms for comparing configurations.

use crate::diff::types::*;
use crate::parser::ConfigNode;
use anyhow::{Context, Result};
use regex::Regex;
use similar::{ChangeTag, TextDiff as SimilarTextDiff};
use std::collections::HashMap;
use tracing::debug;

/// Text-based differ using line-by-line comparison
pub struct TextDiffer {
    options: DiffOptions,
}

/// Hierarchical differ comparing configuration tree structures
pub struct HierarchicalDiffer {
    options: DiffOptions,
}

/// Semantic differ with configuration understanding
pub struct SemanticDiffer {
    options: DiffOptions,
    // Regex patterns for semantic understanding
    ip_pattern: Regex,
    vlan_pattern: Regex,
    interface_pattern: Regex,
}

impl TextDiffer {
    /// Create a new text differ with the given options
    pub fn new(options: DiffOptions) -> Self {
        Self { options }
    }

    /// Compare two text configurations and generate a text diff
    pub fn diff(&self, old_config: &str, new_config: &str) -> Result<TextDiff> {
        debug!("Starting text diff comparison");

        let diff = SimilarTextDiff::from_lines(old_config, new_config);
        let mut changes = Vec::new();
        let mut additions = 0;
        let mut deletions = 0;
        let mut modifications = 0;

        let mut old_line_num = 1;
        let mut new_line_num = 1;

        for change in diff.iter_all_changes() {
            let change_type = match change.tag() {
                ChangeTag::Delete => {
                    deletions += 1;
                    DiffType::Deletion
                }
                ChangeTag::Insert => {
                    additions += 1;
                    DiffType::Addition
                }
                ChangeTag::Equal => DiffType::Unchanged,
            };

            let diff_change = match change.tag() {
                ChangeTag::Delete => DiffChange {
                    change_type,
                    old_line: Some(change.value().trim_end().to_string()),
                    new_line: None,
                    old_line_number: Some(old_line_num),
                    new_line_number: None,
                    context: self.extract_context(change.value()),
                },
                ChangeTag::Insert => DiffChange {
                    change_type,
                    old_line: None,
                    new_line: Some(change.value().trim_end().to_string()),
                    old_line_number: None,
                    new_line_number: Some(new_line_num),
                    context: self.extract_context(change.value()),
                },
                ChangeTag::Equal => DiffChange {
                    change_type,
                    old_line: Some(change.value().trim_end().to_string()),
                    new_line: Some(change.value().trim_end().to_string()),
                    old_line_number: Some(old_line_num),
                    new_line_number: Some(new_line_num),
                    context: self.extract_context(change.value()),
                },
            };

            changes.push(diff_change);

            match change.tag() {
                ChangeTag::Delete => old_line_num += 1,
                ChangeTag::Insert => new_line_num += 1,
                ChangeTag::Equal => {
                    old_line_num += 1;
                    new_line_num += 1;
                }
            }
        }

        // Apply context line filtering
        let filtered_changes = self.apply_context_filtering(changes);

        // Detect modifications (delete + insert pairs)
        let (final_changes, actual_modifications) = self.detect_modifications(filtered_changes);
        modifications = actual_modifications;

        Ok(TextDiff {
            changes: final_changes,
            additions,
            deletions,
            modifications,
            context_lines: self.options.context_lines,
        })
    }

    fn prepare_lines<'a>(&self, config: &'a str) -> Vec<&'a str> {
        let lines: Vec<&str> = config.lines().collect();

        if self.options.ignore_whitespace {
            // For whitespace ignoring, we need to normalize the lines
            // This is a simplified approach - in practice, you might want to
            // store normalized versions separately
        }

        lines
    }

    fn extract_context(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Try to identify the configuration context
        if trimmed.starts_with("interface ") {
            Some("interface".to_string())
        } else if trimmed.starts_with("vlan ") {
            Some("vlan".to_string())
        } else if trimmed.starts_with("router ") {
            Some("routing".to_string())
        } else if trimmed.starts_with("access-list ") {
            Some("access-list".to_string())
        } else {
            None
        }
    }

    fn apply_context_filtering(&self, changes: Vec<DiffChange>) -> Vec<DiffChange> {
        if self.options.context_lines == 0 {
            // Return only non-unchanged lines
            return changes
                .into_iter()
                .filter(|c| c.change_type != DiffType::Unchanged)
                .collect();
        }

        let mut filtered = Vec::new();
        let context_size = self.options.context_lines;

        // Find all change indices (non-unchanged)
        let change_indices: Vec<usize> = changes
            .iter()
            .enumerate()
            .filter(|(_, c)| c.change_type != DiffType::Unchanged)
            .map(|(i, _)| i)
            .collect();

        if change_indices.is_empty() {
            return filtered;
        }

        let mut included_indices = std::collections::HashSet::new();

        // For each change, include context around it
        for &change_idx in &change_indices {
            let start = change_idx.saturating_sub(context_size);
            let end = std::cmp::min(change_idx + context_size + 1, changes.len());

            for i in start..end {
                included_indices.insert(i);
            }
        }

        // Collect the filtered changes in order
        for (i, change) in changes.into_iter().enumerate() {
            if included_indices.contains(&i) {
                filtered.push(change);
            }
        }

        filtered
    }

    fn detect_modifications(&self, changes: Vec<DiffChange>) -> (Vec<DiffChange>, usize) {
        let mut result = Vec::new();
        let mut modifications = 0;
        let mut i = 0;

        while i < changes.len() {
            let current = &changes[i];

            // Look for delete followed by insert pattern
            if current.change_type == DiffType::Deletion && i + 1 < changes.len() {
                let next = &changes[i + 1];
                if next.change_type == DiffType::Addition {
                    // This is a modification
                    let modification = DiffChange {
                        change_type: DiffType::Modification,
                        old_line: current.old_line.clone(),
                        new_line: next.new_line.clone(),
                        old_line_number: current.old_line_number,
                        new_line_number: next.new_line_number,
                        context: current.context.clone().or_else(|| next.context.clone()),
                    };
                    result.push(modification);
                    modifications += 1;
                    i += 2; // Skip both the delete and insert
                    continue;
                }
            }

            result.push(current.clone());
            i += 1;
        }

        (result, modifications)
    }
}

impl HierarchicalDiffer {
    /// Create a new hierarchical differ with the given options
    pub fn new(options: DiffOptions) -> Self {
        Self { options }
    }

    /// Compare two configuration trees and generate a hierarchical diff
    pub fn diff(&self, old_tree: &ConfigNode, new_tree: &ConfigNode) -> Result<HierarchicalDiff> {
        debug!("Starting hierarchical diff comparison");

        let mut sections = Vec::new();
        let mut structure_changes = Vec::new();
        let mut path_changes = HashMap::new();

        self.compare_nodes(
            old_tree,
            new_tree,
            "",
            &mut sections,
            &mut structure_changes,
            &mut path_changes,
        )?;

        Ok(HierarchicalDiff {
            sections,
            structure_changes,
            path_changes,
        })
    }

    fn compare_nodes(
        &self,
        old_node: &ConfigNode,
        new_node: &ConfigNode,
        path: &str,
        sections: &mut Vec<DiffSection>,
        structure_changes: &mut Vec<StructureChange>,
        path_changes: &mut HashMap<String, DiffType>,
    ) -> Result<()> {
        let current_path = if path.is_empty() {
            old_node.command.clone()
        } else {
            format!("{}.{}", path, old_node.command)
        };

        // Compare the nodes themselves
        if old_node.command != new_node.command {
            let section = DiffSection {
                path: current_path.clone(),
                change_type: DiffType::Modification,
                old_section: Some(old_node.clone()),
                new_section: Some(new_node.clone()),
                child_changes: Vec::new(),
            };
            sections.push(section);
            path_changes.insert(current_path.clone(), DiffType::Modification);
        }

        // Compare children using a more sophisticated algorithm
        self.compare_children(
            old_node,
            new_node,
            &current_path,
            sections,
            structure_changes,
            path_changes,
        )?;

        Ok(())
    }

    fn compare_children(
        &self,
        old_node: &ConfigNode,
        new_node: &ConfigNode,
        path: &str,
        sections: &mut Vec<DiffSection>,
        structure_changes: &mut Vec<StructureChange>,
        path_changes: &mut HashMap<String, DiffType>,
    ) -> Result<()> {
        // Create maps for easier lookup
        let old_children: HashMap<String, &ConfigNode> = old_node
            .children
            .iter()
            .map(|child| (child.command.clone(), child))
            .collect();

        let new_children: HashMap<String, &ConfigNode> = new_node
            .children
            .iter()
            .map(|child| (child.command.clone(), child))
            .collect();

        // Find added children
        for (command, new_child) in &new_children {
            if !old_children.contains_key(command) {
                let child_path = format!("{}.{}", path, command);
                let section = DiffSection {
                    path: child_path.clone(),
                    change_type: DiffType::Addition,
                    old_section: None,
                    new_section: Some((*new_child).clone()),
                    child_changes: Vec::new(),
                };
                sections.push(section);
                path_changes.insert(child_path, DiffType::Addition);
            }
        }

        // Find removed children
        for (command, old_child) in &old_children {
            if !new_children.contains_key(command) {
                let child_path = format!("{}.{}", path, command);
                let section = DiffSection {
                    path: child_path.clone(),
                    change_type: DiffType::Deletion,
                    old_section: Some((*old_child).clone()),
                    new_section: None,
                    child_changes: Vec::new(),
                };
                sections.push(section);
                path_changes.insert(child_path, DiffType::Deletion);
            }
        }

        // Compare existing children recursively
        for (command, old_child) in &old_children {
            if let Some(new_child) = new_children.get(command) {
                self.compare_nodes(
                    old_child,
                    new_child,
                    path,
                    sections,
                    structure_changes,
                    path_changes,
                )?;
            }
        }

        Ok(())
    }
}

impl SemanticDiffer {
    /// Create a new semantic differ with the given options
    pub fn new(options: DiffOptions) -> Result<Self> {
        let ip_pattern = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}(?:/\d{1,2})?\b")
            .context("Failed to compile IP address regex")?;
        let vlan_pattern =
            Regex::new(r"\bvlan\s+(\d+)\b").context("Failed to compile VLAN regex")?;
        let interface_pattern = Regex::new(r"\binterface\s+([\w\d\./\-]+)")
            .context("Failed to compile interface regex")?;

        Ok(Self {
            options,
            ip_pattern,
            vlan_pattern,
            interface_pattern,
        })
    }

    /// Generate semantic diff by analyzing configuration meaning
    pub fn diff(
        &self,
        _old_config: &str,
        _new_config: &str,
        text_diff: &TextDiff,
    ) -> Result<SemanticDiff> {
        debug!("Starting semantic diff analysis");

        let mut functional_changes = Vec::new();
        let mut impact_analysis = Vec::new();
        let mut change_groups = Vec::new();

        // Analyze each change for semantic meaning
        for change in &text_diff.changes {
            if change.change_type == DiffType::Unchanged {
                continue;
            }

            let semantic_changes = self.analyze_change_semantics(change)?;
            functional_changes.extend(semantic_changes);
        }

        // Group related changes
        if self.options.group_changes {
            change_groups = self.group_related_changes(&functional_changes);
        }

        // Analyze impact
        if self.options.include_impact {
            impact_analysis = self.analyze_impact(&functional_changes)?;
        }

        Ok(SemanticDiff {
            functional_changes,
            impact_analysis,
            change_groups,
        })
    }

    fn analyze_change_semantics(&self, change: &DiffChange) -> Result<Vec<FunctionalChange>> {
        let mut changes = Vec::new();

        // Analyze old and new lines
        let old_text = change.old_line.as_deref().unwrap_or("");
        let new_text = change.new_line.as_deref().unwrap_or("");

        // Check for IP address changes
        if let Some(ip_change) = self.detect_ip_changes(old_text, new_text, change)? {
            changes.push(ip_change);
        }

        // Check for VLAN changes
        if let Some(vlan_change) = self.detect_vlan_changes(old_text, new_text, change)? {
            changes.push(vlan_change);
        }

        // Check for interface changes
        if let Some(interface_change) = self.detect_interface_changes(old_text, new_text, change)? {
            changes.push(interface_change);
        }

        Ok(changes)
    }

    fn detect_ip_changes(
        &self,
        old_text: &str,
        new_text: &str,
        change: &DiffChange,
    ) -> Result<Option<FunctionalChange>> {
        let old_ips: Vec<&str> = self
            .ip_pattern
            .find_iter(old_text)
            .map(|m| m.as_str())
            .collect();
        let new_ips: Vec<&str> = self
            .ip_pattern
            .find_iter(new_text)
            .map(|m| m.as_str())
            .collect();

        if old_ips != new_ips {
            let path = format!(
                "line_{}",
                change
                    .old_line_number
                    .or(change.new_line_number)
                    .unwrap_or(0)
            );
            return Ok(Some(FunctionalChange {
                change_type: FunctionalChangeType::IpAddressing,
                description: "IP address configuration changed".to_string(),
                path,
                old_value: if old_ips.is_empty() {
                    None
                } else {
                    Some(old_ips.join(", "))
                },
                new_value: if new_ips.is_empty() {
                    None
                } else {
                    Some(new_ips.join(", "))
                },
                severity: ChangeSeverity::Warning,
            }));
        }

        Ok(None)
    }

    fn detect_vlan_changes(
        &self,
        old_text: &str,
        new_text: &str,
        change: &DiffChange,
    ) -> Result<Option<FunctionalChange>> {
        let old_vlans: Vec<String> = self
            .vlan_pattern
            .captures_iter(old_text)
            .map(|cap| cap[1].to_string())
            .collect();
        let new_vlans: Vec<String> = self
            .vlan_pattern
            .captures_iter(new_text)
            .map(|cap| cap[1].to_string())
            .collect();

        if old_vlans != new_vlans {
            let path = format!(
                "line_{}",
                change
                    .old_line_number
                    .or(change.new_line_number)
                    .unwrap_or(0)
            );
            return Ok(Some(FunctionalChange {
                change_type: FunctionalChangeType::VlanConfig,
                description: "VLAN configuration changed".to_string(),
                path,
                old_value: if old_vlans.is_empty() {
                    None
                } else {
                    Some(old_vlans.join(", "))
                },
                new_value: if new_vlans.is_empty() {
                    None
                } else {
                    Some(new_vlans.join(", "))
                },
                severity: ChangeSeverity::Info,
            }));
        }

        Ok(None)
    }

    fn detect_interface_changes(
        &self,
        old_text: &str,
        new_text: &str,
        change: &DiffChange,
    ) -> Result<Option<FunctionalChange>> {
        let old_interfaces: Vec<String> = self
            .interface_pattern
            .captures_iter(old_text)
            .map(|cap| cap[1].to_string())
            .collect();
        let new_interfaces: Vec<String> = self
            .interface_pattern
            .captures_iter(new_text)
            .map(|cap| cap[1].to_string())
            .collect();

        if old_interfaces != new_interfaces {
            let path = format!(
                "line_{}",
                change
                    .old_line_number
                    .or(change.new_line_number)
                    .unwrap_or(0)
            );
            return Ok(Some(FunctionalChange {
                change_type: FunctionalChangeType::InterfaceState,
                description: "Interface configuration changed".to_string(),
                path,
                old_value: if old_interfaces.is_empty() {
                    None
                } else {
                    Some(old_interfaces.join(", "))
                },
                new_value: if new_interfaces.is_empty() {
                    None
                } else {
                    Some(new_interfaces.join(", "))
                },
                severity: ChangeSeverity::Warning,
            }));
        }

        Ok(None)
    }

    fn group_related_changes(&self, changes: &[FunctionalChange]) -> Vec<ChangeGroup> {
        let mut groups = HashMap::new();

        for change in changes {
            let group_type = match change.change_type {
                FunctionalChangeType::InterfaceState => ChangeGroupType::Interface,
                FunctionalChangeType::VlanConfig => ChangeGroupType::Vlan,
                FunctionalChangeType::Routing => ChangeGroupType::Routing,
                FunctionalChangeType::Security | FunctionalChangeType::AccessControl => {
                    ChangeGroupType::Security
                }
                _ => ChangeGroupType::Custom("other".to_string()),
            };

            let group_key = format!("{:?}", group_type);
            groups
                .entry(group_key.clone())
                .or_insert_with(|| ChangeGroup {
                    id: group_key.clone(),
                    description: format!("{:?} related changes", group_type),
                    changes: Vec::new(),
                    group_type,
                })
                .changes
                .push(change.path.clone());
        }

        groups.into_values().collect()
    }

    fn analyze_impact(&self, changes: &[FunctionalChange]) -> Result<Vec<ImpactAnalysis>> {
        let mut analyses = Vec::new();

        for change in changes {
            let impact = match change.change_type {
                FunctionalChangeType::IpAddressing => ImpactAnalysis {
                    affected_components: vec!["routing".to_string(), "connectivity".to_string()],
                    risk_level: RiskLevel::High,
                    impact_description: "IP address changes may affect routing and connectivity".to_string(),
                    validation_steps: vec![
                        "Verify routing table updates".to_string(),
                        "Test connectivity to affected networks".to_string(),
                        "Check for routing protocol convergence".to_string(),
                    ],
                },
                FunctionalChangeType::InterfaceState => ImpactAnalysis {
                    affected_components: vec!["connectivity".to_string(), "throughput".to_string()],
                    risk_level: RiskLevel::Medium,
                    impact_description: "Interface changes may affect link state and performance".to_string(),
                    validation_steps: vec![
                        "Verify interface operational state".to_string(),
                        "Monitor interface utilization".to_string(),
                        "Check for link flapping".to_string(),
                    ],
                },
                FunctionalChangeType::Security | FunctionalChangeType::AccessControl => ImpactAnalysis {
                    affected_components: vec!["security".to_string(), "access control".to_string()],
                    risk_level: RiskLevel::Critical,
                    impact_description: "Security changes may affect access control and network security posture".to_string(),
                    validation_steps: vec![
                        "Test access control rules".to_string(),
                        "Verify security policy enforcement".to_string(),
                        "Conduct security compliance check".to_string(),
                    ],
                },
                _ => ImpactAnalysis {
                    affected_components: vec!["general".to_string()],
                    risk_level: RiskLevel::Low,
                    impact_description: "General configuration change".to_string(),
                    validation_steps: vec!["Verify configuration applied correctly".to_string()],
                },
            };
            analyses.push(impact);
        }

        Ok(analyses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_differ_basic() {
        let options = DiffOptions::default();
        let differ = TextDiffer::new(options);

        let old_config = "interface GigabitEthernet0/1\n ip address 192.168.1.1 255.255.255.0";
        let new_config = "interface GigabitEthernet0/1\n ip address 192.168.1.2 255.255.255.0";

        let result = differ.diff(old_config, new_config).unwrap();
        assert!(!result.changes.is_empty());
    }

    #[test]
    fn test_semantic_differ_ip_detection() {
        let options = DiffOptions::default();
        let differ = SemanticDiffer::new(options).unwrap();

        let change = DiffChange {
            change_type: DiffType::Modification,
            old_line: Some("ip address 192.168.1.1 255.255.255.0".to_string()),
            new_line: Some("ip address 192.168.1.2 255.255.255.0".to_string()),
            old_line_number: Some(1),
            new_line_number: Some(1),
            context: None,
        };

        let semantic_changes = differ.analyze_change_semantics(&change).unwrap();
        assert!(!semantic_changes.is_empty());
        assert!(matches!(
            semantic_changes[0].change_type,
            FunctionalChangeType::IpAddressing
        ));
    }
}
