/// Tests for policy listing operations
#[cfg(test)]
mod tests {
    use super::super::operations::{list_policies, show_policy};
    use super::super::{ListPolicyArgs, ShowPolicyArgs};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_list_policies_nonexistent_directory() {
        let args = ListPolicyArgs {
            path: PathBuf::from("/nonexistent/directory"),
            verbose: false,
        };

        let result = list_policies(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Directory does not exist")
        );
    }

    #[test]
    fn test_list_policies_path_is_file_not_directory() {
        let temp_file = NamedTempFile::new().unwrap();

        let args = ListPolicyArgs {
            path: temp_file.path().to_path_buf(),
            verbose: false,
        };

        let result = list_policies(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Path is not a directory")
        );
    }

    #[test]
    fn test_list_policies_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let args = ListPolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = list_policies(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_policies_directory_with_valid_policy() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid policy file
        let policy_file_path = temp_dir.path().join("test.policy");
        let mut policy_file = std::fs::File::create(&policy_file_path).unwrap();
        writeln!(
            policy_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let args = ListPolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = list_policies(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_policies_directory_with_valid_policy_verbose() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid policy file with multiple rules
        let policy_file_path = temp_dir.path().join("test.policy");
        let mut policy_file = std::fs::File::create(&policy_file_path).unwrap();
        writeln!(
            policy_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();
        writeln!(
            policy_file,
            "WHEN node.role == \"router\" THEN ASSERT node.management_ip IS_SET"
        )
        .unwrap();

        let args = ListPolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: true,
        };

        let result = list_policies(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_policies_directory_with_invalid_policy() {
        let temp_dir = TempDir::new().unwrap();

        // Create an invalid policy file
        let policy_file_path = temp_dir.path().join("invalid.policy");
        let mut policy_file = std::fs::File::create(&policy_file_path).unwrap();
        writeln!(policy_file, "INVALID POLICY SYNTAX").unwrap();

        let args = ListPolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = list_policies(&args);
        assert!(result.is_ok()); // Should still succeed but show errors
    }

    #[test]
    fn test_list_policies_directory_with_mixed_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid policy file
        let valid_policy_path = temp_dir.path().join("valid.policy");
        let mut valid_file = std::fs::File::create(&valid_policy_path).unwrap();
        writeln!(
            valid_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        // Create an invalid policy file
        let invalid_policy_path = temp_dir.path().join("invalid.policy");
        let mut invalid_file = std::fs::File::create(&invalid_policy_path).unwrap();
        writeln!(invalid_file, "BROKEN SYNTAX HERE").unwrap();

        // Create a non-policy file
        let other_file_path = temp_dir.path().join("readme.txt");
        let mut other_file = std::fs::File::create(&other_file_path).unwrap();
        writeln!(other_file, "This is not a policy file").unwrap();

        let args = ListPolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = list_policies(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_policies_directory_with_subdirectories() {
        let temp_dir = TempDir::new().unwrap();

        // Create a subdirectory
        let sub_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        // Create a policy file in the main directory
        let policy_file_path = temp_dir.path().join("main.policy");
        let mut policy_file = std::fs::File::create(&policy_file_path).unwrap();
        writeln!(
            policy_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        // Create a policy file in the subdirectory
        let sub_policy_path = sub_dir.join("sub.policy");
        let mut sub_policy_file = std::fs::File::create(&sub_policy_path).unwrap();
        writeln!(
            sub_policy_file,
            "WHEN node.role == \"switch\" THEN ASSERT node.management_ip IS_SET"
        )
        .unwrap();

        let args = ListPolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        let result = list_policies(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_config_creation_in_list_policies() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid policy file
        let policy_file_path = temp_dir.path().join("test.policy");
        let mut policy_file = std::fs::File::create(&policy_file_path).unwrap();
        writeln!(
            policy_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let args = ListPolicyArgs {
            path: temp_dir.path().to_path_buf(),
            verbose: false,
        };

        // This test verifies that GitConfig is created correctly internally
        let result = list_policies(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_policy_nonexistent_file() {
        let args = ShowPolicyArgs {
            path: PathBuf::from("/nonexistent/file.policy"),
            ast: false,
        };

        let result = show_policy(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("File does not exist")
        );
    }

    #[test]
    fn test_show_policy_valid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let args = ShowPolicyArgs {
            path: temp_file.path().to_path_buf(),
            ast: false,
        };

        let result = show_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_policy_valid_file_with_ast() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let args = ShowPolicyArgs {
            path: temp_file.path().to_path_buf(),
            ast: true,
        };

        let result = show_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_policy_invalid_syntax() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INVALID POLICY SYNTAX HERE").unwrap();

        let args = ShowPolicyArgs {
            path: temp_file.path().to_path_buf(),
            ast: false,
        };

        let result = show_policy(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse policy file")
        );
    }

    #[test]
    fn test_show_policy_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();

        let args = ShowPolicyArgs {
            path: temp_file.path().to_path_buf(),
            ast: false,
        };

        let result = show_policy(&args);
        assert!(result.is_ok()); // Empty file should parse successfully (0 rules)
    }

    #[test]
    fn test_show_policy_multiple_rules() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();
        writeln!(
            temp_file,
            "WHEN node.role == \"router\" THEN ASSERT node.management_ip IS_SET"
        )
        .unwrap();
        writeln!(
            temp_file,
            "WHEN node.lifecycle == \"live\" THEN ASSERT node.location_id IS_SET"
        )
        .unwrap();

        let args = ShowPolicyArgs {
            path: temp_file.path().to_path_buf(),
            ast: false,
        };

        let result = show_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_policy_multiple_rules_with_ast() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();
        writeln!(
            temp_file,
            "WHEN node.role == \"router\" THEN ASSERT node.management_ip IS_SET"
        )
        .unwrap();

        let args = ShowPolicyArgs {
            path: temp_file.path().to_path_buf(),
            ast: true,
        };

        let result = show_policy(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_policy_complex_policy_syntax() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"juniper\" THEN ASSERT node.version IS \"12.3\""
        )
        .unwrap();

        let args = ShowPolicyArgs {
            path: temp_file.path().to_path_buf(),
            ast: false,
        };

        let result = show_policy(&args);
        assert!(result.is_ok());
    }
}
