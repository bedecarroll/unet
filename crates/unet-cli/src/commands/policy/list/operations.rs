/// Core policy listing operations
use anyhow::Result;
use std::path::Path;
use unet_core::config::GitConfig;
use unet_core::policy::{LoadResult, PolicyLoader, PolicyParser};

use super::{ListPolicyArgs, ShowPolicyArgs};
use crate::commands::policy::list::display::{display_policy_details, display_policy_listing};

/// Lists policy files in a directory
/// List policies in a directory.
///
/// # Errors
/// Returns an error if reading or parsing policy files fails.
pub fn list_policies(args: &ListPolicyArgs) -> Result<()> {
    if !args.path.exists() {
        return Err(anyhow::anyhow!(
            "Directory does not exist: {}",
            args.path.display()
        ));
    }

    if !args.path.is_dir() {
        return Err(anyhow::anyhow!(
            "Path is not a directory: {}",
            args.path.display()
        ));
    }

    let load_result = load_policies_from_directory(&args.path)?;
    display_policy_listing(&args.path, &load_result, args.verbose);

    Ok(())
}

/// Shows detailed information about a specific policy file
/// Show details for a single policy file.
///
/// # Errors
/// Returns an error if reading or parsing the policy file fails.
pub fn show_policy(args: &ShowPolicyArgs) -> Result<()> {
    if !args.path.exists() {
        return Err(anyhow::anyhow!(
            "File does not exist: {}",
            args.path.display()
        ));
    }

    let contents = std::fs::read_to_string(&args.path)?;

    // Parse the file to extract rules
    let parse_result = PolicyParser::parse_file(&contents);

    display_policy_details(&args.path, &contents, &parse_result, args.ast)?;

    Ok(())
}

/// Internal helper to load policies from a directory using `PolicyLoader`
fn load_policies_from_directory(path: &Path) -> Result<LoadResult> {
    // Create a default GitConfig for the loader
    let git_config = GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_owned(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };

    let mut loader = PolicyLoader::new(git_config);
    Ok(loader.load_policies_from_directory(path)?)
}
