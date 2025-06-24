//! Template management HTTP handlers

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    error::{ServerError, ServerResult},
    server::AppState,
};
use unet_core::{
    models::template::{
        Template, TemplateAssignment, TemplateRenderRequest, TemplateRenderResult, TemplateUsage,
    },
    template::{ContextBuilder, TemplateRenderer},
};

/// Request to create a new template
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
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
    pub version: Option<String>,
    /// Git repository URL
    pub git_repository: Option<String>,
    /// Git branch
    pub git_branch: Option<String>,
    /// Template-match headers as JSON string
    pub match_headers: Option<String>,
    /// Whether the template is active
    pub is_active: Option<bool>,
    /// Custom metadata as JSON
    pub custom_data: Option<serde_json::Value>,
}

/// Request to update an existing template
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    /// Human-readable name
    pub name: Option<String>,
    /// Path to the template file in the repository
    pub path: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Target vendor (e.g., "cisco", "juniper", "arista")
    pub vendor: Option<String>,
    /// Template type (e.g., "interface", "routing", "acl")
    pub template_type: Option<String>,
    /// Template version
    pub version: Option<String>,
    /// Git repository URL
    pub git_repository: Option<String>,
    /// Git branch
    pub git_branch: Option<String>,
    /// Template-match headers as JSON string
    pub match_headers: Option<String>,
    /// Whether the template is active
    pub is_active: Option<bool>,
    /// Custom metadata as JSON
    pub custom_data: Option<serde_json::Value>,
}

/// Query parameters for listing templates
#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    /// Filter by vendor
    pub vendor: Option<String>,
    /// Filter by template type
    pub template_type: Option<String>,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Response for template operations
#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    /// Template data
    pub template: Template,
}

/// Response for listing templates
#[derive(Debug, Serialize)]
pub struct ListTemplatesResponse {
    /// List of templates
    pub templates: Vec<Template>,
    /// Total number of templates available
    pub total_count: usize,
    /// Number of templates returned
    pub returned_count: usize,
}

/// Create a new template
pub async fn create_template(
    State(state): State<AppState>,
    Json(request): Json<CreateTemplateRequest>,
) -> ServerResult<Json<TemplateResponse>> {
    info!(
        name = %request.name,
        template_type = %request.template_type,
        "Creating new template"
    );

    // Validate required fields
    if request.name.trim().is_empty() {
        return Err(ServerError::BadRequest(
            "Template name cannot be empty".to_string(),
        ));
    }

    if request.path.trim().is_empty() {
        return Err(ServerError::BadRequest(
            "Template path cannot be empty".to_string(),
        ));
    }

    if request.template_type.trim().is_empty() {
        return Err(ServerError::BadRequest(
            "Template type cannot be empty".to_string(),
        ));
    }

    // Create template
    let mut template = Template::new(
        request.name.clone(),
        request.path.clone(),
        request.template_type.clone(),
    );

    // Set optional fields
    template.description = request.description;
    template.vendor = request.vendor;
    if let Some(version) = request.version {
        template.version = version;
    }
    template.git_repository = request.git_repository;
    template.git_branch = request.git_branch;
    template.match_headers = request.match_headers;
    template.is_active = request.is_active.unwrap_or(true);
    template.custom_data = request.custom_data.unwrap_or(serde_json::Value::Null);

    // Validate template
    if let Err(e) = template.validate() {
        return Err(ServerError::BadRequest(e));
    }

    // Save to datastore
    let created_template = state
        .datastore
        .create_template(&template)
        .await
        .map_err(|e| {
            error!("Failed to create template: {}", e);
            ServerError::Internal(format!("Failed to create template: {}", e))
        })?;

    info!(
        template_id = %created_template.id,
        template_name = %created_template.name,
        "Template created successfully"
    );

    Ok(Json(TemplateResponse {
        template: created_template,
    }))
}

/// Get a template by ID
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<TemplateResponse>> {
    info!(template_id = %id, "Getting template");

    let template = state
        .datastore
        .get_template(&id)
        .await
        .map_err(|e| {
            error!("Failed to get template {}: {}", id, e);
            ServerError::Internal(format!("Failed to get template: {}", e))
        })?
        .ok_or_else(|| ServerError::NotFound(format!("Template with id {} not found", id)))?;

    Ok(Json(TemplateResponse { template }))
}

