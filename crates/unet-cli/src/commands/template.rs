use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use unet_core::datastore::DataStore;
use unet_core::models::template::TemplateAssignment;
use unet_core::template::{ContextBuilder, RenderOptions, TemplateEngine, TemplateRenderer};
use uuid::Uuid;

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// Render a template for a specific node
    Render(RenderTemplateArgs),
    /// Validate template syntax and security
    Validate(ValidateTemplateArgs),
    /// Assign a template to a node
    Assign(AssignTemplateArgs),
    /// List template assignments for a node
    Assignments(ListAssignmentsArgs),
    /// Remove a template assignment
    Unassign(UnassignTemplateArgs),
    /// Debug template rendering with detailed output
    Debug(DebugTemplateArgs),
}

#[derive(Args)]
pub struct RenderTemplateArgs {
    /// Template ID to render
    #[arg(short, long)]
    template_id: Uuid,

    /// Node ID to render template for
    #[arg(short, long)]
    node_id: Uuid,

    /// Additional template variables as JSON
    #[arg(short = 'V', long)]
    variables: Option<String>,

    /// Specific config section to render
    #[arg(short, long)]
    section: Option<String>,

    /// Disable output caching
    #[arg(long)]
    no_cache: bool,

    /// Disable vendor-specific formatting
    #[arg(long)]
    no_format: bool,

    /// Include template validation warnings in output
    #[arg(long)]
    show_warnings: bool,
}

#[derive(Args)]
pub struct ValidateTemplateArgs {
    /// Template ID to validate
    #[arg(short, long)]
    template_id: Uuid,

    /// Validate with a specific node context (optional)
    #[arg(short, long)]
    node_id: Option<Uuid>,

    /// Show detailed validation information
    #[arg(long)]
    detailed: bool,

    /// Check template security constraints
    #[arg(long)]
    security: bool,

    /// Validate template syntax only
    #[arg(long)]
    syntax_only: bool,
}

#[derive(Args)]
pub struct AssignTemplateArgs {
    /// Template ID to assign
    #[arg(short, long)]
    template_id: Uuid,

    /// Node ID to assign template to
    #[arg(short, long)]
    node_id: Uuid,

    /// Assignment type (manual, automatic, policy)
    #[arg(short = 'T', long, default_value = "manual")]
    assignment_type: String,

    /// Priority for template ordering (lower = higher priority)
    #[arg(short, long, default_value = "100")]
    priority: i32,

    /// Specific config section this applies to
    #[arg(short, long)]
    section: Option<String>,

    /// Template variables as JSON
    #[arg(short = 'V', long)]
    variables: Option<String>,

    /// Force assignment even if template is not active
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
pub struct ListAssignmentsArgs {
    /// Node ID to list assignments for
    #[arg(short, long)]
    node_id: Uuid,

    /// Show only active assignments
    #[arg(long)]
    active_only: bool,

    /// Show template details in output
    #[arg(long)]
    show_templates: bool,

    /// Filter by assignment type
    #[arg(short = 'T', long)]
    assignment_type: Option<String>,
}

#[derive(Args)]
pub struct UnassignTemplateArgs {
    /// Assignment ID to remove
    #[arg(short, long)]
    assignment_id: Uuid,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
pub struct DebugTemplateArgs {
    /// Template ID to debug
    #[arg(short, long)]
    template_id: Uuid,

    /// Node ID to debug template for
    #[arg(short, long)]
    node_id: Uuid,

    /// Show template context data
    #[arg(long)]
    show_context: bool,

    /// Show template validation details
    #[arg(long)]
    show_validation: bool,

    /// Show render timing information
    #[arg(long)]
    show_timing: bool,

    /// Show cache statistics
    #[arg(long)]
    show_cache: bool,

