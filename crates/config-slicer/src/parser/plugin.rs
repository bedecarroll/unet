//! Parser plugin architecture
//!
//! This module provides a plugin system for extending the configuration parser
//! with new vendor-specific parsers and custom parsing logic.

use super::core::{ConfigNode, ParserConfig};
use super::vendors::{Vendor, VendorValidationReport};
use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashMap;
use tracing::info;

/// Trait for implementing custom configuration parsers
pub trait ConfigParserPlugin: Send + Sync {
    /// Get the name of this parser plugin
    fn name(&self) -> &str;

    /// Get the vendor type supported by this plugin
    fn vendor(&self) -> Vendor;

    /// Parse configuration text using this plugin
    fn parse(&self, config_text: &str, config: &ParserConfig) -> Result<ConfigNode>;

    /// Validate parsed configuration using vendor-specific rules
    fn validate(&self, tree: &ConfigNode) -> Result<VendorValidationReport>;

    /// Get supported file extensions for auto-detection
    fn supported_extensions(&self) -> Vec<&str> {
        vec![]
    }

    /// Detect if this parser can handle the given configuration
    fn can_parse(&self, _config_text: &str) -> bool {
        false
    }

    /// Get parser priority (higher numbers have higher priority)
    fn priority(&self) -> u32 {
        100
    }
}

/// Plugin registry for managing configuration parser plugins
#[derive(Default)]
pub struct PluginRegistry {
    /// Registered plugins by name
    plugins: HashMap<String, Box<dyn ConfigParserPlugin>>,
    /// Plugin detection patterns
    detection_patterns: Vec<DetectionPattern>,
}

/// Pattern for auto-detecting appropriate parser
#[derive(Debug, Clone)]
struct DetectionPattern {
    /// Name of the plugin to use
    plugin_name: String,
    /// Regex patterns to match against configuration
    patterns: Vec<Regex>,
    /// Minimum number of pattern matches required
    min_matches: usize,
}

impl PluginRegistry {
    /// Create a new plugin registry
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a parser plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn ConfigParserPlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        info!("Registering parser plugin: {}", name);

        // Add detection patterns for this plugin
        self.add_detection_patterns(&plugin)?;

        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Get a registered plugin by name
    #[must_use]
    pub fn get_plugin(&self, name: &str) -> Option<&dyn ConfigParserPlugin> {
        self.plugins.get(name).map(std::convert::AsRef::as_ref)
    }

    /// List all registered plugins
    #[must_use]
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Auto-detect the best parser for given configuration
    #[must_use]
    pub fn detect_parser(&self, config_text: &str) -> Option<&str> {
        let mut candidates: Vec<(&str, u32, usize)> = Vec::new();

        // Check each plugin's can_parse method
        for (name, plugin) in &self.plugins {
            if plugin.can_parse(config_text) {
                candidates.push((name, plugin.priority(), 0));
            }
        }

        // Check detection patterns
        for pattern in &self.detection_patterns {
            let matches = pattern
                .patterns
                .iter()
                .filter(|p| p.is_match(config_text))
                .count();

            if matches >= pattern.min_matches {
                if let Some(plugin) = self.plugins.get(&pattern.plugin_name) {
                    candidates.push((&pattern.plugin_name, plugin.priority(), matches));
                }
            }
        }

        // Sort by priority (highest first), then by pattern matches
        candidates.sort_by(|a, b| {
            b.1.cmp(&a.1) // Priority (descending)
                .then(b.2.cmp(&a.2)) // Pattern matches (descending)
        });

        candidates.first().map(|(name, _, _)| *name)
    }

    /// Parse configuration using auto-detected or specified parser
    pub fn parse_with_detection(
        &self,
        config_text: &str,
        config: &ParserConfig,
        force_parser: Option<&str>,
    ) -> Result<(ConfigNode, String)> {
        let parser_name = if let Some(forced) = force_parser {
            if !self.plugins.contains_key(forced) {
                return Err(anyhow!("Parser plugin '{}' not found", forced));
            }
            forced.to_string()
        } else {
            self.detect_parser(config_text)
                .ok_or_else(|| anyhow!("Could not auto-detect appropriate parser"))?
                .to_string()
        };

        let plugin = self
            .get_plugin(&parser_name)
            .ok_or_else(|| anyhow!("Parser plugin '{}' not found", parser_name))?;

        info!("Using parser plugin: {}", parser_name);
        let tree = plugin.parse(config_text, config)?;
        Ok((tree, parser_name))
    }

