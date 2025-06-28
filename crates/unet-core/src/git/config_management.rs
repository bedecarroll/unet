//! Environment-specific configuration management
//!
//! This module provides functionality for managing environment-specific
//! configurations, including configuration layering, merging, and validation.

use crate::git::environment::{EnvironmentConfig, EnvironmentType};
use crate::git::types::{GitError, GitResult};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

/// Configuration layer priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfigPriority {
    /// Base configuration (lowest priority)
    Base = 0,
    /// Global environment defaults
    Global = 10,
    /// Environment type defaults (dev, staging, prod)
    EnvironmentType = 20,
    /// Environment-specific overrides
    Environment = 30,
    /// Local development overrides (highest priority)
    Local = 40,
}

impl ConfigPriority {
    /// Get all priority levels in order
    pub fn all_levels() -> Vec<ConfigPriority> {
        vec![
            ConfigPriority::Base,
            ConfigPriority::Global,
            ConfigPriority::EnvironmentType,
            ConfigPriority::Environment,
            ConfigPriority::Local,
        ]
    }
}

impl std::fmt::Display for ConfigPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigPriority::Base => write!(f, "base"),
            ConfigPriority::Global => write!(f, "global"),
            ConfigPriority::EnvironmentType => write!(f, "environment-type"),
            ConfigPriority::Environment => write!(f, "environment"),
            ConfigPriority::Local => write!(f, "local"),
        }
    }
}

/// Configuration layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLayer {
    /// Layer name
    pub name: String,
    /// Layer priority
    pub priority: ConfigPriority,
    /// Configuration values
    pub values: JsonValue,
    /// Layer source (file path, environment, etc.)
    pub source: String,
    /// Whether this layer is enabled
    pub enabled: bool,
    /// Layer metadata
    pub metadata: HashMap<String, JsonValue>,
}

impl ConfigLayer {
    /// Create a new configuration layer
    pub fn new(name: String, priority: ConfigPriority, values: JsonValue, source: String) -> Self {
        Self {
            name,
            priority,
            values,
            source,
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    /// Create a layer from a file
    pub fn from_file<P: AsRef<Path>>(
        name: String,
        priority: ConfigPriority,
        path: P,
    ) -> GitResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| GitError::RepositoryOperation {
            message: format!("Failed to read config file {}: {}", path.display(), e),
        })?;

        let values = if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            serde_yaml::from_str(&content).map_err(|e| GitError::RepositoryOperation {
                message: format!("Failed to parse YAML config {}: {}", path.display(), e),
            })?
        } else {
            serde_json::from_str(&content).map_err(|e| GitError::RepositoryOperation {
                message: format!("Failed to parse JSON config {}: {}", path.display(), e),
            })?
        };

        Ok(Self::new(
            name,
            priority,
            values,
            path.display().to_string(),
        ))
    }

    /// Check if this layer contains a specific key
    pub fn contains_key(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }

    /// Get a value from this layer
    pub fn get_value(&self, key: &str) -> Option<&JsonValue> {
        get_nested_value(&self.values, key)
    }

    /// Set a value in this layer
    pub fn set_value(&mut self, key: &str, value: JsonValue) -> GitResult<()> {
        set_nested_value(&mut self.values, key, value).map_err(|e| GitError::RepositoryOperation {
            message: format!("Failed to set config value '{}': {}", key, e),
        })
    }

    /// Remove a value from this layer
    pub fn remove_value(&mut self, key: &str) -> Option<JsonValue> {
        remove_nested_value(&mut self.values, key)
    }
}

/// Environment configuration manager
#[derive(Debug)]
pub struct EnvironmentConfigManager {
    /// Configuration layers by priority
    layers: Vec<ConfigLayer>,
    /// Current environment
    current_environment: Option<String>,
    /// Base configuration directory
    config_dir: PathBuf,
}

impl EnvironmentConfigManager {
    /// Create a new environment configuration manager
    pub fn new<P: AsRef<Path>>(config_dir: P) -> Self {
        Self {
            layers: Vec::new(),
            current_environment: None,
            config_dir: config_dir.as_ref().to_path_buf(),
        }
    }

    /// Load configuration for an environment
    pub fn load_environment_config(&mut self, env: &EnvironmentConfig) -> GitResult<()> {
        info!("Loading configuration for environment: {}", env.name);

        // Clear existing layers
        self.layers.clear();
        self.current_environment = Some(env.name.clone());

        // Load base configuration
        self.load_base_config()?;

        // Load global environment configuration
        self.load_global_config()?;

        // Load environment type configuration
        self.load_environment_type_config(env.environment_type)?;

        // Load environment-specific configuration
        self.load_environment_specific_config(env)?;

        // Load local overrides if they exist
        self.load_local_config()?;

        // Apply environment-specific overrides from the environment config
        self.apply_environment_overrides(env)?;

        // Sort layers by priority
        self.layers.sort_by_key(|layer| layer.priority);

        info!(
            "Loaded {} configuration layers for environment: {}",
            self.layers.len(),
            env.name
        );
        Ok(())
    }

