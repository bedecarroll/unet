//! Template Integration for μNet Core
//!
//! This module provides integration between the template engine and the data layer,
//! enabling template loading from Git repositories and management of template lifecycle.

use anyhow::Result;
use minijinja;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

use crate::config::GitConfig;
use crate::template::{TemplateEnvironment, TemplateLoader};

/// Template service that orchestrates template loading, Git sync, and validation
#[derive(Debug)]
pub struct TemplateService {
    loader: TemplateLoader,
    environment: Arc<TemplateEnvironment>,
}

impl TemplateService {
    /// Creates a new template service with Git configuration
    pub fn new(git_config: GitConfig) -> Result<Self> {
        let loader = if let Some(ref templates_repo) = git_config.templates_repo {
            let local_path = PathBuf::from("./git-repos/templates");
            TemplateLoader::with_git_repository(
                templates_repo.clone(),
                git_config.branch.clone(),
                None, // Use root of repository as template directory
                local_path,
            )?
        } else {
            TemplateLoader::new()?
        };

        let environment = Arc::new(TemplateEnvironment::new()?);

        Ok(Self {
            loader,
            environment,
        })
    }

    /// Creates a new template service with local directories
    pub fn with_local_directories(template_dirs: Vec<PathBuf>) -> Result<Self> {
        let loader = TemplateLoader::with_directories(template_dirs);
        let environment = Arc::new(TemplateEnvironment::new()?);

        Ok(Self {
            loader,
            environment,
        })
    }

    /// Creates a new template service with custom environment
    pub fn with_environment(
        git_config: GitConfig,
        environment: Arc<TemplateEnvironment>,
    ) -> Result<Self> {
        let loader = if let Some(ref templates_repo) = git_config.templates_repo {
            let local_path = PathBuf::from("./git-repos/templates");
            TemplateLoader::with_git_repository(
                templates_repo.clone(),
                git_config.branch.clone(),
                None,
                local_path,
            )?
        } else {
            TemplateLoader::new()?
        };

        Ok(Self {
            loader,
            environment,
        })
    }

    /// Lists all available templates
    pub async fn list_templates(&self) -> Result<Vec<String>> {
        self.loader.list_templates().await
    }

    /// Loads a template by name
    pub async fn load_template(&self, template_name: &str) -> Result<String> {
        self.loader.load_template(template_name).await
    }

    /// Checks if a template exists
    pub async fn template_exists(&self, template_name: &str) -> bool {
        self.loader.template_exists(template_name).await
    }

    /// Validates a single template
    pub async fn validate_template(&self, template_name: &str) -> Result<(bool, Option<String>)> {
        self.loader.validate_template(template_name).await
    }

    /// Validates all templates
    pub async fn validate_all_templates(&self) -> Result<Vec<(String, bool, Option<String>)>> {
        let templates = self.list_templates().await?;
        self.loader.validate_templates(&templates).await
    }

    /// Validates all templates with dependency resolution
    pub async fn validate_all_templates_with_dependencies(
        &self,
    ) -> Result<Vec<(String, bool, Option<String>)>> {
        let templates = self.list_templates().await?;
        self.loader
            .validate_templates_with_dependencies(&templates)
            .await
    }

    /// Extracts dependencies for a specific template
    pub async fn get_template_dependencies(&self, template_name: &str) -> Result<Vec<String>> {
        let content = self.load_template(template_name).await?;
        Ok(self.loader.extract_dependencies(&content))
    }

    /// Resolves the full dependency chain for a template
    pub async fn resolve_template_dependencies(&self, template_name: &str) -> Result<Vec<String>> {
        self.loader.resolve_dependencies(template_name).await
    }

    /// Loads all templates with their dependencies into the shared environment
    pub async fn load_all_templates_with_dependencies(&mut self) -> Result<Vec<String>> {
        // Need to make environment mutable
        let env_arc = Arc::clone(&self.environment);
        // This is tricky because Arc<TemplateEnvironment> is not mutable
        // We need to refactor the service to use Arc<RwLock<TemplateEnvironment>>
        // For now, let's create a new environment and replace it
        let mut new_env = crate::template::environment::TemplateEnvironment::new()?;
        let loaded_templates = self
            .loader
            .load_templates_with_dependencies(&mut new_env)
            .await?;

        // Note: In a real implementation, we'd want to use Arc<RwLock<TemplateEnvironment>>
        // to allow proper mutation of the shared environment
        self.environment = Arc::new(new_env);

        Ok(loaded_templates)
    }

