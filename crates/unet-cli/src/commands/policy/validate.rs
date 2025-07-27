/// Policy validation functionality
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_validate_policy_valid_file_non_verbose() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let args = ValidatePolicyArgs {
            path: temp_file.path().to_path_buf(),
            verbose: false,
        };

        let result = validate_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policy_valid_file_verbose() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let args = ValidatePolicyArgs {
            path: temp_file.path().to_path_buf(),
            verbose: true,
        };

        let result = validate_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policy_invalid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INVALID POLICY SYNTAX").unwrap();

        let args = ValidatePolicyArgs {
            path: temp_file.path().to_path_buf(),
            verbose: false,
        };

        let result = validate_policy(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_policy_valid_directory_non_verbose() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("test.policy");

        fs::write(
            &policy_file,
            "WHEN node.role == \"router\" THEN ASSERT node.snmp_enabled IS true",
        )
        .unwrap();

        let args = ValidatePolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = validate_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policy_valid_directory_verbose() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file1 = temp_dir.path().join("test1.policy");
        let policy_file2 = temp_dir.path().join("test2.policy");

        fs::write(
            &policy_file1,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"",
        )
        .unwrap();
        fs::write(
            &policy_file2,
            "WHEN node.role == \"router\" THEN ASSERT node.snmp_enabled IS true",
        )
        .unwrap();

        let args = ValidatePolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: true,
        };

        let result = validate_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policy_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let args = ValidatePolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = validate_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policy_nonexistent_path() {
        let args = ValidatePolicyArgs {
            path: std::path::Path::new("/nonexistent/path").to_path_buf(),
            verbose: false,
        };

        let result = validate_policy(&args);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Path does not exist"));
    }

    #[test]
    fn test_validate_policy_directory_with_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        let valid_file = temp_dir.path().join("valid.policy");
        let invalid_file = temp_dir.path().join("invalid.policy");

        fs::write(
            &valid_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"",
        )
        .unwrap();
        fs::write(&invalid_file, "INVALID POLICY SYNTAX").unwrap();

        let args = ValidatePolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = validate_policy(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_policy_args_creation() {
        let path = std::path::Path::new("/test/path").to_path_buf();

        let args = ValidatePolicyArgs {
            path: path.clone(),
            verbose: true,
        };

        assert_eq!(args.path, path);
        assert!(args.verbose);

        let args_non_verbose = ValidatePolicyArgs {
            path: path.clone(),
            verbose: false,
        };

        assert_eq!(args_non_verbose.path, path);
        assert!(!args_non_verbose.verbose);
    }

    #[test]
    fn test_validate_policy_multiple_rules_in_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();
        writeln!(
            temp_file,
            "WHEN node.role == \"router\" THEN ASSERT node.snmp_enabled IS true"
        )
        .unwrap();

        let args = ValidatePolicyArgs {
            path: temp_file.path().to_path_buf(),
            verbose: true,
        };

        let result = validate_policy(&args);
        assert!(result.is_ok());
    }
}
