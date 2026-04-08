//! Policy results handlers and utilities

use axum::{
    Json,
    extract::{Query, State},
};
use tracing::info;

use crate::{
    error::{ServerError, ServerResult},
    server::AppState,
};

use super::types::{PolicyResultsQuery, PolicyResultsResponse};

/// Get policy evaluation results
///
/// # Errors
/// Returns an error if datastore operations fail.
pub async fn get_policy_results(
    State(state): State<AppState>,
    Query(query): Query<PolicyResultsQuery>,
) -> ServerResult<Json<PolicyResultsResponse>> {
    info!("Getting policy results with filter: {:?}", query);

    let node_id = query.node_id.ok_or_else(|| {
        ServerError::BadRequest(
            "node_id query parameter is required; cross-node policy result queries are not supported"
                .to_string(),
        )
    })?;

    let results = state.datastore.get_policy_results(&node_id).await?;
    let total_count = results.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);
    let paginated_results: Vec<_> = results.into_iter().skip(offset).take(limit).collect();
    let returned_count = paginated_results.len();

    Ok(Json(PolicyResultsResponse {
        results: paginated_results,
        total_count,
        returned_count,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use std::sync::Arc;
    use tempfile::tempdir;
    use unet_core::{
        config::Config,
        datastore::{DataStoreError, MockDataStore},
        policy::{
            Action, Condition, EvaluationResult, FieldRef, PolicyExecutionResult, PolicyRule, Value,
        },
        policy_integration::PolicyService,
    };
    use uuid::Uuid;

    fn test_policy_result(rule_id: &str) -> PolicyExecutionResult {
        PolicyExecutionResult::new(
            PolicyRule {
                id: Some(rule_id.to_string()),
                condition: Condition::True,
                action: Action::Assert {
                    field: FieldRef {
                        path: vec!["node".to_string(), "status".to_string()],
                    },
                    expected: Value::String("active".to_string()),
                },
            },
            EvaluationResult::NotSatisfied,
            None,
        )
    }

    #[tokio::test]
    async fn test_get_policy_results_with_node_id() {
        let policies_dir = tempdir().unwrap();
        let policies_path = policies_dir.path().to_str().unwrap();
        let node_id = Uuid::new_v4();
        let mut mock_datastore = MockDataStore::new();
        mock_datastore
            .expect_get_policy_results()
            .withf(move |candidate| *candidate == node_id)
            .return_once(|_| {
                Box::pin(async move {
                    Ok(vec![
                        test_policy_result("rule-1"),
                        test_policy_result("rule-2"),
                    ])
                })
            });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::with_local_dir(policies_path),
        };

        let query = PolicyResultsQuery {
            node_id: Some(node_id),
            offset: None,
            limit: None,
        };

        let Json(response) = get_policy_results(State(app_state), axum::extract::Query(query))
            .await
            .unwrap();
        assert_eq!(response.total_count, 2);
        assert_eq!(response.returned_count, 2);
        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].rule.id.as_deref(), Some("rule-1"));
    }

    #[tokio::test]
    async fn test_get_policy_results_without_node_id() {
        let policies_dir = tempdir().unwrap();
        let policies_path = policies_dir.path().to_str().unwrap();
        let app_state = AppState {
            datastore: Arc::new(MockDataStore::new()),
            policy_service: PolicyService::with_local_dir(policies_path),
        };

        let query = PolicyResultsQuery {
            node_id: None,
            offset: None,
            limit: None,
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("node_id query parameter is required")
        );
    }

    #[tokio::test]
    async fn test_get_policy_results_with_pagination() {
        let policies_dir = tempdir().unwrap();
        let policies_path = policies_dir.path().to_str().unwrap();
        let node_id = Uuid::new_v4();
        let mut mock_datastore = MockDataStore::new();
        mock_datastore
            .expect_get_policy_results()
            .withf(move |candidate| *candidate == node_id)
            .return_once(|_| {
                Box::pin(async move {
                    Ok(vec![
                        test_policy_result("rule-1"),
                        test_policy_result("rule-2"),
                        test_policy_result("rule-3"),
                    ])
                })
            });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::with_local_dir(policies_path),
        };

        let query = PolicyResultsQuery {
            node_id: Some(node_id),
            offset: Some(1),
            limit: Some(1),
        };

        let Json(response) = get_policy_results(State(app_state), axum::extract::Query(query))
            .await
            .unwrap();
        assert_eq!(response.total_count, 3);
        assert_eq!(response.returned_count, 1);
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].rule.id.as_deref(), Some("rule-2"));
    }

    #[tokio::test]
    async fn test_get_policy_results_with_unsupported_datastore_operation() {
        let node_id = Uuid::new_v4();
        let mut mock_datastore = MockDataStore::new();
        mock_datastore
            .expect_get_policy_results()
            .withf(move |candidate| *candidate == node_id)
            .return_once(|_| {
                Box::pin(async move {
                    Err(DataStoreError::UnsupportedOperation {
                        operation: "get_policy_results".to_string(),
                    })
                })
            });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::new(Config::default().git),
        };

        let query = PolicyResultsQuery {
            node_id: Some(node_id),
            offset: None,
            limit: None,
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("get_policy_results")
        );
    }
}
