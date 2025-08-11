/// Policy management commands for μNet CLI
///
/// This module provides commands for managing and testing policy rules,
/// including validation, evaluation, and compliance checking.
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use unet_core::datastore::DataStore;
use uuid::Uuid;

pub mod eval;
pub mod helpers;
pub mod list;
pub mod validate;

#[derive(Subcommand, Debug)]
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

#[derive(Args, Debug)]
pub struct ValidatePolicyArgs {
    /// Path to policy file or directory
    #[arg(short, long)]
    pub path: PathBuf,

    /// Show detailed validation results
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct EvalPolicyArgs {
    /// Path to policy file or directory
    #[arg(short, long)]
    pub path: PathBuf,

    /// Node ID to evaluate against (optional, evaluates against all nodes if not specified)
    #[arg(short, long)]
    pub node_id: Option<Uuid>,

    /// Show detailed evaluation results
    #[arg(short, long)]
    pub verbose: bool,

    /// Only show rules that failed evaluation
    #[arg(short, long)]
    pub failures_only: bool,
}

#[derive(Args, Debug)]
pub struct DiffPolicyArgs {
    /// Path to policy file or directory
    #[arg(short, long)]
    pub path: PathBuf,

    /// Node ID to check compliance for
    #[arg(short, long)]
    pub node_id: Uuid,

    /// Show detailed differences
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct ListPolicyArgs {
    /// Path to policy directory
    #[arg(short, long)]
    pub path: PathBuf,

    /// Show detailed information about each policy file
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct ShowPolicyArgs {
    /// Path to policy file
    #[arg(short, long)]
    pub path: PathBuf,

    /// Parse and show AST structure
    #[arg(short, long)]
    pub ast: bool,
}

/// Execute policy commands
/// Execute top-level policy commands.
///
/// # Errors
/// Returns an error if policy parsing or datastore operations fail.
pub async fn execute(command: PolicyCommands, datastore: &dyn DataStore) -> Result<()> {
    match command {
        PolicyCommands::Validate(args) => validate::validate_policy(&args),
        PolicyCommands::Eval(args) => eval::eval_policy(args, datastore).await,
        PolicyCommands::Diff(args) => eval::diff_policy(args, datastore).await,
        PolicyCommands::List(args) => list::list_policies(&args),
        PolicyCommands::Show(args) => list::show_policy(&args),
    }
}

#[cfg(test)]
mod eval_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod dispatch_tests;
