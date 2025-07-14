//! Policy listing and display functionality

use anyhow::Result;
use unet_core::config::GitConfig;
use unet_core::policy::{PolicyLoader, PolicyParser};

use super::{ListPolicyArgs, ShowPolicyArgs};

pub fn list_policies(args: &ListPolicyArgs) -> Result<()> {
    println!("Listing policy files in: {}", args.path.display());

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

    let load_result = loader.load_policies_from_directory(&args.path)?;

    if load_result.loaded.is_empty() && load_result.errors.is_empty() {
        println!("No policy files found in directory");
        return Ok(());
    }

    println!("Found {} policy file(s):", load_result.loaded.len());
    for policy_file in load_result.loaded {
        println!(
            "📄 {} ({} rules)",
            policy_file.path.display(),
            policy_file.rules.len()
        );
        if args.verbose {
            for (i, rule) in policy_file.rules.iter().enumerate() {
                println!("    Rule {}: {}", i + 1, rule);
            }
        }
    }

    for (file_path, error) in load_result.errors {
        println!("❌ {}: {}", file_path.display(), error);
    }

    Ok(())
}

pub fn show_policy(args: &ShowPolicyArgs) -> Result<()> {
    println!("Policy file: {}", args.path.display());

    if !args.path.exists() {
        return Err(anyhow::anyhow!(
            "File does not exist: {}",
            args.path.display()
        ));
    }

    println!("\n--- File Contents ---");
    let contents = std::fs::read_to_string(&args.path)?;
    println!("{contents}");

    // Parse the file to show the rules
    match PolicyParser::parse_file(&contents) {
        Ok(rules) => {
            println!("\n--- Parsed Rules ({}) ---", rules.len());
            for (i, rule) in rules.iter().enumerate() {
                println!("Rule {}: {}", i + 1, rule);

                if args.ast {
                    println!("  AST: {rule:#?}");
                }
            }
        }
        Err(e) => {
            println!("\n❌ Failed to parse policy file: {e}");
            return Err(anyhow::anyhow!("Failed to parse policy file: {}", e));
        }
    }

    Ok(())
}
