/// Tests for node list functionality
#[cfg(test)]
mod tests {
    use super::super::list::list_nodes;
    use super::super::types::ListNodeArgs;
    use unet_core::datastore::{MockDataStore, QueryOptions, testing::ready_ok};

    #[tokio::test]
    async fn test_list_nodes_builds_filters_and_pagination() {
        let last_options = std::sync::Arc::new(std::sync::Mutex::new(None::<QueryOptions>));
        let options_capture = last_options.clone();

        let mut store = MockDataStore::new();
        store.expect_list_nodes().returning(move |options| {
            *options_capture.lock().expect("lock last_options") = Some(options.clone());
            ready_ok(unet_core::datastore::types::PagedResult::new(
                vec![],
                0,
                options.pagination.as_ref(),
            ))
        });

        let args = ListNodeArgs {
            lifecycle: Some("live".to_string()),
            role: Some("router".to_string()),
            vendor: Some("cisco".to_string()),
            page: 2,
            per_page: 5,
        };

        let result = list_nodes(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());

        let opts = last_options
            .lock()
            .expect("lock last_options after list")
            .clone()
            .expect("options recorded");
        assert_eq!(opts.filters.len(), 3);
        assert!(opts.pagination.is_some());
        let pagination = opts.pagination.expect("pagination should be present");
        assert_eq!(pagination.limit, 5);
        assert_eq!(pagination.offset, 5);
    }
}