/// List templates with optional filtering
pub async fn list_templates(
    State(state): State<AppState>,
    Query(query): Query<ListTemplatesQuery>,
) -> ServerResult<Json<ListTemplatesResponse>> {
    info!("Listing templates with filter: {:?}", query);

    // Build query options
    let mut options = unet_core::datastore::QueryOptions::default();

    // Set pagination
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    options.pagination = Some(
        unet_core::datastore::Pagination::new(limit, offset).map_err(|e| {
            ServerError::BadRequest(format!("Invalid pagination parameters: {}", e))
        })?,
    );

    // Add filters
    if let Some(vendor) = &query.vendor {
        options.filters.push(unet_core::datastore::Filter {
            field: "vendor".to_string(),
            operation: unet_core::datastore::FilterOperation::Equals,
            value: unet_core::datastore::FilterValue::String(vendor.clone()),
        });
    }
    if let Some(template_type) = &query.template_type {
        options.filters.push(unet_core::datastore::Filter {
            field: "template_type".to_string(),
            operation: unet_core::datastore::FilterOperation::Equals,
            value: unet_core::datastore::FilterValue::String(template_type.clone()),
        });
    }
    if let Some(is_active) = query.is_active {
        options.filters.push(unet_core::datastore::Filter {
            field: "is_active".to_string(),
            operation: unet_core::datastore::FilterOperation::Equals,
            value: unet_core::datastore::FilterValue::Boolean(is_active),
        });
    }

    let result = state
        .datastore
        .list_templates(&options)
        .await
        .map_err(|e| {
            error!("Failed to list templates: {}", e);
            ServerError::Internal(format!("Failed to list templates: {}", e))
        })?;

    info!(
        total_count = result.total_count,
        returned_count = result.items.len(),
        "Templates listed successfully"
    );

    let returned_count = result.items.len();
    Ok(Json(ListTemplatesResponse {
        templates: result.items,
        total_count: result.total_count,
        returned_count,
    }))
}

/// Update an existing template
pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTemplateRequest>,
) -> ServerResult<Json<TemplateResponse>> {
    info!(template_id = %id, "Updating template");

    // Get existing template
    let mut template = state
        .datastore
        .get_template(&id)
        .await
        .map_err(|e| {
            error!("Failed to get template {}: {}", id, e);
            ServerError::Internal(format!("Failed to get template: {}", e))
        })?
        .ok_or_else(|| ServerError::NotFound(format!("Template with id {} not found", id)))?;

    // Update fields if provided
    if let Some(name) = request.name {
        template.name = name;
    }
    if let Some(path) = request.path {
        template.path = path;
    }
    if let Some(description) = request.description {
        template.description = Some(description);
    }
    if let Some(vendor) = request.vendor {
        template.vendor = Some(vendor);
    }
    if let Some(template_type) = request.template_type {
        template.template_type = template_type;
    }
    if let Some(version) = request.version {
        template.version = version;
    }
    if let Some(git_repository) = request.git_repository {
        template.git_repository = Some(git_repository);
    }
    if let Some(git_branch) = request.git_branch {
        template.git_branch = Some(git_branch);
    }
    if let Some(match_headers) = request.match_headers {
        template.match_headers = Some(match_headers);
    }
    if let Some(is_active) = request.is_active {
        template.is_active = is_active;
    }
    if let Some(custom_data) = request.custom_data {
        template.custom_data = custom_data;
    }

    // Update timestamp
    template.updated_at = chrono::Utc::now();

    // Validate template
    if let Err(e) = template.validate() {
        return Err(ServerError::BadRequest(e));
    }

    // Save to datastore
    let updated_template = state
        .datastore
        .update_template(&template)
        .await
        .map_err(|e| {
            error!("Failed to update template {}: {}", id, e);
            ServerError::Internal(format!("Failed to update template: {}", e))
        })?;

    info!(
        template_id = %updated_template.id,
        template_name = %updated_template.name,
        "Template updated successfully"
    );

    Ok(Json(TemplateResponse {
        template: updated_template,
    }))
}