    /// Get the final merged configuration
    pub fn get_merged_config(&self) -> JsonValue {
        let mut merged = JsonValue::Object(serde_json::Map::new());

        // Merge layers in priority order (lowest to highest)
        for layer in &self.layers {
            if layer.enabled {
                merge_json_values(&mut merged, &layer.values);
            }
        }

        merged
    }

    /// Get a specific configuration value
    pub fn get_config_value(&self, key: &str) -> Option<JsonValue> {
        // Check layers in reverse priority order (highest to lowest)
        for layer in self.layers.iter().rev() {
            if layer.enabled {
                if let Some(value) = layer.get_value(key) {
                    return Some(value.clone());
                }
            }
        }
        None
    }

    /// Set a configuration value in the environment layer
    pub fn set_config_value(&mut self, key: &str, value: JsonValue) -> GitResult<()> {
        // Find or create environment-specific layer
        if let Some(layer) = self
            .layers
            .iter_mut()
            .find(|l| l.priority == ConfigPriority::Environment)
        {
            layer.set_value(key, value)?;
        } else {
            // Create new environment layer
            let mut layer = ConfigLayer::new(
                "environment".to_string(),
                ConfigPriority::Environment,
                JsonValue::Object(serde_json::Map::new()),
                "dynamic".to_string(),
            );
            layer.set_value(key, value)?;
            self.layers.push(layer);
            self.layers.sort_by_key(|layer| layer.priority);
        }
        Ok(())
    }

    /// Get all configuration layers
    pub fn get_layers(&self) -> &[ConfigLayer] {
        &self.layers
    }

    /// Enable or disable a configuration layer
    pub fn set_layer_enabled(&mut self, layer_name: &str, enabled: bool) -> bool {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.name == layer_name) {
            layer.enabled = enabled;
            info!(
                "Layer '{}' {}",
                layer_name,
                if enabled { "enabled" } else { "disabled" }
            );
            true
        } else {
            false
        }
    }

    /// Get configuration diff between environments
    pub fn get_environment_diff(&self, other_config: &JsonValue) -> ConfigDiff {
        let current_config = self.get_merged_config();
        generate_config_diff(&current_config, other_config)
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Vec<ConfigValidationError> {
        let mut errors = Vec::new();
        let _merged_config = self.get_merged_config();

        // Validate required keys
        let required_keys = [
            "server.host",
            "server.port",
            "database.url",
            "logging.level",
        ];

        for key in required_keys {
            if self.get_config_value(key).is_none() {
                errors.push(ConfigValidationError {
                    key: key.to_string(),
                    error_type: ConfigErrorType::MissingRequired,
                    message: format!("Required configuration key '{}' is missing", key),
                    layer: None,
                });
            }
        }

        // Validate data types
        if let Some(port_value) = self.get_config_value("server.port") {
            if !port_value.is_u64() && !port_value.is_i64() {
                errors.push(ConfigValidationError {
                    key: "server.port".to_string(),
                    error_type: ConfigErrorType::InvalidType,
                    message: "server.port must be a number".to_string(),
                    layer: None,
                });
            }
        }

        // Check for conflicting configurations
        for layer in &self.layers {
            if layer.enabled {
                errors.extend(self.validate_layer(layer));
            }
        }

        errors
    }

    /// Export configuration to file
    pub fn export_config<P: AsRef<Path>>(&self, path: P, format: ConfigFormat) -> GitResult<()> {
        let config = self.get_merged_config();
        let content = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(&config).map_err(|e| {
                GitError::RepositoryOperation {
                    message: format!("Failed to serialize config to JSON: {}", e),
                }
            })?,
            ConfigFormat::Yaml => {
                serde_yaml::to_string(&config).map_err(|e| GitError::RepositoryOperation {
                    message: format!("Failed to serialize config to YAML: {}", e),
                })?
            }
        };

        std::fs::write(path.as_ref(), content).map_err(|e| GitError::RepositoryOperation {
            message: format!("Failed to write config file: {}", e),
        })?;

        info!("Exported configuration to: {}", path.as_ref().display());
        Ok(())
    }

    // Private helper methods

    fn load_base_config(&mut self) -> GitResult<()> {
        let base_path = self.config_dir.join("base.json");
        if base_path.exists() {
            let layer =
                ConfigLayer::from_file("base".to_string(), ConfigPriority::Base, &base_path)?;
            self.layers.push(layer);
        }
        Ok(())
    }

    fn load_global_config(&mut self) -> GitResult<()> {
        let global_path = self.config_dir.join("global.json");
        if global_path.exists() {
            let layer =
                ConfigLayer::from_file("global".to_string(), ConfigPriority::Global, &global_path)?;
            self.layers.push(layer);
        }
        Ok(())
    }

    fn load_environment_type_config(&mut self, env_type: EnvironmentType) -> GitResult<()> {
        let env_type_path = self.config_dir.join(format!("{}.json", env_type));
        if env_type_path.exists() {
            let layer = ConfigLayer::from_file(
                format!("env-type-{}", env_type),
                ConfigPriority::EnvironmentType,
                &env_type_path,
            )?;
            self.layers.push(layer);
        }
        Ok(())
    }

    fn load_environment_specific_config(&mut self, env: &EnvironmentConfig) -> GitResult<()> {
        let env_path = self
            .config_dir
            .join("environments")
            .join(format!("{}.json", env.name));
        if env_path.exists() {
            let layer = ConfigLayer::from_file(
                format!("env-{}", env.name),
                ConfigPriority::Environment,
                &env_path,
            )?;
            self.layers.push(layer);
        }
        Ok(())
    }

    fn load_local_config(&mut self) -> GitResult<()> {
        let local_path = self.config_dir.join("local.json");
        if local_path.exists() {
            let layer =
                ConfigLayer::from_file("local".to_string(), ConfigPriority::Local, &local_path)?;
            self.layers.push(layer);
        }
        Ok(())
    }

    fn apply_environment_overrides(&mut self, env: &EnvironmentConfig) -> GitResult<()> {
        if !env.config_overrides.is_empty() {
            let override_values = serde_json::to_value(&env.config_overrides).map_err(|e| {
                GitError::RepositoryOperation {
                    message: format!("Failed to convert environment overrides: {}", e),
                }
            })?;

            let layer = ConfigLayer::new(
                format!("env-overrides-{}", env.name),
                ConfigPriority::Environment,
                override_values,
                "environment-config".to_string(),
            );
            self.layers.push(layer);
        }
        Ok(())
    }

    fn validate_layer(&self, _layer: &ConfigLayer) -> Vec<ConfigValidationError> {
        let errors = Vec::new();

        // Add layer-specific validation logic here
        // For now, this is a placeholder for future validation rules

        errors
    }
}

