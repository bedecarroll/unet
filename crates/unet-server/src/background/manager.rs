//! Background task manager

use std::sync::Arc;
use tracing::info;
use unet_core::{config::Config, datastore::DataStore, policy_integration::PolicyService};

use super::policy_task::PolicyEvaluationTask;

/// Background task manager
pub struct BackgroundTasks {
    config: Config,
    datastore: Arc<dyn DataStore + Send + Sync>,
    policy_service: PolicyService,
}

impl BackgroundTasks {
    /// Create a new background task manager
    pub fn new(
        config: Config,
        datastore: Arc<dyn DataStore + Send + Sync>,
        policy_service: PolicyService,
    ) -> Self {
        Self {
            config,
            datastore,
            policy_service,
        }
    }

    /// Start all background tasks
    pub fn start(&self) {
        info!("Starting background tasks");

        let mut policy_task = PolicyEvaluationTask::new(
            self.datastore.clone(),
            self.policy_service.clone(),
            self.config.git.sync_interval,
        );

        tokio::spawn(async move {
            policy_task.run().await;
        });

        info!("Background tasks started");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::{Migrator, MigratorTrait};
    use std::sync::Arc;
    use unet_core::datastore::sqlite::SqliteStore;

    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    #[tokio::test]
    async fn test_background_tasks_new() {
        let datastore = setup_test_datastore().await;
        let config = Config::default();
        let policy_service = PolicyService::with_local_dir("/tmp");

        let background_tasks = BackgroundTasks::new(config, Arc::new(datastore), policy_service);

        assert_eq!(background_tasks.config.git.sync_interval, 300);
    }
}