/// Delete a template
pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<serde_json::Value>> {
    info!(template_id = %id, "Deleting template");

    // Check if template exists
    let template = state
        .datastore
        .get_template(&id)
        .await
        .map_err(|e| {
            error!("Failed to get template {}: {}", id, e);
            ServerError::Internal(format!("Failed to get template: {}", e))
        })?
        .ok_or_else(|| ServerError::NotFound(format!("Template with id {} not found", id)))?;

    // Check for active assignments
    let assignments = state
        .datastore
        .get_template_assignments_for_template(&id)
        .await
        .map_err(|e| {
            error!("Failed to get template assignments for {}: {}", id, e);
            ServerError::Internal(format!("Failed to check template assignments: {}", e))
        })?;

    if !assignments.is_empty() {
        warn!(
            template_id = %id,
            assignments_count = assignments.len(),
            "Attempted to delete template with active assignments"
        );
        return Err(ServerError::BadRequest(format!(
            "Cannot delete template with {} active assignments. Delete assignments first.",
            assignments.len()
        )));
    }

    // Delete template
    state.datastore.delete_template(&id).await.map_err(|e| {
        error!("Failed to delete template {}: {}", id, e);
        ServerError::Internal(format!("Failed to delete template: {}", e))
    })?;

    info!(
        template_id = %id,
        template_name = %template.name,
        "Template deleted successfully"
    );

    Ok(Json(serde_json::json!({
        "message": "Template deleted successfully",
        "template_id": id
    })))
}

/// Render a template for a specific node
pub async fn render_template(
    State(state): State<AppState>,
    Json(request): Json<TemplateRenderRequest>,
) -> ServerResult<Json<TemplateRenderResult>> {
    info!(
        template_id = %request.template_id,
        node_id = %request.node_id,
        "Rendering template"
    );

    let start_time = std::time::Instant::now();

    // Get template
    let template = state
        .datastore
        .get_template_required(&request.template_id)
        .await
        .map_err(|e| {
            error!("Failed to get template {}: {}", request.template_id, e);
            ServerError::Internal(format!("Failed to get template: {}", e))
        })?;

    // Check if template is active
    if !template.is_active {
        return Err(ServerError::BadRequest(
            "Template is not active".to_string(),
        ));
    }

    // Build template context
    let context = ContextBuilder::new(&*state.datastore, request.node_id)
        .with_variables(request.variables.clone())
        .build()
        .await
        .map_err(|e| {
            error!(
                "Failed to build template context for node {}: {}",
                request.node_id, e
            );
            ServerError::Internal(format!("Failed to build template context: {}", e))
        })?;

    // Create template renderer
    let renderer = TemplateRenderer::new().map_err(|e| {
        error!("Failed to create template renderer: {}", e);
        ServerError::Internal(format!("Failed to create template renderer: {}", e))
    })?;

    // Render template
    match renderer.render_template(&template.name, context).await {
        Ok(result) => {
            let render_time = start_time.elapsed().as_millis() as u64;

            // Record usage analytics
            let mut usage = TemplateUsage::new(
                request.template_id,
                "render".to_string(),
                "success".to_string(),
            );
            usage.node_id = Some(request.node_id);
            usage.render_time = Some(render_time as i32);
            usage.output_size = Some(result.output.len() as i32);

            // Store usage record (don't fail if this fails)
            if let Err(e) = state.datastore.record_template_usage(&usage).await {
                warn!("Failed to record template usage: {}", e);
            }

            info!(
                template_id = %request.template_id,
                node_id = %request.node_id,
                render_time_ms = render_time,
                output_size = result.output.len(),
                "Template rendered successfully"
            );

            Ok(Json(TemplateRenderResult::success(
                result.output,
                render_time,
                template.version,
            )))
        }
        Err(e) => {
            let render_time = start_time.elapsed().as_millis() as u64;
            let error_message = e.to_string();

            // Record usage analytics for error
            let mut usage = TemplateUsage::new(
                request.template_id,
                "render".to_string(),
                "error".to_string(),
            );
            usage.node_id = Some(request.node_id);
            usage.render_time = Some(render_time as i32);
            usage.error_message = Some(error_message.clone());

            // Store usage record (don't fail if this fails)
            if let Err(e) = state.datastore.record_template_usage(&usage).await {
                warn!("Failed to record template usage: {}", e);
            }

            error!(
                template_id = %request.template_id,
                node_id = %request.node_id,
                error = %error_message,
                "Template rendering failed"
            );

            Ok(Json(TemplateRenderResult::error(
                error_message,
                render_time,
            )))
        }
    }
}

