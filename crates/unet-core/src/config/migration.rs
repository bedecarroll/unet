//! Configuration migration utilities for μNet
//!
//! This module provides tools for migrating configuration files between different
//! versions of μNet, handling schema changes, and ensuring backward compatibility.

use crate::config::Config;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use toml;

/// Version information for configuration schema
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Migration rule for transforming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRule {
    /// Rule name for identification
    pub name: String,
    /// Source version range
    pub from_version: Version,
    /// Target version
    pub to_version: Version,
    /// Type of migration operation
    pub operation: MigrationOperation,
    /// Field path to modify
    pub field_path: String,
    /// Additional parameters for the operation
    pub parameters: HashMap<String, toml::Value>,
}

/// Types of migration operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationOperation {
    /// Add a new field with default value
    AddField { default_value: toml::Value },
    /// Remove a field
    RemoveField,
    /// Rename a field
    RenameField { new_name: String },
    /// Transform field value
    TransformValue { transformation: ValueTransformation },
    /// Move field to different location
    MoveField { new_path: String },
    /// Split field into multiple fields
    SplitField {
        new_fields: HashMap<String, toml::Value>,
    },
    /// Merge multiple fields into one
    MergeFields {
        source_fields: Vec<String>,
        merge_strategy: MergeStrategy,
    },
    /// Custom transformation function
    CustomTransform { function_name: String },
}

/// Value transformation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueTransformation {
    /// Convert string to number
    StringToNumber,
    /// Convert number to string
    NumberToString,
    /// Convert boolean to string
    BooleanToString,
    /// Convert string to boolean
    StringToBoolean,
    /// Apply regex replacement
    RegexReplace {
        pattern: String,
        replacement: String,
    },
    /// Convert to uppercase
    ToUppercase,
    /// Convert to lowercase
    ToLowercase,
    /// Apply custom mapping
    MapValue {
        mapping: HashMap<String, toml::Value>,
    },
}

/// Merge strategy for combining fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Take first non-null value
    FirstNonNull,
    /// Concatenate string values
    Concatenate { separator: String },
    /// Sum numeric values
    Sum,
    /// Create array of values
    Array,
    /// Create object with field names as keys
    Object,
}

/// Configuration migration engine
#[derive(Debug)]
pub struct ConfigMigrator {
    /// Available migration rules
    rules: Vec<MigrationRule>,
    /// Current schema version
    current_version: Version,
}

/// Migration result with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Whether migration was successful
    pub success: bool,
    /// Source version
    pub from_version: Version,
    /// Target version
    pub to_version: Version,
    /// Applied migration rules
    pub applied_rules: Vec<String>,
    /// Any warnings generated during migration
    pub warnings: Vec<String>,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Backup file path (if created)
    pub backup_path: Option<String>,
}

