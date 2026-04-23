/// Command-level execution tests for node comparison.
#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use unet_core::datastore::{DataStoreError, MockDataStore};
    use uuid::Uuid;

    use super::super::compare::compare_nodes;
    use super::super::types::{CompareNodeArgs, CompareType};

    #[tokio::test]
    async fn test_compare_nodes_surfaces_missing_node_errors() {
        let first_node_id = Uuid::new_v4();
        let second_node_id = Uuid::new_v4();
        let mut store = MockDataStore::new();

        store
            .expect_get_node_required()
            .with(eq(first_node_id))
            .returning(move |_| {
                Box::pin(async move {
                    Err(DataStoreError::NotFound {
                        entity_type: "Node".to_string(),
                        id: first_node_id.to_string(),
                    })
                })
            });

        let args = CompareNodeArgs {
            node_a: first_node_id,
            node_b: Some(second_node_id),
            compare_type: vec![CompareType::All],
            diff_only: false,
        };

        let result = compare_nodes(args, &store, crate::OutputFormat::Json).await;

        assert!(matches!(
            result.unwrap_err().downcast_ref::<DataStoreError>(),
            Some(DataStoreError::NotFound { .. })
        ));
    }
}
