/// Policy listing display and formatting functionality
use anyhow::Result;
use std::path::Path;
use unet_core::policy::{LoadResult, ParseError, PolicyRule};

/// Displays the results of policy directory listing
pub fn display_policy_listing(path: &Path, load_result: &LoadResult, verbose: bool) {
    println!("Listing policy files in: {}", path.display());

    if load_result.loaded.is_empty() && load_result.errors.is_empty() {
        println!("No policy files found in directory");
        return;
    }

    display_loaded_policies(load_result, verbose);
    display_policy_errors(load_result);
}

/// Displays details for a specific policy file
/// Display parsed policy information and optional AST.
///
/// # Errors
/// Returns an error if output formatting fails.
pub fn display_policy_details(
    path: &Path,
    contents: &str,
    parse_result: &Result<Vec<PolicyRule>, ParseError>,
    show_ast: bool,
) -> Result<()> {
    println!("Policy file: {}", path.display());

    println!("\n--- File Contents ---");
    println!("{contents}");

    match parse_result {
        Ok(rules) => {
            display_parsed_rules(rules, show_ast);
        }
        Err(e) => {
            println!("\n❌ Failed to parse policy file: {e}");
            return Err(anyhow::anyhow!("Failed to parse policy file: {}", e));
        }
    }

    Ok(())
}

/// Displays successfully loaded policy files
fn display_loaded_policies(load_result: &LoadResult, verbose: bool) {
    if !load_result.loaded.is_empty() {
        println!("Found {} policy file(s):", load_result.loaded.len());

        for policy_file in &load_result.loaded {
            println!(
                "📄 {} ({} rules)",
                policy_file.path.display(),
                policy_file.rules.len()
            );

            if verbose {
                display_policy_rules(&policy_file.rules);
            }
        }
    }
}

/// Displays policy loading errors
fn display_policy_errors(load_result: &LoadResult) {
    for (file_path, error) in &load_result.errors {
        println!("❌ {}: {}", file_path.display(), error);
    }
}

/// Displays individual policy rules with indexing
fn display_policy_rules(rules: &[PolicyRule]) {
    for (i, rule) in rules.iter().enumerate() {
        println!("    Rule {}: {}", i + 1, rule);
    }
}

/// Displays parsed rules with optional AST information
fn display_parsed_rules(rules: &[PolicyRule], show_ast: bool) {
    println!("\n--- Parsed Rules ({}) ---", rules.len());

    for (i, rule) in rules.iter().enumerate() {
        println!("Rule {}: {}", i + 1, rule);

        if show_ast {
            println!("  AST: {rule:#?}");
        }
    }
}