/// Configuration backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    /// Original file path
    pub original_path: String,
    /// Backup file path
    pub backup_path: String,
    /// Backup timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Version of configuration that was backed up
    pub version: Version,
    /// SHA-256 hash of original file
    pub file_hash: String,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse version from string (e.g., "1.2.3")
    pub fn parse(version_str: &str) -> Result<Self> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::config(format!(
                "Invalid version format: {}",
                version_str
            )));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| Error::config(format!("Invalid major version: {}", parts[0])))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| Error::config(format!("Invalid minor version: {}", parts[1])))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| Error::config(format!("Invalid patch version: {}", parts[2])))?;

        Ok(Version::new(major, minor, patch))
    }

    /// Convert version to string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl ConfigMigrator {
    /// Create a new migrator with current version
    pub fn new(current_version: Version) -> Self {
        let mut migrator = Self {
            rules: Vec::new(),
            current_version,
        };

        // Load built-in migration rules
        migrator.load_builtin_rules();

        migrator
    }

    /// Load built-in migration rules for known version transitions
    fn load_builtin_rules(&mut self) {
        // Example: Migration from 0.1.0 to 0.2.0
        self.add_rule(MigrationRule {
            name: "add_resource_management".to_string(),
            from_version: Version::new(0, 1, 0),
            to_version: Version::new(0, 2, 0),
            operation: MigrationOperation::AddField {
                default_value: toml::Value::Boolean(false),
            },
            field_path: "resource_management.memory.enabled".to_string(),
            parameters: HashMap::new(),
        });

        // Migration from 0.2.0 to 1.0.0 - rename auth.token_endpoint
        self.add_rule(MigrationRule {
            name: "rename_token_endpoint".to_string(),
            from_version: Version::new(0, 2, 0),
            to_version: Version::new(1, 0, 0),
            operation: MigrationOperation::RenameField {
                new_name: "validation_endpoint".to_string(),
            },
            field_path: "auth.token_endpoint".to_string(),
            parameters: HashMap::new(),
        });

        // Migration for TLS configuration restructuring
        self.add_rule(MigrationRule {
            name: "restructure_tls_config".to_string(),
            from_version: Version::new(0, 9, 0),
            to_version: Version::new(1, 0, 0),
            operation: MigrationOperation::MoveField {
                new_path: "server.tls".to_string(),
            },
            field_path: "tls".to_string(),
            parameters: HashMap::new(),
        });

        // Migration for log level value changes
        self.add_rule(MigrationRule {
            name: "normalize_log_levels".to_string(),
            from_version: Version::new(0, 8, 0),
            to_version: Version::new(0, 9, 0),
            operation: MigrationOperation::TransformValue {
                transformation: ValueTransformation::MapValue {
                    mapping: {
                        let mut map = HashMap::new();
                        map.insert(
                            "verbose".to_string(),
                            toml::Value::String("debug".to_string()),
                        );
                        map.insert(
                            "normal".to_string(),
                            toml::Value::String("info".to_string()),
                        );
                        map.insert("quiet".to_string(), toml::Value::String("warn".to_string()));
                        map
                    },
                },
            },
            field_path: "logging.level".to_string(),
            parameters: HashMap::new(),
        });
    }

    /// Add a custom migration rule
    pub fn add_rule(&mut self, rule: MigrationRule) {
        self.rules.push(rule);
    }

    /// Load migration rules from a file
    pub fn load_rules_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::config_with_source("Failed to read migration rules file", e))?;

        let rules: Vec<MigrationRule> = toml::from_str(&content)
            .map_err(|e| Error::config_with_source("Failed to parse migration rules", e))?;

        self.rules.extend(rules);
        Ok(())
    }

    /// Migrate configuration from file
    pub fn migrate_file<P: AsRef<Path>>(
        &self,
        config_path: P,
        target_version: Option<Version>,
    ) -> Result<MigrationResult> {
        let config_path = config_path.as_ref();
        let target_version = target_version.unwrap_or_else(|| self.current_version.clone());

        // Read the original configuration
        let original_content = std::fs::read_to_string(config_path)
            .map_err(|e| Error::config_with_source("Failed to read configuration file", e))?;

        // Parse as generic TOML to preserve unknown fields
        let mut config_value: toml::Value = toml::from_str(&original_content)
            .map_err(|e| Error::config_with_source("Failed to parse configuration file", e))?;

        // Detect current version
        let current_version = self.detect_version(&config_value)?;

        // Create backup
        let backup_path = self.create_backup(config_path, &current_version)?;

        // Perform migration
        let mut result = MigrationResult {
            success: false,
            from_version: current_version.clone(),
            to_version: target_version.clone(),
            applied_rules: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            backup_path: Some(backup_path),
        };

        // Apply migration rules
        if let Err(e) = self.apply_migrations(
            &mut config_value,
            &current_version,
            &target_version,
            &mut result,
        ) {
            result.errors.push(e.to_string());
            return Ok(result);
        }

        // Add version metadata to migrated configuration
        if let toml::Value::Table(ref mut table) = config_value {
            table.insert(
                "_schema_version".to_string(),
                toml::Value::String(target_version.to_string()),
            );
        }

        // Write migrated configuration
        let migrated_content = toml::to_string_pretty(&config_value).map_err(|e| {
            Error::config_with_source("Failed to serialize migrated configuration", e)
        })?;

        std::fs::write(config_path, migrated_content)
            .map_err(|e| Error::config_with_source("Failed to write migrated configuration", e))?;

        result.success = true;
        Ok(result)
    }

    /// Migrate configuration in memory
    pub fn migrate_config(
        &self,
        config: &Config,
        target_version: Option<Version>,
    ) -> Result<(Config, MigrationResult)> {
        let target_version = target_version.unwrap_or_else(|| self.current_version.clone());

        // Serialize config to TOML value for manipulation
        let config_str = toml::to_string_pretty(config)
            .map_err(|e| Error::config_with_source("Failed to serialize configuration", e))?;

        let mut config_value: toml::Value = toml::from_str(&config_str)
            .map_err(|e| Error::config_with_source("Failed to parse configuration", e))?;

        // Detect version (use default if not present)
        let current_version = self
            .detect_version(&config_value)
            .unwrap_or_else(|_| Version::new(0, 1, 0));

        let mut result = MigrationResult {
            success: false,
            from_version: current_version.clone(),
            to_version: target_version.clone(),
            applied_rules: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            backup_path: None,
        };

        // Apply migrations
        if let Err(e) = self.apply_migrations(
            &mut config_value,
            &current_version,
            &target_version,
            &mut result,
        ) {
            result.errors.push(e.to_string());
            let migrated_config =
                toml::from_str::<Config>(&toml::to_string(&config_value).unwrap())
                    .unwrap_or_else(|_| config.clone());
            return Ok((migrated_config, result));
        }

        // Convert back to Config struct
        let migrated_config = toml::from_str::<Config>(&toml::to_string(&config_value).unwrap())
            .map_err(|e| {
                Error::config_with_source("Failed to deserialize migrated configuration", e)
            })?;

        result.success = true;
        Ok((migrated_config, result))
    }

    /// Detect configuration version from TOML value
    fn detect_version(&self, config: &toml::Value) -> Result<Version> {
        if let toml::Value::Table(table) = config {
            if let Some(toml::Value::String(version_str)) = table.get("_schema_version") {
                return Version::parse(version_str);
            }
        }

        // Try to infer version from configuration structure
        self.infer_version(config)
    }

    /// Infer version from configuration structure
    fn infer_version(&self, config: &toml::Value) -> Result<Version> {
        if let toml::Value::Table(table) = config {
            // Check for version-specific fields
            if table.contains_key("resource_management") {
                if table
                    .get("resource_management")
                    .and_then(|rm| rm.as_table())
                    .and_then(|rm_table| rm_table.get("monitoring"))
                    .is_some()
                {
                    return Ok(Version::new(1, 0, 0)); // Latest version
                }
                return Ok(Version::new(0, 2, 0)); // Resource management added
            }

            if table.contains_key("cluster") {
                return Ok(Version::new(0, 1, 5)); // Cluster support added
            }

            // Default to earliest version if structure is minimal
            return Ok(Version::new(0, 1, 0));
        }

        Err(Error::config("Invalid configuration format".to_string()))
    }

    /// Apply migration rules to transform configuration
    fn apply_migrations(
        &self,
        config: &mut toml::Value,
        from_version: &Version,
        to_version: &Version,
        result: &mut MigrationResult,
    ) -> Result<()> {
        // Find applicable rules
        let applicable_rules: Vec<&MigrationRule> = self
            .rules
            .iter()
            .filter(|rule| rule.from_version <= *from_version && rule.to_version <= *to_version)
            .collect();

        // Sort rules by target version
        let mut sorted_rules = applicable_rules;
        sorted_rules.sort_by_key(|rule| &rule.to_version);

        // Apply each rule
        for rule in sorted_rules {
            match self.apply_rule(config, rule) {
                Ok(()) => {
                    result.applied_rules.push(rule.name.clone());
                }
                Err(e) => {
                    result
                        .warnings
                        .push(format!("Failed to apply rule '{}': {}", rule.name, e));
                }
            }
        }

        Ok(())
    }

    /// Apply a single migration rule
    fn apply_rule(&self, config: &mut toml::Value, rule: &MigrationRule) -> Result<()> {
        match &rule.operation {
            MigrationOperation::AddField { default_value } => {
                self.add_field(config, &rule.field_path, default_value.clone())
            }
            MigrationOperation::RemoveField => self.remove_field(config, &rule.field_path),
            MigrationOperation::RenameField { new_name } => {
                self.rename_field(config, &rule.field_path, new_name)
            }
            MigrationOperation::TransformValue { transformation } => {
                self.transform_value(config, &rule.field_path, transformation)
            }
            MigrationOperation::MoveField { new_path } => {
                self.move_field(config, &rule.field_path, new_path)
            }
            MigrationOperation::SplitField { new_fields } => {
                self.split_field(config, &rule.field_path, new_fields)
            }
            MigrationOperation::MergeFields {
                source_fields,
                merge_strategy,
            } => self.merge_fields(config, source_fields, merge_strategy, &rule.field_path),
            MigrationOperation::CustomTransform { function_name } => {
                // Custom transforms not yet implemented
                eprintln!(
                    "Warning: Custom transform '{}' not implemented",
                    function_name
                );
                Ok(())
            }
        }
    }

    /// Add a field with default value
    fn add_field(
        &self,
        config: &mut toml::Value,
        field_path: &str,
        default_value: toml::Value,
    ) -> Result<()> {
        let path_parts: Vec<&str> = field_path.split('.').collect();
        let mut current = config;

        // Navigate to parent
        for part in &path_parts[..path_parts.len() - 1] {
            if let toml::Value::Table(table) = current {
                if !table.contains_key(*part) {
                    table.insert(part.to_string(), toml::Value::Table(toml::map::Map::new()));
                }
                current = table.get_mut(*part).unwrap();
            } else {
                return Err(Error::config(format!(
                    "Cannot navigate to field path: {}",
                    field_path
                )));
            }
        }

        // Add the field
        if let toml::Value::Table(table) = current {
            let field_name = path_parts.last().unwrap();
            if !table.contains_key(*field_name) {
                table.insert(field_name.to_string(), default_value);
            }
        }

        Ok(())
    }

    /// Remove a field
    fn remove_field(&self, config: &mut toml::Value, field_path: &str) -> Result<()> {
        let path_parts: Vec<&str> = field_path.split('.').collect();
        let mut current = config;

        // Navigate to parent
        for part in &path_parts[..path_parts.len() - 1] {
            if let toml::Value::Table(table) = current {
                if let Some(next) = table.get_mut(*part) {
                    current = next;
                } else {
                    return Ok(()); // Field doesn't exist, nothing to remove
                }
            }
        }

        // Remove the field
        if let toml::Value::Table(table) = current {
            let field_name = path_parts.last().unwrap();
            table.remove(*field_name);
        }

        Ok(())
    }

    /// Rename a field
    fn rename_field(
        &self,
        config: &mut toml::Value,
        field_path: &str,
        new_name: &str,
    ) -> Result<()> {
        let path_parts: Vec<&str> = field_path.split('.').collect();
        let mut current = config;

        // Navigate to parent
        for part in &path_parts[..path_parts.len() - 1] {
            if let toml::Value::Table(table) = current {
                if let Some(next) = table.get_mut(*part) {
                    current = next;
                } else {
                    return Ok(()); // Field doesn't exist
                }
            }
        }

        // Rename the field
        if let toml::Value::Table(table) = current {
            let old_name = path_parts.last().unwrap();
            if let Some(value) = table.remove(*old_name) {
                table.insert(new_name.to_string(), value);
            }
        }

        Ok(())
    }

    /// Transform field value
    fn transform_value(
        &self,
        config: &mut toml::Value,
        field_path: &str,
        transformation: &ValueTransformation,
    ) -> Result<()> {
        let path_parts: Vec<&str> = field_path.split('.').collect();
        let mut current = config;

        // Navigate to field
        for part in &path_parts[..path_parts.len() - 1] {
            if let toml::Value::Table(table) = current {
                if let Some(next) = table.get_mut(*part) {
                    current = next;
                } else {
                    return Ok(()); // Field doesn't exist
                }
            }
        }

        // Transform the value
        if let toml::Value::Table(table) = current {
            let field_name = path_parts.last().unwrap();
            if let Some(value) = table.get_mut(*field_name) {
                *value = self.apply_value_transformation(value, transformation)?;
            }
        }

        Ok(())
    }

    /// Apply value transformation
    fn apply_value_transformation(
        &self,
        value: &toml::Value,
        transformation: &ValueTransformation,
    ) -> Result<toml::Value> {
        match transformation {
            ValueTransformation::StringToNumber => {
                if let toml::Value::String(s) = value {
                    if let Ok(i) = s.parse::<i64>() {
                        Ok(toml::Value::Integer(i))
                    } else if let Ok(f) = s.parse::<f64>() {
                        Ok(toml::Value::Float(f))
                    } else {
                        Err(Error::config(format!("Cannot convert '{}' to number", s)))
                    }
                } else {
                    Err(Error::config("Value is not a string".to_string()))
                }
            }
            ValueTransformation::NumberToString => match value {
                toml::Value::Integer(i) => Ok(toml::Value::String(i.to_string())),
                toml::Value::Float(f) => Ok(toml::Value::String(f.to_string())),
                _ => Err(Error::config("Value is not a number".to_string())),
            },
            ValueTransformation::BooleanToString => {
                if let toml::Value::Boolean(b) = value {
                    Ok(toml::Value::String(b.to_string()))
                } else {
                    Err(Error::config("Value is not a boolean".to_string()))
                }
            }
            ValueTransformation::StringToBoolean => {
                if let toml::Value::String(s) = value {
                    match s.to_lowercase().as_str() {
                        "true" | "yes" | "1" | "on" | "enable" | "enabled" => {
                            Ok(toml::Value::Boolean(true))
                        }
                        "false" | "no" | "0" | "off" | "disable" | "disabled" => {
                            Ok(toml::Value::Boolean(false))
                        }
                        _ => Err(Error::config(format!("Cannot convert '{}' to boolean", s))),
                    }
                } else {
                    Err(Error::config("Value is not a string".to_string()))
                }
            }
            ValueTransformation::ToUppercase => {
                if let toml::Value::String(s) = value {
                    Ok(toml::Value::String(s.to_uppercase()))
                } else {
                    Err(Error::config("Value is not a string".to_string()))
                }
            }
            ValueTransformation::ToLowercase => {
                if let toml::Value::String(s) = value {
                    Ok(toml::Value::String(s.to_lowercase()))
                } else {
                    Err(Error::config("Value is not a string".to_string()))
                }
            }
            ValueTransformation::MapValue { mapping } => {
                if let Some(mapped_value) = mapping.get(&value.to_string()) {
                    Ok(mapped_value.clone())
                } else {
                    Ok(value.clone()) // Return original if no mapping found
                }
            }
            ValueTransformation::RegexReplace {
                pattern,
                replacement,
            } => {
                if let toml::Value::String(s) = value {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| Error::config_with_source("Invalid regex pattern", e))?;
                    let replaced = regex.replace_all(s, replacement);
                    Ok(toml::Value::String(replaced.to_string()))
                } else {
                    Err(Error::config("Value is not a string".to_string()))
                }
            }
        }
    }

    /// Move field to new location
    fn move_field(&self, config: &mut toml::Value, old_path: &str, new_path: &str) -> Result<()> {
        // Get the value from old location
        let value = self.get_field_value(config, old_path)?;

        // Remove from old location
        self.remove_field(config, old_path)?;

        // Add to new location
        if let Some(val) = value {
            self.add_field(config, new_path, val)?;
        }

        Ok(())
    }

    /// Split field into multiple fields
    fn split_field(
        &self,
        config: &mut toml::Value,
        field_path: &str,
        new_fields: &HashMap<String, toml::Value>,
    ) -> Result<()> {
        // Remove original field
        self.remove_field(config, field_path)?;

        // Add new fields
        for (new_field_path, default_value) in new_fields {
            self.add_field(config, new_field_path, default_value.clone())?;
        }

        Ok(())
    }

    /// Merge multiple fields into one
    fn merge_fields(
        &self,
        config: &mut toml::Value,
        source_fields: &[String],
        merge_strategy: &MergeStrategy,
        target_field: &str,
    ) -> Result<()> {
        let mut values = Vec::new();

        // Collect values from source fields
        for field_path in source_fields {
            if let Ok(Some(value)) = self.get_field_value(config, field_path) {
                values.push(value);
            }
        }

        // Apply merge strategy
        let merged_value = self.apply_merge_strategy(&values, merge_strategy)?;

        // Set merged value
        self.add_field(config, target_field, merged_value)?;

        // Remove source fields
        for field_path in source_fields {
            self.remove_field(config, field_path)?;
        }

        Ok(())
    }

    /// Apply merge strategy to combine values
    fn apply_merge_strategy(
        &self,
        values: &[toml::Value],
        strategy: &MergeStrategy,
    ) -> Result<toml::Value> {
        match strategy {
            MergeStrategy::FirstNonNull => Ok(values
                .first()
                .cloned()
                .unwrap_or(toml::Value::String("".to_string()))),
            MergeStrategy::Concatenate { separator } => {
                let strings: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                Ok(toml::Value::String(strings.join(separator)))
            }
            MergeStrategy::Sum => {
                let mut sum = 0i64;
                for value in values {
                    match value {
                        toml::Value::Integer(i) => sum += i,
                        toml::Value::Float(f) => sum += *f as i64,
                        _ => {
                            return Err(Error::config("Cannot sum non-numeric values".to_string()));
                        }
                    }
                }
                Ok(toml::Value::Integer(sum))
            }
            MergeStrategy::Array => Ok(toml::Value::Array(values.to_vec())),
            MergeStrategy::Object => {
                let mut table = toml::map::Map::new();
                for (i, value) in values.iter().enumerate() {
                    table.insert(format!("field_{}", i), value.clone());
                }
                Ok(toml::Value::Table(table))
            }
        }
    }

    /// Apply custom transformation function

    /// Get field value by path
    fn get_field_value(
        &self,
        config: &toml::Value,
        field_path: &str,
    ) -> Result<Option<toml::Value>> {
        let path_parts: Vec<&str> = field_path.split('.').collect();
        let mut current = config;

        for part in &path_parts[..path_parts.len() - 1] {
            if let toml::Value::Table(table) = current {
                if let Some(next) = table.get(*part) {
                    current = next;
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }

        if let toml::Value::Table(table) = current {
            let field_name = path_parts.last().unwrap();
            Ok(table.get(*field_name).cloned())
        } else {
            Ok(None)
        }
    }

    /// Create backup of configuration file
    fn create_backup<P: AsRef<Path>>(&self, config_path: P, version: &Version) -> Result<String> {
        let config_path = config_path.as_ref();
        let timestamp = chrono::Utc::now();
        let backup_filename = format!(
            "{}.backup.{}.{}",
            config_path.file_name().unwrap().to_str().unwrap(),
            version.to_string(),
            timestamp.format("%Y%m%d_%H%M%S")
        );

        let backup_path = config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(backup_filename);

        std::fs::copy(config_path, &backup_path)
            .map_err(|e| Error::config_with_source("Failed to create backup", e))?;

        Ok(backup_path.to_string_lossy().to_string())
    }

    /// List available migration rules
    pub fn list_rules(&self) -> &[MigrationRule] {
        &self.rules
    }

    /// Check if migration is needed
    pub fn needs_migration(&self, config: &Config) -> Result<bool> {
        let config_str = toml::to_string_pretty(config)
            .map_err(|e| Error::config_with_source("Failed to serialize configuration", e))?;

        let config_value: toml::Value = toml::from_str(&config_str)
            .map_err(|e| Error::config_with_source("Failed to parse configuration", e))?;

        let current_version = self
            .detect_version(&config_value)
            .unwrap_or_else(|_| Version::new(0, 1, 0));

        Ok(current_version < self.current_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_add_field_migration() {
        let migrator = ConfigMigrator::new(Version::new(1, 0, 0));
        let mut config = toml::Value::Table(toml::map::Map::new());

        migrator
            .add_field(
                &mut config,
                "resource_management.memory.enabled",
                toml::Value::Boolean(true),
            )
            .unwrap();

        // Verify field was added
        if let toml::Value::Table(table) = &config {
            let rm = table.get("resource_management").unwrap();
            if let toml::Value::Table(rm_table) = rm {
                assert_eq!(rm_table.get("enabled"), Some(&toml::Value::Boolean(true)));
            } else {
                panic!("Expected resource_management to be a table");
            }
        } else {
            panic!("Expected config to be a table");
        }
    }

    #[test]
    fn test_value_transformation() {
        let migrator = ConfigMigrator::new(Version::new(1, 0, 0));

        // Test string to number
        let value = toml::Value::String("42".to_string());
        let result = migrator
            .apply_value_transformation(&value, &ValueTransformation::StringToNumber)
            .unwrap();
        assert_eq!(result, toml::Value::Integer(42));

        // Test boolean to string
        let value = toml::Value::Boolean(true);
        let result = migrator
            .apply_value_transformation(&value, &ValueTransformation::BooleanToString)
            .unwrap();
        assert_eq!(result, toml::Value::String("true".to_string()));
    }

    #[test]
    fn test_config_migration() {
        let migrator = ConfigMigrator::new(Version::new(1, 0, 0));
        let config = Config::default();

        let (migrated_config, result) = migrator.migrate_config(&config, None).unwrap();

        assert!(result.success);
        assert!(!result.applied_rules.is_empty());

        // Verify resource management was added
        assert!(migrated_config.resource_management.memory.enabled);
    }
}