    /// Sync templates from Git and reload them with validation
    pub async fn sync_and_reload_templates(&self) -> Result<Vec<String>> {
        let valid_templates = self.loader.sync_and_reload().await?;

        info!(
            "Synced and validated {} templates from Git",
            valid_templates.len()
        );

        // Log validation results
        let all_templates = self.list_templates().await?;
        let failed_count = all_templates.len() - valid_templates.len();

        if failed_count > 0 {
            warn!(
                "Template sync completed with {} validation failures out of {} total templates",
                failed_count,
                all_templates.len()
            );
        }

        Ok(valid_templates)
    }

    /// Force reload templates from source (clears cache)
    pub async fn reload_templates(&self) -> Result<Vec<String>> {
        // Clear cache
        self.loader.clear_cache().await;

        // Sync if Git is configured, otherwise just validate local templates
        let valid_templates = if self.is_git_configured() {
            self.loader.sync_and_reload().await?
        } else {
            let templates = self.list_templates().await?;
            let validation_results = self.loader.validate_templates(&templates).await?;

            validation_results
                .into_iter()
                .filter_map(|(name, is_valid, _)| if is_valid { Some(name) } else { None })
                .collect()
        };

        info!("Force reloaded {} templates", valid_templates.len());

        Ok(valid_templates)
    }

    /// Sync templates from Git repository (if configured)
    pub async fn sync_templates(&self) -> Result<()> {
        self.loader.sync_templates_from_git().await
    }

    /// Renders a template with the given context
    pub async fn render_template(
        &self,
        template_name: &str,
        context: &serde_json::Value,
    ) -> Result<String> {
        let template_content = self.load_template(template_name).await?;
        let minijinja_value = minijinja::Value::from_serialize(context);
        self.environment
            .render_str(&template_content, &minijinja_value)
    }

    /// Gets the template loader (for accessing cached templates, etc.)
    pub fn loader(&self) -> &TemplateLoader {
        &self.loader
    }

    /// Gets the template environment (for rendering, etc.)
    pub fn environment(&self) -> &TemplateEnvironment {
        &self.environment
    }

    /// Clears the template cache
    pub async fn clear_cache(&self) {
        self.loader.clear_cache().await;
    }

    /// Gets cache statistics
    pub async fn cache_stats(&self) -> (usize, Vec<String>) {
        self.loader.cache_stats().await
    }

    /// Checks if Git is configured for this service
    fn is_git_configured(&self) -> bool {
        self.loader.is_git_configured()
    }
}

impl Clone for TemplateService {
    fn clone(&self) -> Self {
        Self {
            loader: self.loader.clone(),
            environment: Arc::clone(&self.environment),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_template_dir() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let template_dir = temp_dir.path().to_path_buf();

        // Create test templates
        fs::write(template_dir.join("test.j2"), "Hello {{ name }}!").await?;
        fs::write(
            template_dir.join("cisco.jinja2"),
            "interface {{ interface }}\n ip address {{ ip }} {{ netmask }}",
        )
        .await?;

        Ok((temp_dir, template_dir))
    }

    #[tokio::test]
    async fn test_template_service_creation() {
        let git_config = GitConfig {
            policies_repo: None,
            templates_repo: None,
            branch: "main".to_string(),
            sync_interval: 300,
        };

        let service = TemplateService::new(git_config);
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_template_service_with_local_directories() {
        let (_temp_dir, template_dir) = create_test_template_dir().await.unwrap();
        let service = TemplateService::with_local_directories(vec![template_dir]).unwrap();

        let templates = service.list_templates().await.unwrap();
        assert!(templates.contains(&"test".to_string()));
        assert!(templates.contains(&"cisco".to_string()));
    }

    #[tokio::test]
    async fn test_template_validation() {
        let (_temp_dir, template_dir) = create_test_template_dir().await.unwrap();
        let service = TemplateService::with_local_directories(vec![template_dir]).unwrap();

        let (is_valid, error) = service.validate_template("test").await.unwrap();
        assert!(is_valid);
        assert!(error.is_none());

        let validation_results = service.validate_all_templates().await.unwrap();
        assert!(validation_results.len() >= 2);

        for (_, is_valid, _) in validation_results {
            assert!(is_valid);
        }
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let (_temp_dir, template_dir) = create_test_template_dir().await.unwrap();
        let service = TemplateService::with_local_directories(vec![template_dir]).unwrap();

        let context = serde_json::json!({
            "name": "World"
        });

        let result = service.render_template("test", &context).await.unwrap();
        assert_eq!(result, "Hello World!");
    }
}
