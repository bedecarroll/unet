/// Tests for policy command arguments structures
use crate::commands::policy::{
    DiffPolicyArgs, EvalPolicyArgs, ListPolicyArgs, ShowPolicyArgs, ValidatePolicyArgs,
};

use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_validate_policy_args_creation() {
    let path = PathBuf::from("/test/policies/router.toml");
    let args = ValidatePolicyArgs {
        path: path.clone(),
        verbose: true,
    };

    assert_eq!(args.path, path);
    assert!(args.verbose);
}

#[test]
fn test_validate_policy_args_non_verbose() {
    let path = PathBuf::from("/simple/path");
    let args = ValidatePolicyArgs {
        path: path.clone(),
        verbose: false,
    };

    assert_eq!(args.path, path);
    assert!(!args.verbose);
}

#[test]
fn test_eval_policy_args_with_node_id() {
    let path = PathBuf::from("/policies/eval.toml");
    let node_id = Uuid::new_v4();
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: Some(node_id),
        verbose: true,
        failures_only: false,
    };

    assert_eq!(args.path, path);
    assert_eq!(args.node_id, Some(node_id));
    assert!(args.verbose);
    assert!(!args.failures_only);
}

#[test]
fn test_eval_policy_args_without_node_id() {
    let path = PathBuf::from("/policies/global.toml");
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: None,
        verbose: false,
        failures_only: true,
    };

    assert_eq!(args.path, path);
    assert_eq!(args.node_id, None);
    assert!(!args.verbose);
    assert!(args.failures_only);
}

#[test]
fn test_diff_policy_args_creation() {
    let path = PathBuf::from("/config/diff.toml");
    let node_id = Uuid::new_v4();
    let args = DiffPolicyArgs {
        path: path.clone(),
        node_id,
        verbose: true,
    };

    assert_eq!(args.path, path);
    assert_eq!(args.node_id, node_id);
    assert!(args.verbose);
}

#[test]
fn test_diff_policy_args_non_verbose() {
    let path = PathBuf::from("/simple/diff");
    let node_id = Uuid::new_v4();
    let args = DiffPolicyArgs {
        path: path.clone(),
        node_id,
        verbose: false,
    };

    assert_eq!(args.path, path);
    assert_eq!(args.node_id, node_id);
    assert!(!args.verbose);
}

#[test]
fn test_list_policy_args_creation() {
    let path = PathBuf::from("/policies/list");
    let args = ListPolicyArgs {
        path: path.clone(),
        verbose: true,
    };

    assert_eq!(args.path, path);
    assert!(args.verbose);
}

#[test]
fn test_list_policy_args_non_verbose() {
    let path = PathBuf::from("/minimal");
    let args = ListPolicyArgs {
        path: path.clone(),
        verbose: false,
    };

    assert_eq!(args.path, path);
    assert!(!args.verbose);
}

#[test]
fn test_show_policy_args_with_ast() {
    let path = PathBuf::from("/show/ast.toml");
    let args = ShowPolicyArgs {
        path: path.clone(),
        ast: true,
    };

    assert_eq!(args.path, path);
    assert!(args.ast);
}

#[test]
fn test_show_policy_args_without_ast() {
    let path = PathBuf::from("/show/no-ast.toml");
    let args = ShowPolicyArgs {
        path: path.clone(),
        ast: false,
    };

    assert_eq!(args.path, path);
    assert!(!args.ast);
}