/// Configuration validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationError {
    /// Configuration key that has the error
    pub key: String,
    /// Type of error
    pub error_type: ConfigErrorType,
    /// Error message
    pub message: String,
    /// Layer where the error occurred (if applicable)
    pub layer: Option<String>,
}

/// Configuration error types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigErrorType {
    /// Required configuration is missing
    MissingRequired,
    /// Invalid data type
    InvalidType,
    /// Invalid value
    InvalidValue,
    /// Conflicting values between layers
    Conflict,
    /// Deprecated configuration
    Deprecated,
}

/// Configuration diff entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiffEntry {
    /// Configuration key
    pub key: String,
    /// Change type
    pub change_type: ConfigChangeType,
    /// Old value (if changed or removed)
    pub old_value: Option<JsonValue>,
    /// New value (if changed or added)
    pub new_value: Option<JsonValue>,
}

/// Configuration change types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigChangeType {
    /// Key was added
    Added,
    /// Key was modified
    Modified,
    /// Key was removed
    Removed,
}

/// Configuration diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiff {
    /// List of changes
    pub changes: Vec<ConfigDiffEntry>,
    /// Whether there are any changes
    pub has_changes: bool,
}

/// Configuration export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

// Helper functions for JSON manipulation

fn get_nested_value<'a>(value: &'a JsonValue, key: &str) -> Option<&'a JsonValue> {
    let keys: Vec<&str> = key.split('.').collect();
    let mut current = value;

    for k in keys {
        current = current.get(k)?;
    }

    Some(current)
}

fn set_nested_value(value: &mut JsonValue, key: &str, new_value: JsonValue) -> Result<(), String> {
    let keys: Vec<&str> = key.split('.').collect();
    let mut current = value;

    // Navigate to the parent of the final key
    for &k in &keys[..keys.len() - 1] {
        if !current.is_object() {
            *current = JsonValue::Object(serde_json::Map::new());
        }

        current = current
            .as_object_mut()
            .ok_or_else(|| "Expected object".to_string())?
            .entry(k.to_string())
            .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
    }

    // Set the final value
    if let Some(final_key) = keys.last() {
        if !current.is_object() {
            *current = JsonValue::Object(serde_json::Map::new());
        }

        current
            .as_object_mut()
            .ok_or_else(|| "Expected object".to_string())?
            .insert(final_key.to_string(), new_value);
    }

    Ok(())
}

