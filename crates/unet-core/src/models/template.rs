//! Template-related data models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Template metadata model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    /// Unique identifier for the template
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Path to the template file in the repository
    pub path: String,
    /// Optional description
    pub description: Option<String>,
    /// Target vendor (e.g., "cisco", "juniper", "arista")
    pub vendor: Option<String>,
    /// Template type (e.g., "interface", "routing", "acl")
    pub template_type: String,
    /// Template version
    pub version: String,
    /// Git repository URL
    pub git_repository: Option<String>,
    /// Git branch
    pub git_branch: Option<String>,
    /// Git commit hash
    pub git_commit: Option<String>,
    /// Content hash for change detection
    pub content_hash: Option<String>,
    /// Template-match headers as JSON string
    pub match_headers: Option<String>,
    /// Whether the template is active
    pub is_active: bool,
    /// Custom metadata as JSON
    pub custom_data: serde_json::Value,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Template {
    /// Creates a new template with default values
    pub fn new(name: String, path: String, template_type: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            path,
            description: None,
            vendor: None,
            template_type,
            version: "1.0.0".to_string(),
            git_repository: None,
            git_branch: None,
            git_commit: None,
            content_hash: None,
            match_headers: None,
            is_active: true,
            custom_data: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validates the template data
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Template name cannot be empty".to_string());
        }

        if self.path.trim().is_empty() {
            return Err("Template path cannot be empty".to_string());
        }

        if self.template_type.trim().is_empty() {
            return Err("Template type cannot be empty".to_string());
        }

        if self.version.trim().is_empty() {
            return Err("Template version cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Template assignment model - links templates to nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateAssignment {
    /// Unique identifier for the assignment
    pub id: Uuid,
    /// Node ID this template is assigned to
    pub node_id: Uuid,
    /// Template ID being assigned
    pub template_id: Uuid,
    /// Type of assignment (e.g., "manual", "automatic", "policy")
    pub assignment_type: String,
    /// Priority for ordering (lower numbers = higher priority)
    pub priority: i32,
    /// Whether the assignment is active
    pub is_active: bool,
    /// Specific config section this applies to
    pub config_section: Option<String>,
    /// Template variables as JSON
    pub variables: Option<String>,
    /// Custom metadata as JSON
    pub custom_data: serde_json::Value,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TemplateAssignment {
    /// Creates a new template assignment
    pub fn new(node_id: Uuid, template_id: Uuid, assignment_type: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            node_id,
            template_id,
            assignment_type,
            priority: 100,
            is_active: true,
            config_section: None,
            variables: None,
            custom_data: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validates the assignment data
    pub fn validate(&self) -> Result<(), String> {
        if self.assignment_type.trim().is_empty() {
            return Err("Assignment type cannot be empty".to_string());
        }

        if self.priority < 0 {
            return Err("Assignment priority cannot be negative".to_string());
        }

        Ok(())
    }
}

/// Template version model for version control
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVersion {
    /// Unique identifier for the version
    pub id: Uuid,
    /// Template ID this version belongs to
    pub template_id: Uuid,
    /// Version string
    pub version: String,
    /// Git commit hash
    pub git_commit: String,
    /// Content hash
    pub content_hash: String,
    /// Template content
    pub content: String,
    /// Change log for this version
    pub change_log: Option<String>,
    /// Whether this is a stable version
    pub is_stable: bool,
    /// Custom metadata as JSON
    pub custom_data: serde_json::Value,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TemplateVersion {
    /// Creates a new template version
    pub fn new(template_id: Uuid, version: String, git_commit: String, content: String) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());

        Self {
            id: Uuid::new_v4(),
            template_id,
            version,
            git_commit,
            content_hash,
            content,
            change_log: None,
            is_stable: false,
            custom_data: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
        }
    }

    /// Validates the version data
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("Version cannot be empty".to_string());
        }

        if self.git_commit.trim().is_empty() {
            return Err("Git commit cannot be empty".to_string());
        }

        if self.content_hash.trim().is_empty() {
            return Err("Content hash cannot be empty".to_string());
        }

        if self.content.trim().is_empty() {
            return Err("Content cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Template usage analytics model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateUsage {
    /// Unique identifier for the usage record
    pub id: Uuid,
    /// Template ID being used
    pub template_id: Uuid,
    /// Node ID where template was used (optional)
    pub node_id: Option<Uuid>,
    /// Operation performed (e.g., "render", "validate", "preview")
    pub operation: String,
    /// Status of the operation (e.g., "success", "error", "warning")
    pub status: String,
    /// Render time in milliseconds
    pub render_time: Option<i32>,
    /// Output size in bytes
    pub output_size: Option<i32>,
    /// Error message if operation failed
    pub error_message: Option<String>,
    /// Template context as JSON
    pub context: Option<String>,
    /// Custom metadata as JSON
    pub custom_data: serde_json::Value,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TemplateUsage {
    /// Creates a new template usage record
    pub fn new(template_id: Uuid, operation: String, status: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            template_id,
            node_id: None,
            operation,
            status,
            render_time: None,
            output_size: None,
            error_message: None,
            context: None,
            custom_data: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
        }
    }

    /// Validates the usage data
    pub fn validate(&self) -> Result<(), String> {
        if self.operation.trim().is_empty() {
            return Err("Operation cannot be empty".to_string());
        }

        if self.status.trim().is_empty() {
            return Err("Status cannot be empty".to_string());
        }

        if let Some(render_time) = self.render_time {
            if render_time < 0 {
                return Err("Render time cannot be negative".to_string());
            }
        }

        if let Some(output_size) = self.output_size {
            if output_size < 0 {
                return Err("Output size cannot be negative".to_string());
            }
        }

        Ok(())
    }
}

/// Template rendering request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRenderRequest {
    /// Template ID to render
    pub template_id: Uuid,
    /// Node ID to render for
    pub node_id: Uuid,
    /// Additional variables to pass to template
    pub variables: HashMap<String, serde_json::Value>,
    /// Specific config section to render
    pub config_section: Option<String>,
}

/// Template rendering result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRenderResult {
    /// Whether rendering was successful
    pub success: bool,
    /// Rendered output
    pub output: Option<String>,
    /// Error message if rendering failed
    pub error: Option<String>,
    /// Render time in milliseconds
    pub render_time: u64,
    /// Template version used
    pub template_version: String,
    /// Variables used in rendering
    pub variables_used: HashMap<String, serde_json::Value>,
}

impl TemplateRenderResult {
    /// Creates a successful render result
    pub fn success(output: String, render_time: u64, template_version: String) -> Self {
        Self {
            success: true,
            output: Some(output),
            error: None,
            render_time,
            template_version,
            variables_used: HashMap::new(),
        }
    }

    /// Creates a failed render result
    pub fn error(error: String, render_time: u64) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(error),
            render_time,
            template_version: "unknown".to_string(),
            variables_used: HashMap::new(),
        }
    }
}