/// Validate a template without rendering
pub async fn validate_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<serde_json::Value>> {
    info!(template_id = %id, "Validating template");

    // Get template
    let template = state
        .datastore
        .get_template_required(&id)
        .await
        .map_err(|e| {
            error!("Failed to get template {}: {}", id, e);
            ServerError::Internal(format!("Failed to get template: {}", e))
        })?;

    // Create template engine for validation
    let engine = unet_core::template::TemplateEngine::new().map_err(|e| {
        error!("Failed to create template engine: {}", e);
        ServerError::Internal(format!("Failed to create template engine: {}", e))
    })?;

    // Validate template syntax and security
    match engine.validate_template(&template.name).await {
        Ok(validation_result) => {
            info!(
                template_id = %id,
                template_name = %template.name,
                valid = validation_result.overall_valid,
                errors = validation_result.errors.len(),
                "Template validation completed"
            );

            Ok(Json(serde_json::json!({
                "template_id": id,
                "template_name": template.name,
                "valid": validation_result.overall_valid,
                "syntax_valid": validation_result.syntax_valid,
                "security_valid": validation_result.security_valid,
                "complexity": validation_result.complexity,
                "variables": validation_result.variables,
                "filters": validation_result.filters,
                "errors": validation_result.errors
            })))
        }
        Err(e) => {
            error!(
                template_id = %id,
                template_name = %template.name,
                error = %e,
                "Template validation failed"
            );

            Ok(Json(serde_json::json!({
                "template_id": id,
                "template_name": template.name,
                "valid": false,
                "syntax_valid": false,
                "security_valid": false,
                "complexity": "Low",
                "variables": [],
                "filters": [],
                "errors": [e.to_string()]
            })))
        }
    }
}

/// Get template usage analytics
pub async fn get_template_usage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<UsageQuery>,
) -> ServerResult<Json<TemplateUsageResponse>> {
    info!(template_id = %id, "Getting template usage analytics");

    // For now, return empty usage data since the usage operations aren't implemented yet
    let _ = (query, state);

    Ok(Json(TemplateUsageResponse {
        template_id: id,
        total_renders: 0,
        successful_renders: 0,
        failed_renders: 0,
        average_render_time: 0.0,
        usage_records: Vec::new(),
    }))
}

/// Query parameters for usage analytics
#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    /// Start date for usage data
    pub start_date: Option<String>,
    /// End date for usage data
    pub end_date: Option<String>,
    /// Limit number of usage records
    pub limit: Option<usize>,
}

/// Response for template usage analytics
#[derive(Debug, Serialize)]
pub struct TemplateUsageResponse {
    /// Template ID
    pub template_id: Uuid,
    /// Total number of renders
    pub total_renders: usize,
    /// Number of successful renders
    pub successful_renders: usize,
    /// Number of failed renders
    pub failed_renders: usize,
    /// Average render time in milliseconds
    pub average_render_time: f64,
    /// Recent usage records
    pub usage_records: Vec<TemplateUsage>,
}

// Template Assignment handlers

/// Request to create a template assignment
#[derive(Debug, Deserialize)]
pub struct CreateTemplateAssignmentRequest {
    /// Node ID this template is assigned to
    pub node_id: Uuid,
    /// Template ID being assigned
    pub template_id: Uuid,
    /// Type of assignment (e.g., "manual", "automatic", "policy")
    pub assignment_type: String,
    /// Priority for ordering (lower numbers = higher priority)
    pub priority: Option<i32>,
    /// Whether the assignment is active
    pub is_active: Option<bool>,
    /// Specific config section this applies to
    pub config_section: Option<String>,
    /// Template variables as JSON
    pub variables: Option<HashMap<String, serde_json::Value>>,
    /// Custom metadata as JSON
    pub custom_data: Option<serde_json::Value>,
}

