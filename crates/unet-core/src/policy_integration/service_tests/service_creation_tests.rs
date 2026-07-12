//! Tests for `PolicyService` creation and configuration

use super::super::service::PolicyService;
use super::mocks::{MockPolicyEvaluationEngine, create_test_git_config};
use std::sync::Arc;

#[test]
fn test_policy_service_new() {
    let git_config = create_test_git_config();
    let service = PolicyService::new(git_config);

    // Test that service was created successfully
    let _loader = service.loader();
    let _orchestrator = service.orchestrator();
}

#[test]
fn test_policy_service_with_local_dir() {
    let policies_dir = "/tmp/test_policies";
    let service = PolicyService::with_local_dir(policies_dir);

    // Test that service was created successfully with local directory
    let _loader = service.loader();
    let _orchestrator = service.orchestrator();
}

#[test]
fn test_policy_service_with_engine() {
    let git_config = create_test_git_config();
    let mock_engine = Arc::new(MockPolicyEvaluationEngine::new());
    let service = PolicyService::with_engine(git_config, mock_engine);

    // Test that service was created successfully with custom engine
    let _loader = service.loader();
    let _orchestrator = service.orchestrator();
}

#[test]
fn test_policy_service_loader_getter() {
    let git_config = create_test_git_config();
    let service = PolicyService::new(git_config);

    let _loader = service.loader();
    // Test that loader is accessible
}

#[test]
fn test_policy_service_orchestrator_getter() {
    let git_config = create_test_git_config();
    let service = PolicyService::new(git_config);

    let _orchestrator = service.orchestrator();
    // Test that orchestrator is accessible
}

#[test]
fn test_policy_service_clone() {
    let git_config = create_test_git_config();
    let service = PolicyService::new(git_config);

    let cloned_service = service;

    // Test that service can be cloned
    let _loader = cloned_service.loader();
    let _orchestrator = cloned_service.orchestrator();
}
