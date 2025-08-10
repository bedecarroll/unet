/// Location management CLI commands
use anyhow::Result;
use unet_core::datastore::DataStore;

mod crud;
mod types;

pub use types::LocationCommands;

pub async fn execute(
    command: LocationCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        LocationCommands::Add(args) => crud::add_location(args, datastore, output_format).await,
        LocationCommands::List(args) => crud::list_locations(args, datastore, output_format).await,
        LocationCommands::Show(args) => crud::show_location(args, datastore, output_format).await,
        LocationCommands::Update(args) => {
            crud::update_location(args, datastore, output_format).await
        }
        LocationCommands::Delete(args) => {
            crud::delete_location(args, datastore, output_format).await
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dispatch_tests {
    use super::*;
    use crate::commands::locations::types::ListLocationArgs;
    use mockall::predicate::always;
    use unet_core::datastore::{types::PagedResult, MockDataStore};

    #[tokio::test]
    async fn test_execute_list_locations_dispatch() {
        let mut mock = MockDataStore::new();
        mock.expect_list_locations()
            .with(always())
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let args = ListLocationArgs { location_type: None, parent_id: None, page: 1, per_page: 20 };

        let res = execute(
            types::LocationCommands::List(args),
            &mock,
            crate::OutputFormat::Json,
        )
        .await;
        assert!(res.is_ok());
    }
}