/// Request to update a template assignment
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateAssignmentRequest {
    /// Type of assignment (e.g., "manual", "automatic", "policy")
    pub assignment_type: Option<String>,
    /// Priority for ordering (lower numbers = higher priority)
    pub priority: Option<i32>,
    /// Whether the assignment is active
    pub is_active: Option<bool>,
    /// Specific config section this applies to
    pub config_section: Option<String>,
    /// Template variables as JSON
    pub variables: Option<HashMap<String, serde_json::Value>>,
    /// Custom metadata as JSON
    pub custom_data: Option<serde_json::Value>,
}

/// Response for template assignment operations
#[derive(Debug, Serialize)]
pub struct TemplateAssignmentResponse {
    /// Assignment data
    pub assignment: TemplateAssignment,
}

/// Response for listing template assignments
#[derive(Debug, Serialize)]
pub struct ListTemplateAssignmentsResponse {
    /// List of assignments
    pub assignments: Vec<TemplateAssignment>,
    /// Total number of assignments available
    pub total_count: usize,
    /// Number of assignments returned
    pub returned_count: usize,
}

/// Create a template assignment
pub async fn create_template_assignment(
    State(state): State<AppState>,
    Json(request): Json<CreateTemplateAssignmentRequest>,
) -> ServerResult<Json<TemplateAssignmentResponse>> {
    info!(
        node_id = %request.node_id,
        template_id = %request.template_id,
        assignment_type = %request.assignment_type,
        "Creating template assignment"
    );

    // Validate required fields
    if request.assignment_type.trim().is_empty() {
        return Err(ServerError::BadRequest(
            "Assignment type cannot be empty".to_string(),
        ));
    }

    // Check that node exists
    if state.datastore.get_node(&request.node_id).await?.is_none() {
        return Err(ServerError::NotFound(format!(
            "Node with id {} not found",
            request.node_id
        )));
    }

    // Check that template exists
    if state
        .datastore
        .get_template(&request.template_id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound(format!(
            "Template with id {} not found",
            request.template_id
        )));
    }

    // Create assignment
    let mut assignment = TemplateAssignment::new(
        request.node_id,
        request.template_id,
        request.assignment_type.clone(),
    );

    // Set optional fields
    if let Some(priority) = request.priority {
        assignment.priority = priority;
    }
    assignment.is_active = request.is_active.unwrap_or(true);
    assignment.config_section = request.config_section;
    if let Some(variables) = request.variables {
        assignment.variables = Some(
            serde_json::to_string(&variables)
                .map_err(|e| ServerError::BadRequest(format!("Invalid variables JSON: {}", e)))?,
        );
    }
    assignment.custom_data = request.custom_data.unwrap_or(serde_json::Value::Null);

    // Validate assignment
    if let Err(e) = assignment.validate() {
        return Err(ServerError::BadRequest(e));
    }

    // Save to datastore
    let created_assignment = state
        .datastore
        .create_template_assignment(&assignment)
        .await
        .map_err(|e| {
            error!("Failed to create template assignment: {}", e);
            ServerError::Internal(format!("Failed to create template assignment: {}", e))
        })?;

    info!(
        assignment_id = %created_assignment.id,
        node_id = %created_assignment.node_id,
        template_id = %created_assignment.template_id,
        "Template assignment created successfully"
    );

    Ok(Json(TemplateAssignmentResponse {
        assignment: created_assignment,
    }))
}

/// Get template assignments for a node
pub async fn get_template_assignments_for_node(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
) -> ServerResult<Json<ListTemplateAssignmentsResponse>> {
    info!(node_id = %node_id, "Getting template assignments for node");

    let assignments = state
        .datastore
        .get_template_assignments_for_node(&node_id)
        .await
        .map_err(|e| {
            error!(
                "Failed to get template assignments for node {}: {}",
                node_id, e
            );
            ServerError::Internal(format!("Failed to get template assignments: {}", e))
        })?;

    info!(
        node_id = %node_id,
        assignments_count = assignments.len(),
        "Template assignments retrieved successfully"
    );

    Ok(Json(ListTemplateAssignmentsResponse {
        total_count: assignments.len(),
        returned_count: assignments.len(),
        assignments,
    }))
}

