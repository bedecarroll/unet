//! Tests for policy loading and parsing functionality

use crate::handlers::policies::policy_execution::policy_loader;
use crate::handlers::policies::types::PolicyEvaluationRequest;
use unet_core::config::GitConfig;
use unet_core::policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value};
use unet_core::prelude::PolicyService;

#[tokio::test]
async fn test_load_policies_for_request_with_policies() {
    let test_rule = PolicyRule {
        id: Some("test-rule".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["version".to_string()],
            },
            expected: Value::String("15.1".to_string()),
        },
    };

    let request = PolicyEvaluationRequest {
        policies: Some(vec![test_rule]),
        ..Default::default()
    };

    let git_config = GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };
    let mut policy_service = PolicyService::new(git_config);
    let result = policy_loader::load_policies_for_request(&mut policy_service, &request);
    assert!(result.is_ok());
    let loaded_policies = result.unwrap();
    assert_eq!(loaded_policies.len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_load_policies_for_request_without_policies() {
    let request = PolicyEvaluationRequest {
        policies: None,
        ..Default::default()
    };

    // Create a temporary directory for the policy service with an empty policies dir
    let temp_dir = std::env::temp_dir().join("test_policies");
    std::fs::create_dir_all(&temp_dir).unwrap();
    // Create an empty .gitkeep file to make the directory valid for policy loading
    std::fs::write(temp_dir.join(".gitkeep"), "").unwrap();

    let git_config = GitConfig {
        repository_url: None,
        local_directory: Some(temp_dir.to_string_lossy().to_string()),
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };
    let mut policy_service = PolicyService::new(git_config);
    let result = policy_loader::load_policies_for_request(&mut policy_service, &request);
    assert!(result.is_ok());
    let loaded_policies = result.unwrap();
    assert!(loaded_policies.is_empty());

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_load_policies_for_request_empty_policies() {
    let request = PolicyEvaluationRequest {
        policies: Some(vec![]),
        ..Default::default()
    };

    let git_config = GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };
    let mut policy_service = PolicyService::new(git_config);
    let result = policy_loader::load_policies_for_request(&mut policy_service, &request);
    assert!(result.is_ok());
    let loaded_policies = result.unwrap();
    assert!(loaded_policies.is_empty());
}

#[test]
fn test_load_policies_for_request_multiple_policies() {
    let test_rule1 = PolicyRule {
        id: Some("rule1".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["version".to_string()],
            },
            expected: Value::String("15.1".to_string()),
        },
    };

    let test_rule2 = PolicyRule {
        id: Some("rule2".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["model".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("ASR1000".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["status".to_string()],
            },
            expected: Value::String("up".to_string()),
        },
    };

    let request = PolicyEvaluationRequest {
        policies: Some(vec![test_rule1, test_rule2]),
        ..Default::default()
    };

    let git_config = GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };
    let mut policy_service = PolicyService::new(git_config);
    let result = policy_loader::load_policies_for_request(&mut policy_service, &request);
    assert!(result.is_ok());
    let loaded_policies = result.unwrap();
    assert_eq!(loaded_policies.len(), 2);
}