    /// Add detection patterns for a plugin
    fn add_detection_patterns(&mut self, plugin: &Box<dyn ConfigParserPlugin>) -> Result<()> {
        let name = plugin.name();
        let patterns = match plugin.vendor() {
            Vendor::Cisco => vec![
                Regex::new(r"^\s*!")?,           // Cisco comments
                Regex::new(r"^\s*hostname\s+")?, // Cisco hostname
                Regex::new(r"^\s*interface\s+(GigabitEthernet|FastEthernet|TenGigabitEthernet)")?,
                Regex::new(r"^\s*router\s+(ospf|eigrp|bgp)")?,
                Regex::new(r"^\s*ip\s+route\s+")?,
            ],
            Vendor::Juniper => vec![
                Regex::new(r"^\s*#")?,               // Juniper comments
                Regex::new(r"^\s*set\s+system\s+")?, // Juniper set commands
                Regex::new(r"^\s*interfaces\s+\{")?, // Juniper interface blocks
                Regex::new(r"^\s*protocols\s+\{")?,  // Juniper protocol blocks
                Regex::new(r"^\s*.*\s*\{")?,         // Juniper brace syntax
            ],
            Vendor::Arista => vec![
                Regex::new(r"^\s*!")?,           // Arista comments (like Cisco)
                Regex::new(r"^\s*hostname\s+")?, // Arista hostname
                Regex::new(r"^\s*interface\s+(Ethernet|Management)")?,
                Regex::new(r"^\s*daemon\s+")?, // Arista-specific daemon config
            ],
            Vendor::Generic => vec![
                Regex::new(r"^\s*\[\w+\]")?, // INI-style sections
                Regex::new(r"^\s*\w+\s*=")?, // Key-value pairs
            ],
        };

        self.detection_patterns.push(DetectionPattern {
            plugin_name: name.to_string(),
            patterns,
            min_matches: 2, // Require at least 2 pattern matches
        });

        Ok(())
    }
}

/// Builder for creating plugin registry with default plugins
pub struct PluginRegistryBuilder {
    registry: PluginRegistry,
}

impl PluginRegistryBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
        }
    }

    /// Add all default vendor plugins
    pub fn with_default_plugins(mut self) -> Result<Self> {
        self = self.with_cisco_plugin()?;
        self = self.with_juniper_plugin()?;
        self = self.with_arista_plugin()?;
        self = self.with_generic_plugin()?;
        Ok(self)
    }

    /// Add Cisco parser plugin
    pub fn with_cisco_plugin(mut self) -> Result<Self> {
        self.registry
            .register_plugin(Box::new(CiscoParserPlugin::new()?))?;
        Ok(self)
    }

    /// Add Juniper parser plugin
    pub fn with_juniper_plugin(mut self) -> Result<Self> {
        self.registry
            .register_plugin(Box::new(JuniperParserPlugin::new()?))?;
        Ok(self)
    }

    /// Add Arista parser plugin
    pub fn with_arista_plugin(mut self) -> Result<Self> {
        self.registry
            .register_plugin(Box::new(AristaParserPlugin::new()?))?;
        Ok(self)
    }

    /// Add generic parser plugin
    pub fn with_generic_plugin(mut self) -> Result<Self> {
        self.registry
            .register_plugin(Box::new(GenericParserPlugin::new()?))?;
        Ok(self)
    }

    /// Add a custom plugin
    pub fn with_plugin(mut self, plugin: Box<dyn ConfigParserPlugin>) -> Result<Self> {
        self.registry.register_plugin(plugin)?;
        Ok(self)
    }

    /// Build the plugin registry
    #[must_use]
    pub fn build(self) -> PluginRegistry {
        self.registry
    }
}

impl Default for PluginRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Default plugin implementations
struct CiscoParserPlugin;
struct JuniperParserPlugin;
struct AristaParserPlugin;
struct GenericParserPlugin;

impl CiscoParserPlugin {
    const fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl ConfigParserPlugin for CiscoParserPlugin {
    fn name(&self) -> &'static str {
        "cisco"
    }

    fn vendor(&self) -> Vendor {
        Vendor::Cisco
    }

    fn parse(&self, config_text: &str, config: &ParserConfig) -> Result<ConfigNode> {
        use super::vendors::VendorParser;
        let parser = VendorParser::with_config(Vendor::Cisco, config.clone())?;
        parser.parse(config_text)
    }

    fn validate(&self, tree: &ConfigNode) -> Result<VendorValidationReport> {
        use super::vendors::VendorParser;
        let parser = VendorParser::new(Vendor::Cisco)?;
        parser.validate_vendor_syntax(tree)
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["ios", "cfg", "conf"]
    }

    fn can_parse(&self, config_text: &str) -> bool {
        // Simple heuristic: look for Cisco-specific patterns
        config_text.contains("interface GigabitEthernet")
            || config_text.contains("interface FastEthernet")
            || (config_text.contains("hostname") && config_text.contains('!'))
    }

    fn priority(&self) -> u32 {
        200
    }
}

impl JuniperParserPlugin {
    const fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl ConfigParserPlugin for JuniperParserPlugin {
    fn name(&self) -> &'static str {
        "juniper"
    }

    fn vendor(&self) -> Vendor {
        Vendor::Juniper
    }

    fn parse(&self, config_text: &str, config: &ParserConfig) -> Result<ConfigNode> {
        use super::vendors::VendorParser;
        let parser = VendorParser::with_config(Vendor::Juniper, config.clone())?;
        parser.parse(config_text)
    }

