//! Policy validation handlers and utilities

use axum::{Json, extract::State};
use tracing::info;
use unet_core::policy::PolicyRule;

use crate::{error::ServerResult, server::AppState};

/// Validate policy rules
pub async fn validate_policies(
    State(_state): State<AppState>,
    Json(policies): Json<Vec<PolicyRule>>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Validating {} policy rules", policies.len());

    let mut validation_results = Vec::new();
    let mut valid_count = 0;
    let mut error_count = 0;

    for (index, policy) in policies.iter().enumerate() {
        // For now, just check if the policy has basic required fields
        // In a full implementation, we'd parse and validate the policy syntax
        let is_valid =
            !policy.condition.to_string().is_empty() && !policy.action.to_string().is_empty();

        if is_valid {
            valid_count += 1;
            validation_results.push(serde_json::json!({
                "index": index,
                "valid": true,
                "message": "Policy rule is valid"
            }));
        } else {
            error_count += 1;
            validation_results.push(serde_json::json!({
                "index": index,
                "valid": false,
                "message": "Policy rule is missing required fields"
            }));
        }
    }

    Ok(Json(serde_json::json!({
        "total_policies": policies.len(),
        "valid_policies": valid_count,
        "invalid_policies": error_count,
        "validation_results": validation_results
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use migration::{Migrator, MigratorTrait};
    use std::sync::Arc;
    use unet_core::{
        datastore::sqlite::SqliteStore,
        policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value},
        policy_integration::PolicyService,
    };

    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    fn create_test_policy_rule() -> PolicyRule {
        PolicyRule {
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
        }
    }

    #[tokio::test]
    async fn test_validate_policies_valid() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let policies = vec![create_test_policy_rule()];

        let result = validate_policies(State(app_state), Json(policies)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response["total_policies"], 1);
        assert_eq!(response["valid_policies"], 1);
        assert_eq!(response["invalid_policies"], 0);
    }

    #[tokio::test]
    async fn test_validate_policies_empty() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let policies = vec![];

        let result = validate_policies(State(app_state), Json(policies)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response["total_policies"], 0);
        assert_eq!(response["valid_policies"], 0);
        assert_eq!(response["invalid_policies"], 0);
    }
}
