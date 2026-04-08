/// Execution tests for location CRUD commands
#[cfg(test)]
mod tests {
    use super::crud::*;
    use super::types::*;
    use unet_core::datastore::{MockDataStore, QueryOptions, testing::ready_ok};
    use uuid::Uuid;

    fn store_for_location(
        location: Option<unet_core::models::Location>,
        last_options: std::sync::Arc<std::sync::Mutex<Option<QueryOptions>>>,
        deleted: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> MockDataStore {
        let list_capture = last_options.clone();
        let deleted_flag = deleted.clone();
        let mut store = MockDataStore::new();
        store.expect_create_location()
            .returning(|location| ready_ok(location.clone()));
        store.expect_get_location()
            .returning(move |_| ready_ok(location.clone()));
        store.expect_list_locations().returning(move |options| {
            *list_capture
                .lock()
                .expect("lock last_options in list_locations") = Some(options.clone());
            ready_ok(unet_core::datastore::types::PagedResult::new(
                vec![],
                0,
                options.pagination.as_ref(),
            ))
        });
        store.expect_update_location()
            .returning(|location| ready_ok(location.clone()));
        store.expect_delete_location().returning(move |_| {
            deleted_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            ready_ok(())
        });
        store
    }

    fn example_location() -> unet_core::models::Location {
        unet_core::models::Location {
            id: Uuid::new_v4(),
            name: "loc1".to_string(),
            location_type: "dc".to_string(),
            parent_id: None,
            path: "loc1".to_string(),
            description: None,
            address: None,
            custom_data: serde_json::Value::Null,
        }
    }

    #[tokio::test]
    async fn test_add_show_update_delete_location() {
        let last_options = std::sync::Arc::new(std::sync::Mutex::new(None));
        let deleted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let store = store_for_location(None, last_options.clone(), deleted.clone());

        let args = AddLocationArgs {
            name: "loc1".to_string(),
            location_type: "dc".to_string(),
            parent_id: None,
            address: Some("addr".to_string()),
            city: Some("city".to_string()),
            country: Some("cty".to_string()),
            custom_data: Some("{}".to_string()),
        };
        assert!(add_location(args, &store, crate::OutputFormat::Json).await.is_ok());

        let location = example_location();
        let show_store =
            store_for_location(Some(location.clone()), last_options.clone(), deleted.clone());
        let show_args = ShowLocationArgs { id: location.id };
        assert!(show_location(show_args, &show_store, crate::OutputFormat::Json).await.is_ok());

        let upd_args = UpdateLocationArgs {
            id: location.id,
            name: Some("L2".to_string()),
            location_type: Some("room".to_string()),
            parent_id: None,
            address: Some("a".to_string()),
            city: Some("b".to_string()),
            country: Some("c".to_string()),
            custom_data: Some("{}".to_string()),
        };
        assert!(update_location(upd_args, &show_store, crate::OutputFormat::Json).await.is_ok());

        let delete_store =
            store_for_location(Some(location.clone()), last_options.clone(), deleted.clone());
        let del_args = DeleteLocationArgs {
            id: location.id,
            yes: true,
        };
        assert!(delete_location(del_args, &delete_store, crate::OutputFormat::Json).await.is_ok());
        assert!(deleted.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_list_locations_captures_filters() {
        let last_options = std::sync::Arc::new(std::sync::Mutex::new(None));
        let deleted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let store = store_for_location(None, last_options.clone(), deleted);

        let args = ListLocationArgs {
            location_type: Some("dc".to_string()),
            parent_id: Some(Uuid::new_v4()),
            page: 1,
            per_page: 20,
        };
        assert!(list_locations(args, &store, crate::OutputFormat::Json).await.is_ok());
        let options = last_options
            .lock()
            .expect("lock last_options after list_locations")
            .clone()
            .expect("options recorded");
        assert!(options.pagination.is_some());
        assert_eq!(options.filters.len(), 2);
    }
}
