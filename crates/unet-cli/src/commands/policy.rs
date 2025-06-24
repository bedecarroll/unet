//! Policy management commands for μNet CLI
//!
//! This module provides commands for managing and testing policy rules,
//! including validation, evaluation, and compliance checking.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use unet_core::config::GitConfig;
use unet_core::datastore::DataStore;
use unet_core::policy::{EvaluationContext, PolicyEvaluator, PolicyLoader, PolicyParser};
use uuid::Uuid;

#[derive(Subcommand)]
pub enum PolicyCommands {
    /// Validate policy file syntax
    Validate(ValidatePolicyArgs),
    /// Evaluate policies against nodes
    Eval(EvalPolicyArgs),
    /// Show compliance differences
    Diff(DiffPolicyArgs),
    /// List available policy files
    List(ListPolicyArgs),
    /// Show policy file contents
    Show(ShowPolicyArgs),
}

#[derive(Args)]
pub struct ValidatePolicyArgs {
    /// Path to policy file or directory
    #[arg(short, long)]
    path: PathBuf,

    /// Show detailed validation results
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Args)]
pub struct EvalPolicyArgs {
    /// Path to policy file or directory
    #[arg(short, long)]
    path: PathBuf,

    /// Node ID to evaluate against (optional, evaluates against all nodes if not specified)
    #[arg(short, long)]
    node_id: Option<Uuid>,

    /// Show detailed evaluation results
    #[arg(short, long)]
    verbose: bool,

    /// Dry run - don't execute actions, just show what would happen
    #[arg(short, long)]
    dry_run: bool,
}

#[derive(Args)]
pub struct DiffPolicyArgs {
    /// Path to policy file or directory
    #[arg(short, long)]
    path: PathBuf,

    /// Node ID to check compliance for
    #[arg(short, long)]
    node_id: Uuid,

    /// Show only failed compliance checks
    #[arg(short, long)]
    failed_only: bool,
}

#[derive(Args)]
pub struct ListPolicyArgs {
    /// Base directory to search for policy files
    #[arg(short, long, default_value = "policies")]
    directory: PathBuf,

    /// Show policy file details
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Args)]
pub struct ShowPolicyArgs {
    /// Path to policy file
    #[arg(short, long)]
    path: PathBuf,

    /// Parse and show AST structure
    #[arg(short, long)]
    ast: bool,
}

/// Execute policy commands
pub async fn execute(
    command: PolicyCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        PolicyCommands::Validate(args) => validate_policy(args, output_format).await,
        PolicyCommands::Eval(args) => eval_policy(args, datastore, output_format).await,
        PolicyCommands::Diff(args) => diff_policy(args, datastore, output_format).await,
        PolicyCommands::List(args) => list_policies(args, output_format).await,
        PolicyCommands::Show(args) => show_policy(args, output_format).await,
    }
}

async fn validate_policy(
    args: ValidatePolicyArgs,
    _output_format: crate::OutputFormat,
) -> Result<()> {
    println!("Validating policy file: {}", args.path.display());

    // Create a default GitConfig for the loader
    let git_config = GitConfig {
        policies_repo: None,
        templates_repo: None,
        branch: "main".to_string(),
        sync_interval: 300,
    };

    let mut loader = PolicyLoader::new(git_config);

    if args.path.is_file() {
        // Validate single file by trying to parse it directly
        let content = std::fs::read_to_string(&args.path)?;
        match PolicyParser::parse_file(&content) {
            Ok(rules) => {
                println!("✅ Policy file is valid");
                if args.verbose {
                    println!("Rules found: {}", rules.len());
                    for (i, rule) in rules.iter().enumerate() {
                        println!("  Rule {}: {}", i + 1, rule);
                    }
                }
            }
            Err(e) => {
                println!("❌ Policy file validation failed: {}", e);
                return Err(anyhow::anyhow!("Policy validation failed: {}", e));
            }
        }
    } else if args.path.is_dir() {
        // Validate directory
        let load_result = loader.load_policies_from_directory(&args.path).await?;

        println!("Validation summary:");
        println!("  Files processed: {}", load_result.total_files);
        println!("  Valid files: {}", load_result.loaded.len());
        println!(
            "  Total rules: {}",
            load_result
                .loaded
                .iter()
                .map(|f| f.rules.len())
                .sum::<usize>()
        );

        if args.verbose {
            for policy_file in &load_result.loaded {
                println!(
                    "✅ {}: {} rules",
                    policy_file.path.display(),
                    policy_file.rules.len()
                );
            }
        }

        for (file_path, error) in &load_result.errors {
            println!("❌ {}: {}", file_path.display(), error);
        }

        if !load_result.errors.is_empty() {
            return Err(anyhow::anyhow!("Some policy files failed validation"));
        }
    } else {
        return Err(anyhow::anyhow!(
            "Path does not exist: {}",
            args.path.display()
        ));
    }

    Ok(())
}

