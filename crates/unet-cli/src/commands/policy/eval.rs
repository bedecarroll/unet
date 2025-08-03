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
}
