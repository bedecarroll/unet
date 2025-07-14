//! Policy validation functionality

use anyhow::Result;
use unet_core::config::GitConfig;
use unet_core::policy::{PolicyLoader, PolicyParser};

use super::ValidatePolicyArgs;

pub fn validate_policy(args: &ValidatePolicyArgs) -> Result<()> {
    println!("Validating policy file: {}", args.path.display());

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
                println!("❌ Policy file validation failed: {e}");
                return Err(anyhow::anyhow!("Policy validation failed: {}", e));
            }
        }
    } else if args.path.is_dir() {
        // Validate directory
        let load_result = loader.load_policies_from_directory(&args.path)?;

        if load_result.errors.is_empty() {
            println!("✅ All policy files in directory are valid");
            if args.verbose {
                println!("Policy files found: {}", load_result.loaded.len());
                for policy_file in &load_result.loaded {
                    println!(
                        "  📄 {} ({} rules)",
                        policy_file.path.display(),
                        policy_file.rules.len()
                    );
                }
            }
        } else {
            println!("❌ Some policy files failed validation:");
            for (file_path, error) in &load_result.errors {
                println!("  📄 {}: {}", file_path.display(), error);
            }
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