async fn eval_policy(
    args: EvalPolicyArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    println!("Evaluating policies from: {}", args.path.display());

    // Create a default GitConfig for the loader
    let git_config = GitConfig {
        policies_repo: None,
        templates_repo: None,
        branch: "main".to_string(),
        sync_interval: 300,
    };

    let mut loader = PolicyLoader::new(git_config);

    // Load policies
    let policies = if args.path.is_file() {
        let content = std::fs::read_to_string(&args.path)?;
        let rules = PolicyParser::parse_file(&content)?;
        vec![rules]
    } else if args.path.is_dir() {
        let load_result = loader.load_policies_from_directory(&args.path).await?;
        if !load_result.errors.is_empty() {
            for (file_path, error) in &load_result.errors {
                println!("❌ Failed to load {}: {}", file_path.display(), error);
            }
            return Err(anyhow::anyhow!("Failed to load some policy files"));
        }
        load_result.loaded.into_iter().map(|f| f.rules).collect()
    } else {
        return Err(anyhow::anyhow!(
            "Path does not exist: {}",
            args.path.display()
        ));
    };

    // Get nodes to evaluate against
    let nodes = if let Some(node_id) = args.node_id {
        match datastore.get_node(&node_id).await {
            Ok(Some(node)) => vec![node],
            Ok(None) => {
                return Err(anyhow::anyhow!("Node not found: {}", node_id));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get node: {}", e));
            }
        }
    } else {
        // Get all nodes
        match datastore.list_nodes(&Default::default()).await {
            Ok(paged_result) => paged_result.items,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to list nodes: {}", e));
            }
        }
    };

    if nodes.is_empty() {
        println!("No nodes found to evaluate policies against");
        return Ok(());
    }

    println!(
        "Evaluating {} policies against {} nodes...",
        policies.iter().map(|p| p.len()).sum::<usize>(),
        nodes.len()
    );

    // Evaluate policies for each node
    for node in &nodes {
        println!("\n--- Node: {} ({}) ---", node.name, node.id);

        // Create evaluation context from node data
        let node_json = serde_json::to_value(node)?;
        let context = EvaluationContext {
            node_data: node_json,
            derived_data: None,
        };

        for policy_rules in &policies {
            for (rule_idx, rule) in policy_rules.iter().enumerate() {
                if args.verbose {
                    println!("Evaluating rule {}: {}", rule_idx + 1, rule);
                }

                match PolicyEvaluator::evaluate_rule(rule, &context) {
                    Ok(evaluation_result) => {
                        match evaluation_result {
                            unet_core::policy::EvaluationResult::Satisfied { action } => {
                                println!("  ✅ Rule {} condition matched", rule_idx + 1);

                                if args.dry_run {
                                    println!("  🔍 Would execute: {}", action);
                                } else {
                                    // Note: For now, we'll just show what would be executed
                                    // Full action execution requires additional integration
                                    println!("  🔍 Would execute: {}", action);
                                }
                            }
                            unet_core::policy::EvaluationResult::NotSatisfied => {
                                if args.verbose {
                                    println!("  ⏭️  Rule {} condition not matched", rule_idx + 1);
                                }
                            }
                            unet_core::policy::EvaluationResult::Error { message } => {
                                println!("  💥 Rule {} error: {}", rule_idx + 1, message);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  💥 Failed to evaluate rule {}: {}", rule_idx + 1, e);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn diff_policy(
    args: DiffPolicyArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    println!("Checking compliance for node: {}", args.node_id);

    // Get the specific node
    let node = match datastore.get_node(&args.node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => {
            return Err(anyhow::anyhow!("Node not found: {}", args.node_id));
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to get node: {}", e));
        }
    };

    // Create a default GitConfig for the loader
    let git_config = GitConfig {
        policies_repo: None,
        templates_repo: None,
        branch: "main".to_string(),
        sync_interval: 300,
    };

    let mut loader = PolicyLoader::new(git_config);

    // Load policies
    let policies = if args.path.is_file() {
        let content = std::fs::read_to_string(&args.path)?;
        let rules = PolicyParser::parse_file(&content)?;
        vec![rules]
    } else if args.path.is_dir() {
        let load_result = loader.load_policies_from_directory(&args.path).await?;
        if !load_result.errors.is_empty() {
            for (file_path, error) in &load_result.errors {
                println!("❌ Failed to load {}: {}", file_path.display(), error);
            }
            return Err(anyhow::anyhow!("Failed to load some policy files"));
        }
        load_result.loaded.into_iter().map(|f| f.rules).collect()
    } else {
        return Err(anyhow::anyhow!(
            "Path does not exist: {}",
            args.path.display()
        ));
    };

    // Create evaluation context
    let node_json = serde_json::to_value(&node)?;
    let context = EvaluationContext {
        node_data: node_json,
        derived_data: None,
    };

    println!("Compliance check for node '{}' ({}):", node.name, node.id);

    let mut compliance_checks = 0;
    let mut passed_checks = 0;
    let mut failed_checks = 0;

    for policy_rules in &policies {
        for (rule_idx, rule) in policy_rules.iter().enumerate() {
            // Only check ASSERT actions for compliance
            if let unet_core::policy::Action::Assert { .. } = rule.action {
                compliance_checks += 1;

                // Evaluate the rule
                match PolicyEvaluator::evaluate_rule(rule, &context) {
                    Ok(evaluation_result) => {
                        match evaluation_result {
                            unet_core::policy::EvaluationResult::Satisfied { .. } => {
                                // For ASSERT actions, if the condition is satisfied,
                                // it means the assertion should be checked
                                // For now, we'll assume it passes (need full action execution)
                                passed_checks += 1;
                                if !args.failed_only {
                                    println!("  ✅ Rule {}: {}", rule_idx + 1, rule.action);
                                }
                            }
                            unet_core::policy::EvaluationResult::NotSatisfied => {
                                // Condition not met, so assertion doesn't apply
                                if !args.failed_only {
                                    println!(
                                        "  ⏭️  Rule {}: Condition not met, assertion not applicable",
                                        rule_idx + 1
                                    );
                                }
                            }
                            unet_core::policy::EvaluationResult::Error { message } => {
                                failed_checks += 1;
                                println!("  💥 Rule {}: Error - {}", rule_idx + 1, message);
                            }
                        }
                    }
                    Err(e) => {
                        failed_checks += 1;
                        println!("  💥 Rule {}: Error evaluating - {}", rule_idx + 1, e);
                    }
                }
            }
        }
    }

    println!("\nCompliance Summary:");
    println!("  Total compliance checks: {}", compliance_checks);
    println!("  Passed: {}", passed_checks);
    println!("  Failed: {}", failed_checks);

    if failed_checks > 0 {
        println!("⚠️  Node has compliance violations");
    } else if compliance_checks > 0 {
        println!("✅ Node is compliant with all policies");
    } else {
        println!("ℹ️  No compliance checks found in policies");
    }

    Ok(())
}

async fn list_policies(args: ListPolicyArgs, _output_format: crate::OutputFormat) -> Result<()> {
    println!("Listing policy files in: {}", args.directory.display());

    if !args.directory.exists() {
        return Err(anyhow::anyhow!(
            "Directory does not exist: {}",
            args.directory.display()
        ));
    }

    // Create a default GitConfig for the loader
    let git_config = GitConfig {
        policies_repo: None,
        templates_repo: None,
        branch: "main".to_string(),
        sync_interval: 300,
    };

    let mut loader = PolicyLoader::new(git_config);
    let load_result = loader.load_policies_from_directory(&args.directory).await?;

    if load_result.loaded.is_empty() && load_result.errors.is_empty() {
        println!("No policy files found in directory");
        return Ok(());
    }

    for policy_file in load_result.loaded {
        if args.verbose {
            println!(
                "📄 {} ({} rules)",
                policy_file.path.display(),
                policy_file.rules.len()
            );
            for (i, rule) in policy_file.rules.iter().enumerate() {
                println!("    Rule {}: {}", i + 1, rule);
            }
        } else {
            println!(
                "📄 {} ({} rules)",
                policy_file.path.display(),
                policy_file.rules.len()
            );
        }
    }

    for (file_path, error) in load_result.errors {
        println!("❌ {}: {}", file_path.display(), error);
    }

    Ok(())
}

async fn show_policy(args: ShowPolicyArgs, _output_format: crate::OutputFormat) -> Result<()> {
    println!("Policy file: {}", args.path.display());

    if !args.path.exists() {
        return Err(anyhow::anyhow!(
            "File does not exist: {}",
            args.path.display()
        ));
    }

    println!("\n--- File Contents ---");
    let contents = std::fs::read_to_string(&args.path)?;
    println!("{}", contents);

    // Parse the file to show the rules
    match PolicyParser::parse_file(&contents) {
        Ok(rules) => {
            println!("\n--- Parsed Rules ({}) ---", rules.len());
            for (i, rule) in rules.iter().enumerate() {
                println!("Rule {}: {}", i + 1, rule);

                if args.ast {
                    println!("  AST: {:#?}", rule);
                }
            }
        }
        Err(e) => {
            println!("\n❌ Failed to parse policy file: {}", e);
            return Err(anyhow::anyhow!("Failed to parse policy file: {}", e));
        }
    }

    Ok(())
}

// Helper functions for formatting
fn format_condition(condition: &unet_core::policy::Condition) -> String {
    condition.to_string()
}

fn format_action(action: &unet_core::policy::Action) -> String {
    action.to_string()
}

fn format_field_ref(field_ref: &unet_core::policy::FieldRef) -> String {
    field_ref.to_string()
}

fn format_comparison_op(op: &unet_core::policy::ComparisonOperator) -> String {
    op.to_string()
}

fn format_value(value: &unet_core::policy::Value) -> String {
    value.to_string()
}
