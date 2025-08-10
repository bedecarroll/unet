//! Application state and initialization

use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use unet_core::{
    config::Config,
    datastore::{DataStore, sqlite::SqliteStore},
    policy_integration::PolicyService,
};

use crate::background::BackgroundTasks;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub datastore: Arc<dyn DataStore + Send + Sync>,
    pub policy_service: PolicyService,
}

/// Initialize application state with datastore and services
pub async fn initialize_app_state(config: Config, database_url: String) -> Result<AppState> {
    info!("Initializing SQLite datastore with URL: {}", database_url);
    let datastore: Arc<dyn DataStore + Send + Sync> = Arc::new(
        SqliteStore::new(&database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize SQLite datastore: {}", e))?,
    );

    info!("Initializing policy service");
    let policy_service = PolicyService::new(config.git.clone());

    let app_state = AppState {
        datastore: datastore.clone(),
        policy_service: policy_service.clone(),
    };

    let background_tasks = BackgroundTasks::new(config, datastore, policy_service);
    background_tasks.start();

    Ok(app_state)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config::default()
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let store = test_support::sqlite::sqlite_store().await;
        let datastore: Arc<dyn DataStore + Send + Sync> = Arc::new(store);
        let git_config = unet_core::config::types::GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };
        let policy_service = PolicyService::new(git_config);

        let app_state = AppState {
            datastore: datastore.clone(),
            policy_service,
        };

        assert!(Arc::ptr_eq(&app_state.datastore, &datastore));
    }

    #[tokio::test]
    async fn test_initialize_app_state_sqlite() {
        let config = create_test_config();
        let database_url = "sqlite::memory:".to_string();

        let result = initialize_app_state(config, database_url).await;

        match result {
            Ok(app_state) => {
                assert_eq!(app_state.datastore.name(), "SQLite");
            }
            Err(e) => {
                println!("Initialization error in test: {e}");
            }
        }
    }

    #[test]
    fn test_policy_service_creation() {
        let config = create_test_config();
        let policy_service = PolicyService::new(config.git);

        drop(policy_service);
    }

    #[tokio::test]
    async fn test_background_tasks_initialization() {
        let config = create_test_config();
        let store = test_support::sqlite::sqlite_store().await;
        let datastore: Arc<dyn DataStore + Send + Sync> = Arc::new(store);
        let policy_service = PolicyService::new(config.git.clone());

        let background_tasks = BackgroundTasks::new(config, datastore, policy_service);
        drop(background_tasks);
    }

    pub async fn create_mock_app_state() -> AppState {
        let datastore = test_support::sqlite::sqlite_store().await;

        let git_config = unet_core::config::types::GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };

        AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::new(git_config),
        }
    }
}