/// Get template assignments for a template
pub async fn get_template_assignments_for_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> ServerResult<Json<ListTemplateAssignmentsResponse>> {
    info!(template_id = %template_id, "Getting template assignments for template");

    let assignments = state
        .datastore
        .get_template_assignments_for_template(&template_id)
        .await
        .map_err(|e| {
            error!(
                "Failed to get template assignments for template {}: {}",
                template_id, e
            );
            ServerError::Internal(format!("Failed to get template assignments: {}", e))
        })?;

    info!(
        template_id = %template_id,
        assignments_count = assignments.len(),
        "Template assignments retrieved successfully"
    );

    Ok(Json(ListTemplateAssignmentsResponse {
        total_count: assignments.len(),
        returned_count: assignments.len(),
        assignments,
    }))
}

/// Update a template assignment
pub async fn update_template_assignment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTemplateAssignmentRequest>,
) -> ServerResult<Json<TemplateAssignmentResponse>> {
    info!(assignment_id = %id, "Updating template assignment");

    // Get existing assignment
    let mut assignment = state
        .datastore
        .get_template_assignment(&id)
        .await
        .map_err(|e| {
            error!("Failed to get template assignment {}: {}", id, e);
            ServerError::Internal(format!("Failed to get template assignment: {}", e))
        })?
        .ok_or_else(|| {
            ServerError::NotFound(format!("TemplateAssignment with id {} not found", id))
        })?;

    // Update fields if provided
    if let Some(assignment_type) = request.assignment_type {
        assignment.assignment_type = assignment_type;
    }
    if let Some(priority) = request.priority {
        assignment.priority = priority;
    }
    if let Some(is_active) = request.is_active {
        assignment.is_active = is_active;
    }
    if let Some(config_section) = request.config_section {
        assignment.config_section = Some(config_section);
    }
    if let Some(variables) = request.variables {
        assignment.variables = Some(
            serde_json::to_string(&variables)
                .map_err(|e| ServerError::BadRequest(format!("Invalid variables JSON: {}", e)))?,
        );
    }
    if let Some(custom_data) = request.custom_data {
        assignment.custom_data = custom_data;
    }

    // Update timestamp
    assignment.updated_at = chrono::Utc::now();

    // Validate assignment
    if let Err(e) = assignment.validate() {
        return Err(ServerError::BadRequest(e));
    }

    // Save to datastore
    let updated_assignment = state
        .datastore
        .update_template_assignment(&assignment)
        .await
        .map_err(|e| {
            error!("Failed to update template assignment {}: {}", id, e);
            ServerError::Internal(format!("Failed to update template assignment: {}", e))
        })?;

    info!(
        assignment_id = %updated_assignment.id,
        node_id = %updated_assignment.node_id,
        template_id = %updated_assignment.template_id,
        "Template assignment updated successfully"
    );

    Ok(Json(TemplateAssignmentResponse {
        assignment: updated_assignment,
    }))
}

/// Delete a template assignment
pub async fn delete_template_assignment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<serde_json::Value>> {
    info!(assignment_id = %id, "Deleting template assignment");

    // Check if assignment exists
    let assignment = state
        .datastore
        .get_template_assignment(&id)
        .await
        .map_err(|e| {
            error!("Failed to get template assignment {}: {}", id, e);
            ServerError::Internal(format!("Failed to get template assignment: {}", e))
        })?
        .ok_or_else(|| {
            ServerError::NotFound(format!("TemplateAssignment with id {} not found", id))
        })?;

    // Delete assignment
    state
        .datastore
        .delete_template_assignment(&id)
        .await
        .map_err(|e| {
            error!("Failed to delete template assignment {}: {}", id, e);
            ServerError::Internal(format!("Failed to delete template assignment: {}", e))
        })?;

    info!(
        assignment_id = %id,
        node_id = %assignment.node_id,
        template_id = %assignment.template_id,
        "Template assignment deleted successfully"
    );

    Ok(Json(serde_json::json!({
        "message": "Template assignment deleted successfully",
        "assignment_id": id
    })))
}
