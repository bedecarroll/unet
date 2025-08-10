/// Policy evaluation and compliance checking functionality
use anyhow::Result;
use unet_core::datastore::DataStore;
use unet_core::policy::{EvaluationContext, PolicyEvaluator};

use super::helpers::{get_evaluation_nodes, load_policies_from_path};
use super::{DiffPolicyArgs, EvalPolicyArgs};

pub async fn eval_policy(args: EvalPolicyArgs, datastore: &dyn DataStore) -> Result<()> {
    println!("Evaluating policies from: {}", args.path.display());

    // Load policies
    let policies = load_policies_from_path(&args.path)?;

    // Get nodes to evaluate against
    let nodes = get_evaluation_nodes(args.node_id, datastore).await?;

    if nodes.is_empty() {
        println!("⚠️  No nodes found to evaluate policies against");
        return Ok(());
    }

    println!("Evaluating against {} node(s)", nodes.len());

    for node in nodes {
        println!("\n--- Node: {} ({}) ---", node.name, node.id);
        evaluate_node_policies(&node, &policies, &args)?;
    }

    Ok(())
}

#[cfg(test)]
mod e2e_light_tests {
    use super::*;
    use mockall::predicate::eq;
    use tempfile::NamedTempFile;
    use unet_core::datastore::{types::PagedResult, MockDataStore};
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
    use std::io::Write;

    fn make_node_with_version(ver: &str) -> unet_core::models::Node {
        NodeBuilder::new()
            .name("node-pol")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("CSR")
            .role(DeviceRole::Router)
            .version(ver.to_string())
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_eval_policy_no_nodes_branch() {
        // Write a minimal valid policy file
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"").unwrap();

        let mut mock = MockDataStore::new();
        // Return empty nodes set
        mock.expect_list_nodes()
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let args = super::EvalPolicyArgs { path: f.path().into(), node_id: None, verbose: false, failures_only: false };
        let res = super::eval_policy(args, &mock).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_eval_and_diff_policy_with_nodes() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"").unwrap();

        let node = make_node_with_version("15.1");
        let node_id = node.id;

        let mut mock = MockDataStore::new();
        let node_for_list = node.clone();
        mock.expect_list_nodes()
            .returning(move |_| {
                let n = node_for_list.clone();
                Box::pin(async move { Ok(PagedResult::new(vec![n], 1, None)) })
            });

        let args = super::EvalPolicyArgs { path: f.path().into(), node_id: None, verbose: true, failures_only: false };
        let res = super::eval_policy(args, &mock).await;
        assert!(res.is_ok());

        // diff_policy: pass case
        let mut mock2 = MockDataStore::new();
        let node_for_get = node.clone();
        mock2.expect_get_node()
            .with(eq(node_id))
            .returning(move |_| {
                let n = node_for_get.clone();
                Box::pin(async move { Ok(Some(n)) })
            });
        let args2 = super::DiffPolicyArgs { path: f.path().into(), node_id, verbose: true };
        let res2 = super::diff_policy(args2, &mock2).await;
        assert!(res2.is_ok());
    }
}

/// Evaluate policies against a specific node and show the results
fn evaluate_node_policies(
    node: &unet_core::models::Node,
    policies: &[Vec<unet_core::policy::PolicyRule>],
    args: &EvalPolicyArgs,
) -> Result<()> {
    // Create evaluation context
    let node_json = serde_json::to_value(node)?;
    let context = EvaluationContext {
        node_data: node_json,
        derived_data: None,
    };

    let mut total_rules = 0;
    let mut satisfied_rules = 0;
    let mut failed_rules = 0;

    for policy_rules in policies {
        for (rule_idx, rule) in policy_rules.iter().enumerate() {
            total_rules += 1;

            // Evaluate the rule
            match PolicyEvaluator::evaluate_rule(rule, &context) {
                Ok(evaluation_result) => match evaluation_result {
                    unet_core::policy::EvaluationResult::Satisfied { action } => {
                        satisfied_rules += 1;
                        if args.verbose && !args.failures_only {
                            println!("  ✅ Rule {}: SATISFIED", rule_idx + 1);
                            println!("    Action: {action:?}");
                        }
                    }
                    unet_core::policy::EvaluationResult::NotSatisfied => {
                        failed_rules += 1;
                        if args.verbose || args.failures_only {
                            println!("  ❌ Rule {}: NOT SATISFIED", rule_idx + 1);
                        }
                    }
                    unet_core::policy::EvaluationResult::Error { message } => {
                        failed_rules += 1;
                        if args.verbose || args.failures_only {
                            println!("  ⚠️  Rule {}: ERROR", rule_idx + 1);
                            println!("    Error: {message}");
                        }
                    }
                },
                Err(e) => {
                    failed_rules += 1;
                    if args.verbose || args.failures_only {
                        println!("  ⚠️  Rule {}: ERROR", rule_idx + 1);
                        println!("    Error: {e}");
                    }
                }
            }
        }
    }

    println!("Results: {total_rules} total, {satisfied_rules} satisfied, {failed_rules} failed");

    Ok(())
}

