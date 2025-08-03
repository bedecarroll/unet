/// Tests for policy evaluation command arguments
use crate::commands::policy::{DiffPolicyArgs, EvalPolicyArgs};
use std::path::PathBuf;
use uuid::Uuid;

#[tokio::test]
async fn test_eval_policy_args_creation() {
    let path = PathBuf::from("/test/policy");
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

#[tokio::test]
async fn test_eval_policy_args_no_node_id() {
    let path = PathBuf::from("/test/policy");

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

#[tokio::test]
async fn test_diff_policy_args_creation() {
    let path = PathBuf::from("/test/policy");
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

#[tokio::test]
async fn test_diff_policy_args_not_verbose() {
    let path = PathBuf::from("/test/policy");
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