    /// Additional template variables as JSON
    #[arg(short = 'V', long)]
    variables: Option<String>,
}

pub async fn execute(
    command: TemplateCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        TemplateCommands::Render(args) => render_template(args, datastore, output_format).await,
        TemplateCommands::Validate(args) => validate_template(args, datastore, output_format).await,
        TemplateCommands::Assign(args) => assign_template(args, datastore, output_format).await,
        TemplateCommands::Assignments(args) => {
            list_assignments(args, datastore, output_format).await
        }
        TemplateCommands::Unassign(args) => unassign_template(args, datastore, output_format).await,
        TemplateCommands::Debug(args) => debug_template(args, datastore, output_format).await,
    }
}

async fn render_template(
    args: RenderTemplateArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify template exists (stub implementation - template CRUD not yet in DataStore)
    // TODO: Replace with actual template fetching when DataStore template methods are implemented
    let template_info = serde_json::json!({
        "template_id": args.template_id,
        "note": "Template storage in DataStore not yet implemented",
        "status": "Using template engine directly"
    });

    // Verify node exists
    let node = datastore
        .get_node_required(&args.node_id)
        .await
        .with_context(|| format!("Node {} not found", args.node_id))?;

    // Parse additional variables if provided
    let additional_variables = if let Some(vars_json) = args.variables {
        serde_json::from_str::<HashMap<String, JsonValue>>(&vars_json)
            .with_context(|| "Failed to parse variables JSON")?
    } else {
        HashMap::new()
    };

    // Build template context
    let context = ContextBuilder::new(datastore, args.node_id)
        .with_variables(additional_variables)
        .build()
        .await
        .with_context(|| "Failed to build template context")?;

    // Configure render options
    let render_options = RenderOptions {
        use_cache: !args.no_cache,
        format_output: !args.no_format,
        custom_processors: Vec::new(),
    };

    // Create template renderer
    let renderer = TemplateRenderer::new().with_context(|| "Failed to create template renderer")?;

    // For now, create a mock template name since we don't have template storage yet
    let template_name = format!("template_{}", args.template_id);

    // Render template
    match renderer
        .render_template_with_options(&template_name, context, render_options)
        .await
    {
        Ok(result) => {
            let mut output = serde_json::json!({
                "template_id": args.template_id,
                "node_id": args.node_id,
                "node_name": node.name,
                "template_name": template_name,
                "rendered_output": result.output,
                "render_time_ms": result.duration.as_millis(),
                "output_size": result.output.len(),
                "is_valid": result.is_valid(),
                "template_info": template_info
            });

            if args.show_warnings && result.has_warnings() {
                output["warnings"] = serde_json::to_value(&result.warnings)?;
            }

            if let Some(validation_result) = &result.validation_result {
                output["validation"] = serde_json::json!({
                    "syntax_valid": validation_result.syntax_valid,
                    "semantic_valid": validation_result.semantic_valid,
                    "errors": validation_result.errors,
                    "warnings": validation_result.warnings
                });
            }

            if let Some(section) = args.section {
                output["config_section"] = serde_json::Value::String(section);
            }

            crate::commands::print_output(&output, output_format)?;
        }
        Err(e) => {
            let error_output = serde_json::json!({
                "template_id": args.template_id,
                "node_id": args.node_id,
                "error": "Template rendering failed",
                "details": e.to_string(),
                "template_info": template_info,
                "note": "This is expected until template storage and loading is implemented"
            });

            crate::commands::print_output(&error_output, output_format)?;
        }
    }

    Ok(())
}

async fn validate_template(
    args: ValidateTemplateArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Create template renderer for validation
    let renderer = TemplateRenderer::new().with_context(|| "Failed to create template renderer")?;

    let mut output = serde_json::json!({
        "template_id": args.template_id,
        "validation_type": if args.syntax_only { "syntax_only" } else { "comprehensive" },
        "security_check": args.security,
        "detailed": args.detailed
    });

    // If node context is provided, validate with node data
    if let Some(node_id) = args.node_id {
        let node = datastore
            .get_node_required(&node_id)
            .await
            .with_context(|| format!("Node {} not found", node_id))?;

        output["node_id"] = serde_json::Value::String(node_id.to_string());
        output["node_name"] = serde_json::Value::String(node.name.clone());

        // Build context for validation
        let context = ContextBuilder::new(datastore, node_id)
            .build()
            .await
            .with_context(|| "Failed to build template context")?;

        output["context_validation"] = serde_json::json!({
            "status": "success",
            "has_derived_state": context.node_status.is_some(),
            "links_count": context.links.len(),
            "locations_count": context.locations.len(),
            "variables_count": context.variables.len()
        });
    }

    // Mock template validation since template loading is not yet implemented
    let template_name = format!("template_{}", args.template_id);

    // Create template engine for validation
    let engine = TemplateEngine::new().with_context(|| "Failed to create template engine")?;

    // Attempt template validation
    match engine.validate_template(&template_name).await {
        Ok(validation_result) => {
            output["validation_result"] = serde_json::json!({
                "overall_valid": validation_result.overall_valid,
                "syntax_valid": validation_result.syntax_valid,
                "security_valid": validation_result.security_valid,
                "errors": validation_result.errors,
                "complexity": validation_result.complexity
            });

            if args.detailed {
                output["detailed_analysis"] = serde_json::json!({
                    "variables_used": validation_result.variables,
                    "filters_used": validation_result.filters,
                    "variable_count": validation_result.variables.len(),
                    "filter_count": validation_result.filters.len()
                });
            }
        }
        Err(e) => {
            output["validation_result"] = serde_json::json!({
                "overall_valid": false,
                "error": e.to_string(),
                "note": "Template validation failed - this is expected until template storage is implemented"
            });
        }
    }

    // Add implementation status
    output["implementation_status"] = serde_json::json!({
        "template_storage": "not_implemented",
        "template_loading": "not_implemented",
        "validation_framework": "implemented",
        "note": "Template CRUD operations in DataStore are required for full functionality"
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn assign_template(
    args: AssignTemplateArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists
    let node = datastore
        .get_node_required(&args.node_id)
        .await
        .with_context(|| format!("Node {} not found", args.node_id))?;

    // Parse template variables if provided
    let variables_json = if let Some(vars_str) = args.variables {
        Some(
            serde_json::from_str::<HashMap<String, JsonValue>>(&vars_str)
                .with_context(|| "Failed to parse variables JSON")?,
        )
    } else {
        None
    };

    // Create template assignment
    let mut assignment =
        TemplateAssignment::new(args.node_id, args.template_id, args.assignment_type.clone());

    assignment.priority = args.priority;
    assignment.config_section = args.section.clone();

    if let Some(vars) = variables_json {
        assignment.variables = Some(serde_json::to_string(&vars)?);
    }

    // Validate assignment
    assignment
        .validate()
        .map_err(|e| anyhow!("Template assignment validation failed: {}", e))?;

    let output = serde_json::json!({
        "assignment_id": assignment.id,
        "template_id": args.template_id,
        "node_id": args.node_id,
        "node_name": node.name,
        "assignment_type": assignment.assignment_type,
        "priority": assignment.priority,
        "config_section": assignment.config_section,
        "is_active": assignment.is_active,
        "created_at": assignment.created_at,
        "status": "assignment_created",
        "note": "Template assignment storage in DataStore not yet implemented - assignment created in memory only"
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn list_assignments(
    args: ListAssignmentsArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists
    let node = datastore
        .get_node_required(&args.node_id)
        .await
        .with_context(|| format!("Node {} not found", args.node_id))?;

    // Mock assignments since DataStore template methods are not yet implemented
    let mock_assignments = vec![
        serde_json::json!({
            "assignment_id": Uuid::new_v4(),
            "template_id": Uuid::new_v4(),
            "assignment_type": "manual",
            "priority": 50,
            "config_section": "interface",
            "is_active": true,
            "created_at": chrono::Utc::now(),
            "variables": {"interface_type": "GigabitEthernet"}
        }),
        serde_json::json!({
            "assignment_id": Uuid::new_v4(),
            "template_id": Uuid::new_v4(),
            "assignment_type": "automatic",
            "priority": 100,
            "config_section": "routing",
            "is_active": true,
            "created_at": chrono::Utc::now(),
            "variables": null
        }),
    ];

    let filtered_assignments: Vec<_> = mock_assignments
        .into_iter()
        .filter(|assignment| {
            if args.active_only && assignment["is_active"] != true {
                return false;
            }
            if let Some(ref filter_type) = args.assignment_type {
                if assignment["assignment_type"] != serde_json::Value::String(filter_type.clone()) {
                    return false;
                }
            }
            true
        })
        .collect();

    let output = serde_json::json!({
        "node_id": args.node_id,
        "node_name": node.name,
        "assignments": filtered_assignments,
        "total_assignments": filtered_assignments.len(),
        "filters": {
            "active_only": args.active_only,
            "assignment_type": args.assignment_type,
            "show_templates": args.show_templates
        },
        "implementation_status": {
            "template_assignments": "not_implemented_in_datastore",
            "note": "This shows mock data until template assignment storage is implemented"
        }
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn unassign_template(
    args: UnassignTemplateArgs,
    _datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    if !args.yes {
        // Ask for confirmation
        println!(
            "Are you sure you want to remove template assignment {}? [y/N]",
            args.assignment_id
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            let output = serde_json::json!({
                "assignment_id": args.assignment_id,
                "status": "cancelled",
                "message": "Template assignment removal cancelled by user"
            });
            crate::commands::print_output(&output, output_format)?;
            return Ok(());
        }
    }

    let output = serde_json::json!({
        "assignment_id": args.assignment_id,
        "status": "removed",
        "message": "Template assignment would be removed",
        "note": "Template assignment storage in DataStore not yet implemented - operation simulated only"
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn debug_template(
    args: DebugTemplateArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists
    let node = datastore
        .get_node_required(&args.node_id)
        .await
        .with_context(|| format!("Node {} not found", args.node_id))?;

    // Parse additional variables if provided
    let additional_variables = if let Some(vars_json) = args.variables {
        serde_json::from_str::<HashMap<String, JsonValue>>(&vars_json)
            .with_context(|| "Failed to parse variables JSON")?
    } else {
        HashMap::new()
    };

    let mut output = serde_json::json!({
        "template_id": args.template_id,
        "node_id": args.node_id,
        "node_name": node.name,
        "debug_options": {
            "show_context": args.show_context,
            "show_validation": args.show_validation,
            "show_timing": args.show_timing,
            "show_cache": args.show_cache
        }
    });

    // Build template context
    let context_start = std::time::Instant::now();
    let context = ContextBuilder::new(datastore, args.node_id)
        .with_variables(additional_variables.clone())
        .build()
        .await
        .with_context(|| "Failed to build template context")?;
    let context_time = context_start.elapsed();

    if args.show_timing {
        output["timing"] = serde_json::json!({
            "context_build_ms": context_time.as_millis(),
            "note": "Additional timing info would be shown here"
        });
    }

    if args.show_context {
        output["template_context"] = serde_json::json!({
            "node": {
                "id": context.node.id,
                "name": context.node.name,
                "vendor": context.node.vendor,
                "role": context.node.role,
                "fqdn": context.node.fqdn
            },
            "has_derived_state": context.node_status.is_some(),
            "links_count": context.links.len(),
            "locations_count": context.locations.len(),
            "variables_count": context.variables.len(),
            "generated_at": context.generated_at,
            "custom_variables": additional_variables
        });

        if let Some(ref status) = context.node_status {
            output["template_context"]["derived_state"] = serde_json::json!({
                "reachable": status.reachable,
                "interfaces_count": status.interfaces.len(),
                "last_updated": status.last_updated
            });
        }
    }

    // Create template renderer for debugging
    let renderer = TemplateRenderer::new().with_context(|| "Failed to create template renderer")?;

    if args.show_cache {
        let cache_stats = renderer.cache_stats().await;
        output["cache_stats"] = serde_json::json!({
            "total_entries": cache_stats.total_entries,
            "max_entries": cache_stats.max_entries,
            "ttl_seconds": cache_stats.ttl_seconds
        });
    }

    if args.show_validation {
        let template_name = format!("template_{}", args.template_id);
        let engine = TemplateEngine::new().with_context(|| "Failed to create template engine")?;
        match engine.validate_template(&template_name).await {
            Ok(validation_result) => {
                output["validation_debug"] = serde_json::json!({
                    "overall_valid": validation_result.overall_valid,
                    "syntax_valid": validation_result.syntax_valid,
                    "security_valid": validation_result.security_valid,
                    "complexity": validation_result.complexity,
                    "errors": validation_result.errors,
                    "variables_used": validation_result.variables,
                    "filters_used": validation_result.filters,
                    "variable_count": validation_result.variables.len(),
                    "filter_count": validation_result.filters.len()
                });
            }
            Err(e) => {
                output["validation_debug"] = serde_json::json!({
                    "error": e.to_string(),
                    "note": "Template validation failed - expected until template storage is implemented"
                });
            }
        }
    }

    // Implementation status
    output["implementation_status"] = serde_json::json!({
        "template_engine": "implemented",
        "context_builder": "implemented",
        "template_renderer": "implemented",
        "template_storage": "not_implemented",
        "template_loading": "not_implemented",
        "note": "Debug functionality is working but requires template storage implementation for full features"
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}
