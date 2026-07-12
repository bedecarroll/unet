/// Execution tests for node delete command
#[cfg(test)]
mod tests {
    use super::super::delete::delete_node;
    use super::super::types::DeleteNodeArgs;
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use uuid::Uuid;

    fn make_node() -> unet_core::models::Node {
        use unet_core::models::*;
        let id = Uuid::new_v4();
        NodeBuilder::new()
            .id(id)
            .name("edge-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_delete_node_yes_skips_prompt() {
        let node = make_node();
        let deleted = std::sync::Arc::new(std::sync::Mutex::new(false));
        let deleted_flag = deleted.clone();
        let expected_node = node.clone();

        let mut store = MockDataStore::new();
        store
            .expect_get_node_required()
            .returning(move |_| ready_ok(expected_node.clone()));
        store.expect_delete_node().returning(move |_| {
            *deleted_flag.lock().expect("lock deleted flag") = true;
            ready_ok(())
        });

        let args = DeleteNodeArgs {
            id: node.id,
            yes: true,
        };
        let result = delete_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
        assert!(*deleted.lock().expect("lock deleted flag after delete"));
    }
}

#[cfg(test)]
mod confirm_tests {
    use super::super::delete::confirm_deletion;

    #[test]
    fn test_confirm_deletion_negative_and_positive() {
        let mut cur = std::io::Cursor::new(b"n\n".to_vec());
        let res = confirm_deletion(false, "edge-1", &mut cur).unwrap();
        assert!(!res);

        let mut cur2 = std::io::Cursor::new(b"yes\n".to_vec());
        let res2 = confirm_deletion(false, "edge-1", &mut cur2).unwrap();
        assert!(res2);
    }
}
