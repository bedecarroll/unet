/// Link management CLI commands
use anyhow::Result;
use unet_core::datastore::DataStore;

mod crud;
mod types;

pub use types::LinkCommands;

/// Execute link subcommands.
///
/// # Errors
/// Returns an error if datastore operations or output formatting fail.
pub async fn execute(
    command: LinkCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        LinkCommands::Add(args) => crud::add_link(args, datastore, output_format).await,
        LinkCommands::List(args) => crud::list_links(args, datastore, output_format).await,
        LinkCommands::Show(args) => crud::show_link(args, datastore, output_format).await,
        LinkCommands::Update(args) => crud::update_link(args, datastore, output_format).await,
        LinkCommands::Delete(args) => crud::delete_link(args, datastore, output_format).await,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dispatch_tests {
    use super::*;
    use crate::commands::links::types::ListLinkArgs;
    use mockall::predicate::always;
    use unet_core::datastore::{types::PagedResult, MockDataStore};

    #[tokio::test]
    async fn test_execute_list_links_dispatch() {
        let mut mock = MockDataStore::new();
        mock.expect_list_links()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let args = ListLinkArgs { node_id: None, min_bandwidth: None, page: 1, per_page: 20 };

        let res = execute(
            types::LinkCommands::List(args),
            &mock,
            crate::OutputFormat::Json,
        )
        .await;
        assert!(res.is_ok());
    }
}