    fn validate(&self, tree: &ConfigNode) -> Result<VendorValidationReport> {
        use super::vendors::VendorParser;
        let parser = VendorParser::new(Vendor::Juniper)?;
        parser.validate_vendor_syntax(tree)
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["junos", "conf"]
    }

    fn can_parse(&self, config_text: &str) -> bool {
        // Look for Juniper-specific patterns
        config_text.contains("set system") || config_text.contains("interfaces {")
    }

    fn priority(&self) -> u32 {
        200
    }
}

impl AristaParserPlugin {
    const fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl ConfigParserPlugin for AristaParserPlugin {
    fn name(&self) -> &'static str {
        "arista"
    }

    fn vendor(&self) -> Vendor {
        Vendor::Arista
    }

    fn parse(&self, config_text: &str, config: &ParserConfig) -> Result<ConfigNode> {
        use super::vendors::VendorParser;
        let parser = VendorParser::with_config(Vendor::Arista, config.clone())?;
        parser.parse(config_text)
    }

    fn validate(&self, tree: &ConfigNode) -> Result<VendorValidationReport> {
        use super::vendors::VendorParser;
        let parser = VendorParser::new(Vendor::Arista)?;
        parser.validate_vendor_syntax(tree)
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["eos", "cfg", "conf"]
    }

    fn can_parse(&self, config_text: &str) -> bool {
        // Look for Arista-specific patterns
        config_text.contains("interface Ethernet") || config_text.contains("daemon ")
    }

    fn priority(&self) -> u32 {
        200
    }
}

impl GenericParserPlugin {
    const fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl ConfigParserPlugin for GenericParserPlugin {
    fn name(&self) -> &'static str {
        "generic"
    }

    fn vendor(&self) -> Vendor {
        Vendor::Generic
    }

    fn parse(&self, config_text: &str, config: &ParserConfig) -> Result<ConfigNode> {
        use super::vendors::VendorParser;
        let parser = VendorParser::with_config(Vendor::Generic, config.clone())?;
        parser.parse(config_text)
    }

    fn validate(&self, tree: &ConfigNode) -> Result<VendorValidationReport> {
        use super::vendors::VendorParser;
        let parser = VendorParser::new(Vendor::Generic)?;
        parser.validate_vendor_syntax(tree)
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["txt", "cfg", "conf", "ini"]
    }

    fn can_parse(&self, _config_text: &str) -> bool {
        // Generic parser can handle anything as a fallback
        true
    }

    fn priority(&self) -> u32 {
        50 // Lower priority, used as fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::NodeType;

    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.list_plugins().len(), 0);
    }

    #[test]
    fn test_plugin_registration() -> Result<()> {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(CiscoParserPlugin::new()?);

        registry.register_plugin(plugin)?;
        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.get_plugin("cisco").is_some());

        Ok(())
    }

    #[test]
    fn test_default_plugins_builder() -> Result<()> {
        let registry = PluginRegistryBuilder::new().with_default_plugins()?.build();

        let plugins = registry.list_plugins();
        assert!(plugins.contains(&"cisco"));
        assert!(plugins.contains(&"juniper"));
        assert!(plugins.contains(&"arista"));
        assert!(plugins.contains(&"generic"));

        Ok(())
    }

    #[test]
    fn test_cisco_detection() -> Result<()> {
        let registry = PluginRegistryBuilder::new().with_default_plugins()?.build();

        let cisco_config = r"
hostname test-router
!
interface GigabitEthernet0/1
 description Uplink
ip route 0.0.0.0 0.0.0.0 192.168.1.1
";

        let detected = registry.detect_parser(cisco_config);
        assert_eq!(detected, Some("cisco"));

        Ok(())
    }

    #[test]
    fn test_juniper_detection() -> Result<()> {
        let registry = PluginRegistryBuilder::new().with_default_plugins()?.build();

        let juniper_config = r#"
# Juniper configuration
set system host-name test-router
interfaces {
    ge-0/0/1 {
        description "Uplink";
    }
}
"#;

        let detected = registry.detect_parser(juniper_config);
        assert_eq!(detected, Some("juniper"));

        Ok(())
    }

    #[test]
    fn test_parse_with_detection() -> Result<()> {
        let registry = PluginRegistryBuilder::new().with_default_plugins()?.build();

        let cisco_config = r"
hostname test-router
interface GigabitEthernet0/1
 description Test
";

        let config = ParserConfig::default();
        let (tree, parser_name) = registry.parse_with_detection(cisco_config, &config, None)?;

        assert_eq!(parser_name, "cisco");
        assert_eq!(tree.node_type, NodeType::Root);

        Ok(())
    }

    #[test]
    fn test_forced_parser() -> Result<()> {
        let registry = PluginRegistryBuilder::new().with_default_plugins()?.build();

        let config_text = "hostname test";
        let config = ParserConfig::default();

        let (_, parser_name) =
            registry.parse_with_detection(config_text, &config, Some("generic"))?;

        assert_eq!(parser_name, "generic");

        Ok(())
    }
}