pub async fn diff_policy(args: DiffPolicyArgs, datastore: &dyn DataStore) -> Result<()> {
    println!("Checking compliance for node: {}", args.node_id);

    // Get the specific node
    let node = match datastore.get_node(&args.node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(anyhow::anyhow!("Node not found: {}", args.node_id)),
        Err(e) => return Err(anyhow::anyhow!("Failed to get node: {}", e)),
    };

    // Load policies
    let policies = load_policies_from_path(&args.path)?;

    // Create evaluation context
    let node_json = serde_json::to_value(&node)?;
    let context = EvaluationContext {
        node_data: node_json,
        derived_data: None,
    };

    println!("Compliance check for node '{}' ({}):", node.name, node.id);

    let mut compliance_checks = 0;
    let mut passed_checks = 0;
    let mut failed_checks = 0;

    for policy_rules in &policies {
        for (rule_idx, rule) in policy_rules.iter().enumerate() {
            // Only check ASSERT actions for compliance
            if let unet_core::policy::Action::Assert { .. } = rule.action {
                compliance_checks += 1;

                // Evaluate the rule
                match PolicyEvaluator::evaluate_rule(rule, &context) {
                    Ok(evaluation_result) => match evaluation_result {
                        unet_core::policy::EvaluationResult::Satisfied { .. } => {
                            passed_checks += 1;
                            if args.verbose {
                                println!("  ✅ Compliance Rule {}: PASSED", rule_idx + 1);
                            }
                        }
                        unet_core::policy::EvaluationResult::NotSatisfied => {
                            failed_checks += 1;
                            println!("  ❌ Compliance Rule {}: FAILED", rule_idx + 1);
                            println!("    Expected compliance but condition was not satisfied");

                            if args.verbose {
                                println!("    Rule: {rule}");
                            }
                        }
                        unet_core::policy::EvaluationResult::Error { message } => {
                            failed_checks += 1;
                            println!("  ⚠️  Compliance Rule {}: ERROR", rule_idx + 1);
                            println!("    Error: {message}");
                        }
                    },
                    Err(e) => {
                        failed_checks += 1;
                        println!("  ⚠️  Compliance Rule {}: ERROR", rule_idx + 1);
                        println!("    Error: {e}");
                    }
                }
            }
        }
    }

    if compliance_checks == 0 {
        println!("ℹ️  No compliance rules (ASSERT actions) found in policy files");
    } else {
        println!(
            "\nCompliance Summary: {compliance_checks} checks, {passed_checks} passed, {failed_checks} failed"
        );

        if failed_checks == 0 {
            println!("🎉 Node is fully compliant!");
        } else {
            println!("⚠️  Node has compliance violations that need attention");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    //! Tests are organized into focused modules by functionality.

    #[cfg(test)]
    mod args_tests {
        include!("eval/args_tests.rs");
    }

    #[cfg(test)]
    mod evaluation_tests {
        include!("eval/evaluation_tests.rs");
    }

    #[cfg(test)]
    mod utility_tests {
        include!("eval/utility_tests.rs");
    }

    #[cfg(test)]
    mod comprehensive_tests {
        include!("eval/comprehensive_tests.rs");
    }

    #[cfg(test)]
    mod branch_tests {
        use crate::commands::policy::eval::evaluate_node_policies;
        use crate::commands::policy::EvalPolicyArgs;
        use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
        use unet_core::policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value};

        fn make_node() -> unet_core::models::Node {
            NodeBuilder::new()
                .name("pol-node")
                .domain("example.com")
                .vendor(Vendor::Cisco)
                .model("CSR")
                .role(DeviceRole::Router)
                .build()
                .unwrap()
        }

        fn assert_rule_true() -> PolicyRule {
            PolicyRule { id: None, condition: Condition::True, action: Action::Assert { field: FieldRef { path: vec!["node".into(), "vendor".into()] }, expected: Value::String("cisco".into()) } }
        }

        fn assert_rule_false() -> PolicyRule {
            PolicyRule { id: None, condition: Condition::False, action: Action::Assert { field: FieldRef { path: vec!["node".into(), "vendor".into()] }, expected: Value::String("cisco".into()) } }
        }

        fn error_rule() -> PolicyRule {
            // Reference a non-existent field to trigger an evaluation error
            PolicyRule { id: None, condition: Condition::Comparison { field: FieldRef { path: vec!["node".into(), "does_not_exist".into()] }, operator: ComparisonOperator::Equal, value: Value::String("x".into()) }, action: Action::Assert { field: FieldRef { path: vec!["node".into(), "vendor".into()] }, expected: Value::String("cisco".into()) } }
        }

        #[test]
        fn test_evaluate_node_policies_satisfied_verbose() {
            let node = make_node();
            let policies = vec![vec![assert_rule_true()]];
            let args = EvalPolicyArgs { path: std::path::PathBuf::from("."), node_id: None, verbose: true, failures_only: false };
            let res = evaluate_node_policies(&node, &policies, &args);
            assert!(res.is_ok());
        }

        #[test]
        fn test_evaluate_node_policies_not_satisfied_failures_only() {
            let node = make_node();
            let policies = vec![vec![assert_rule_false()]];
            let args = EvalPolicyArgs { path: std::path::PathBuf::from("."), node_id: None, verbose: false, failures_only: true };
            let res = evaluate_node_policies(&node, &policies, &args);
            assert!(res.is_ok());
        }

        #[test]
        fn test_evaluate_node_policies_error_branch() {
            let node = make_node();
            let policies = vec![vec![error_rule()]];
            let args = EvalPolicyArgs { path: std::path::PathBuf::from("."), node_id: None, verbose: true, failures_only: true };
            let res = evaluate_node_policies(&node, &policies, &args);
            assert!(res.is_ok());
        }
    }
}