fn remove_nested_value(value: &mut JsonValue, key: &str) -> Option<JsonValue> {
    let keys: Vec<&str> = key.split('.').collect();
    let mut current = value;

    // Navigate to the parent of the final key
    for &k in &keys[..keys.len() - 1] {
        current = current.get_mut(k)?;
    }

    // Remove the final key
    if let Some(final_key) = keys.last() {
        current.as_object_mut()?.remove(*final_key)
    } else {
        None
    }
}

fn merge_json_values(target: &mut JsonValue, source: &JsonValue) {
    match (target.as_object_mut(), source.as_object()) {
        (Some(target_map), Some(source_map)) => {
            for (key, value) in source_map {
                match target_map.get_mut(key) {
                    Some(existing_value) => {
                        merge_json_values(existing_value, value);
                    }
                    None => {
                        target_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        _ => {
            // For non-object values, source completely replaces target
            *target = source.clone();
        }
    }
}

fn generate_config_diff(old_config: &JsonValue, new_config: &JsonValue) -> ConfigDiff {
    let mut changes = Vec::new();

    // This is a simplified diff implementation
    // In a real implementation, you'd want a more sophisticated diff algorithm
    collect_diff_changes("", old_config, new_config, &mut changes);

    ConfigDiff {
        has_changes: !changes.is_empty(),
        changes,
    }
}

fn collect_diff_changes(
    prefix: &str,
    old_value: &JsonValue,
    new_value: &JsonValue,
    changes: &mut Vec<ConfigDiffEntry>,
) {
    match (old_value, new_value) {
        (JsonValue::Object(old_map), JsonValue::Object(new_map)) => {
            // Check for added and modified keys
            for (key, new_val) in new_map {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };

                match old_map.get(key) {
                    Some(old_val) => {
                        if old_val != new_val {
                            if old_val.is_object() && new_val.is_object() {
                                collect_diff_changes(&full_key, old_val, new_val, changes);
                            } else {
                                changes.push(ConfigDiffEntry {
                                    key: full_key,
                                    change_type: ConfigChangeType::Modified,
                                    old_value: Some(old_val.clone()),
                                    new_value: Some(new_val.clone()),
                                });
                            }
                        }
                    }
                    None => {
                        changes.push(ConfigDiffEntry {
                            key: full_key,
                            change_type: ConfigChangeType::Added,
                            old_value: None,
                            new_value: Some(new_val.clone()),
                        });
                    }
                }
            }

            // Check for removed keys
            for (key, old_val) in old_map {
                if !new_map.contains_key(key) {
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    changes.push(ConfigDiffEntry {
                        key: full_key,
                        change_type: ConfigChangeType::Removed,
                        old_value: Some(old_val.clone()),
                        new_value: None,
                    });
                }
            }
        }
        _ => {
            if old_value != new_value {
                changes.push(ConfigDiffEntry {
                    key: prefix.to_string(),
                    change_type: ConfigChangeType::Modified,
                    old_value: Some(old_value.clone()),
                    new_value: Some(new_value.clone()),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_priority_ordering() {
        let priorities = ConfigPriority::all_levels();
        assert_eq!(priorities[0], ConfigPriority::Base);
        assert_eq!(priorities[4], ConfigPriority::Local);
        assert!(ConfigPriority::Base < ConfigPriority::Local);
    }

    #[test]
    fn test_get_nested_value() {
        let config = json!({
            "server": {
                "host": "localhost",
                "port": 8080
            }
        });

        assert_eq!(
            get_nested_value(&config, "server.host"),
            Some(&json!("localhost"))
        );
        assert_eq!(get_nested_value(&config, "server.port"), Some(&json!(8080)));
        assert_eq!(get_nested_value(&config, "server.missing"), None);
    }

    #[test]
    fn test_set_nested_value() {
        let mut config = json!({});

        set_nested_value(&mut config, "server.host", json!("localhost")).unwrap();
        assert_eq!(
            get_nested_value(&config, "server.host"),
            Some(&json!("localhost"))
        );
    }

    #[test]
    fn test_merge_json_values() {
        let mut target = json!({
            "server": {
                "host": "localhost"
            }
        });

        let source = json!({
            "server": {
                "port": 8080
            },
            "database": {
                "url": "sqlite://db.sqlite"
            }
        });

        merge_json_values(&mut target, &source);

        assert_eq!(
            get_nested_value(&target, "server.host"),
            Some(&json!("localhost"))
        );
        assert_eq!(get_nested_value(&target, "server.port"), Some(&json!(8080)));
        assert_eq!(
            get_nested_value(&target, "database.url"),
            Some(&json!("sqlite://db.sqlite"))
        );
    }
}
