/// Execution tests for link CRUD commands
#[cfg(test)]
mod tests {
    use super::crud::*;
    use super::types::*;
    use unet_core::datastore::{MockDataStore, QueryOptions, testing::ready_ok};
    use uuid::Uuid;

    fn store_for_link(
        link: Option<unet_core::models::Link>,
        last_options: std::sync::Arc<std::sync::Mutex<Option<QueryOptions>>>,
        deleted: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> MockDataStore {
        let list_capture = last_options.clone();
        let deleted_flag = deleted.clone();
        let mut store = MockDataStore::new();
        store.expect_create_link()
            .returning(|link| ready_ok(link.clone()));
        store.expect_get_link()
            .returning(move |_| ready_ok(link.clone()));
        store.expect_list_links().returning(move |options| {
            *list_capture.lock().expect("lock last_options in list_links") = Some(options.clone());
            ready_ok(unet_core::datastore::types::PagedResult::new(
                vec![],
                0,
                options.pagination.as_ref(),
            ))
        });
        store.expect_update_link()
            .returning(|link| ready_ok(link.clone()));
        store.expect_delete_link().returning(move |_| {
            deleted_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            ready_ok(())
        });
        store
    }

    fn example_link() -> unet_core::models::Link {
        unet_core::models::Link::new(
            "l1".to_string(),
            Uuid::new_v4(),
            "e0".to_string(),
            Uuid::new_v4(),
            "e1".to_string(),
        )
    }

    #[tokio::test]
    async fn test_add_and_show_update_delete_link() {
        let list_options = std::sync::Arc::new(std::sync::Mutex::new(None));
        let deleted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let store = store_for_link(None, list_options.clone(), deleted.clone());

        let args = AddLinkArgs {
            name: "l1".to_string(),
            node_a_id: Uuid::new_v4(),
            node_a_interface: "e0".to_string(),
            node_z_id: Some(Uuid::new_v4()),
            node_z_interface: Some("e1".to_string()),
            bandwidth_bps: Some(1_000_000),
            description: Some("desc".to_string()),
            custom_data: Some("{\"prio\":1}".to_string()),
        };
        assert!(add_link(args, &store, crate::OutputFormat::Json).await.is_ok());

        let link = example_link();
        let show_store =
            store_for_link(Some(link.clone()), list_options.clone(), deleted.clone());
        let show_args = ShowLinkArgs { id: link.id };
        assert!(show_link(show_args, &show_store, crate::OutputFormat::Json).await.is_ok());

        let upd_args = UpdateLinkArgs {
            id: link.id,
            name: Some("l2".to_string()),
            node_a_id: None,
            node_a_interface: Some("e2".to_string()),
            node_z_id: None,
            node_z_interface: None,
            bandwidth_bps: Some(2_000_000),
            description: Some("d2".to_string()),
            custom_data: Some("{\"prio\":2}".to_string()),
        };
        assert!(update_link(upd_args, &show_store, crate::OutputFormat::Json).await.is_ok());

        let delete_store =
            store_for_link(Some(link.clone()), list_options.clone(), deleted.clone());
        let del_args = DeleteLinkArgs {
            id: link.id,
            yes: true,
        };
        assert!(delete_link(del_args, &delete_store, crate::OutputFormat::Json).await.is_ok());
        assert!(deleted.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_list_links_captures_filters() {
        let last_options = std::sync::Arc::new(std::sync::Mutex::new(None));
        let deleted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let store = store_for_link(None, last_options.clone(), deleted);

        let args = ListLinkArgs {
            node_id: Some(Uuid::new_v4()),
            page: 2,
            per_page: 10,
        };
        assert!(list_links(args, &store, crate::OutputFormat::Json).await.is_ok());
        let options = last_options
            .lock()
            .expect("lock last_options after list_links")
            .clone()
            .expect("options recorded");
        assert!(options.pagination.is_some());
    }
}
