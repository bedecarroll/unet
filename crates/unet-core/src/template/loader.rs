//! Template loader for loading templates from various sources

use anyhow::{Context, Result, anyhow};
use futures;
use regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Template cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    content: String,
    last_modified: std::time::SystemTime,
    path: PathBuf,
}

/// Template source configuration
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Load templates from local filesystem directories
    Local(Vec<PathBuf>),
    /// Load templates from Git repository (planned for Milestone 6)
    Git {
        /// Git repository URL
        url: String,
        /// Branch or commit to use
        ref_spec: String,
        /// Subdirectory within the repository containing templates
        template_dir: Option<String>,
        /// Local path for cloning the repository
        local_clone_path: PathBuf,
    },
}

/// Template loader with caching and hot-reloading support
#[derive(Debug, Clone)]
pub struct TemplateLoader {
    source: TemplateSource,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    hot_reload: bool,
}

impl TemplateLoader {
    /// Create a new template loader with default local directories
    pub fn new() -> Result<Self> {
        let template_dirs = vec![
            PathBuf::from("templates"),
            PathBuf::from("./templates"),
            PathBuf::from("../templates"),
        ];

        info!(
            "Template loader initialized with directories: {:?}",
            template_dirs
        );

        Ok(Self {
            source: TemplateSource::Local(template_dirs),
            cache: Arc::new(RwLock::new(HashMap::new())),
            hot_reload: true,
        })
    }

    /// Create a new template loader with custom local directories
    pub fn with_directories(template_dirs: Vec<PathBuf>) -> Self {
        info!(
            "Template loader initialized with custom directories: {:?}",
            template_dirs
        );

        Self {
            source: TemplateSource::Local(template_dirs),
            cache: Arc::new(RwLock::new(HashMap::new())),
            hot_reload: true,
        }
    }

    /// Create a new template loader with Git repository source
    pub fn with_git_repository(
        url: String,
        ref_spec: String,
        template_dir: Option<String>,
        local_clone_path: PathBuf,
    ) -> Result<Self> {
        info!("Template loader configured for Git repository: {}", url);

        Ok(Self {
            source: TemplateSource::Git {
                url,
                ref_spec,
                template_dir,
                local_clone_path,
            },
            cache: Arc::new(RwLock::new(HashMap::new())),
            hot_reload: true,
        })
    }

    /// Enable or disable hot reloading
    pub fn set_hot_reload(&mut self, enabled: bool) {
        self.hot_reload = enabled;
        if enabled {
            info!("Hot reloading enabled");
        } else {
            info!("Hot reloading disabled");
        }
    }

    /// Load a template by name
    pub async fn load_template(&self, template_name: &str) -> Result<String> {
        // Check cache first if hot reload is disabled
        if !self.hot_reload {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(template_name) {
                debug!("Template '{}' loaded from cache", template_name);
                return Ok(entry.content.clone());
            }
        }

        // Find template file
        let template_path = self.find_template_file(template_name).await?;

        // Check if we need to reload from cache
        if self.hot_reload {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(template_name) {
                // Check if file has been modified
                if let Ok(metadata) = fs::metadata(&template_path).await {
                    if let Ok(modified) = metadata.modified() {
                        if modified <= entry.last_modified {
                            debug!(
                                "Template '{}' loaded from cache (not modified)",
                                template_name
                            );
                            return Ok(entry.content.clone());
                        }
                    }
                }
            }
        }

        // Load template from file
        let content = fs::read_to_string(&template_path).await.with_context(|| {
            format!("Failed to read template file: {}", template_path.display())
        })?;

        // Update cache
        let mut cache = self.cache.write().await;
        let last_modified = fs::metadata(&template_path)
            .await
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| std::time::SystemTime::now());

        cache.insert(
            template_name.to_string(),
            CacheEntry {
                content: content.clone(),
                last_modified,
                path: template_path.clone(),
            },
        );

        info!(
            "Template '{}' loaded from file: {}",
            template_name,
            template_path.display()
        );
        Ok(content)
    }

    /// Find a template file in the configured source
    async fn find_template_file(&self, template_name: &str) -> Result<PathBuf> {
        match &self.source {
            TemplateSource::Local(template_dirs) => {
                self.find_template_in_directories(template_name, template_dirs)
                    .await
            }
            TemplateSource::Git {
                url: _,
                ref_spec: _,
                template_dir,
                local_clone_path,
            } => {
                let template_path = if let Some(dir) = template_dir {
                    local_clone_path.join(dir)
                } else {
                    local_clone_path.clone()
                };

                self.find_template_in_directories(template_name, &[template_path])
                    .await
            }
        }
    }

    /// Find a template file in local directories
    async fn find_template_in_directories(
        &self,
        template_name: &str,
        directories: &[PathBuf],
    ) -> Result<PathBuf> {
        // Try different file extensions
        let extensions = ["", ".j2", ".jinja", ".jinja2", ".tmpl"];

        for dir in directories {
            for ext in &extensions {
                let filename = if ext.is_empty() {
                    template_name.to_string()
                } else {
                    format!("{}{}", template_name, ext)
                };

                let path = dir.join(&filename);
                if path.exists() {
                    debug!("Found template file: {}", path.display());
                    return Ok(path);
                }
            }
        }

        Err(anyhow!(
            "Template '{}' not found in any of the search directories: {:?}",
            template_name,
            directories
        ))
    }

    /// List all available templates
    pub async fn list_templates(&self) -> Result<Vec<String>> {
        match &self.source {
            TemplateSource::Local(template_dirs) => {
                self.list_templates_in_directories(template_dirs).await
            }
            TemplateSource::Git {
                url: _,
                ref_spec: _,
                template_dir,
                local_clone_path,
            } => {
                let template_path = if let Some(dir) = template_dir {
                    local_clone_path.join(dir)
                } else {
                    local_clone_path.clone()
                };

                self.list_templates_in_directories(&[template_path]).await
            }
        }
    }

    /// List templates in local directories
    async fn list_templates_in_directories(&self, directories: &[PathBuf]) -> Result<Vec<String>> {
        let mut templates = Vec::new();

        for dir in directories {
            if !dir.exists() {
                continue;
            }

            let mut entries = fs::read_dir(dir)
                .await
                .with_context(|| format!("Failed to read template directory: {}", dir.display()))?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        templates.push(name.to_string());
                    }
                }
            }
        }

        templates.sort();
        templates.dedup();
        Ok(templates)
    }

    /// Clear the template cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Template cache cleared");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, Vec<String>) {
        let cache = self.cache.read().await;
        let count = cache.len();
        let templates = cache.keys().cloned().collect();
        (count, templates)
    }

    /// Check if a template exists
    pub async fn template_exists(&self, template_name: &str) -> bool {
        self.find_template_file(template_name).await.is_ok()
    }

    /// Sync templates from Git repository (if configured)
    pub async fn sync_templates_from_git(&self) -> Result<()> {
        match &self.source {
            TemplateSource::Git {
                url,
                ref_spec,
                template_dir: _,
                local_clone_path,
            } => {
                debug!("Syncing templates from Git repository: {}", url);
                self.sync_git_repository(url, ref_spec, local_clone_path)
                    .await
            }
            TemplateSource::Local(_) => {
                debug!("Template loader is configured for local directories, skipping Git sync");
                Ok(())
            }
        }
    }

    /// Sync and reload templates with validation
    pub async fn sync_and_reload(&self) -> Result<Vec<String>> {
        info!("Syncing and reloading templates with validation");

        // Clear cache to force reload
        self.clear_cache().await;

        // Sync from Git if configured
        if let TemplateSource::Git { .. } = &self.source {
            self.sync_templates_from_git().await?;

            // Validate templates after sync
            let templates = self.list_templates().await?;

            // Validate templates with MiniJinja syntax checking
            let mut valid_templates = Vec::new();
            let validation_result = self.validate_templates(&templates).await?;

            for (template_name, is_valid, error) in validation_result {
                if is_valid {
                    debug!("Template '{}' validation passed", template_name);
                    valid_templates.push(template_name);
                } else {
                    warn!(
                        "Template '{}' validation failed: {}",
                        template_name,
                        error.unwrap_or_else(|| "Unknown error".to_string())
                    );
                }
            }

            info!(
                "Template validation completed: {}/{} templates valid",
                valid_templates.len(),
                templates.len()
            );

            Ok(valid_templates)
        } else {
            // For local directories, just reload and validate
            let templates = self.list_templates().await?;
            Ok(templates)
        }
    }

    /// Internal method to sync Git repository
    async fn sync_git_repository(
        &self,
        url: &str,
        ref_spec: &str,
        local_path: &PathBuf,
    ) -> Result<()> {
        use crate::git::{GitClient, GitClientConfig, GitRepository};

        // Clone for use in closure and logging
        let repo_url_for_closure = url.to_string();
        let repo_url_for_logging = url.to_string();
        let branch = ref_spec.to_string();
        let local_clone_path = local_path.clone();

        let result = tokio::task::spawn_blocking(move || {
            // Create a new GitClient for this operation
            let git_config = GitClientConfig {
                base_directory: std::path::PathBuf::from("./git-repos"),
                default_sync_interval: 30,
                max_state_age: 5,
                auto_fetch: true,
                auto_cleanup: false,
            };
            let git_client = GitClient::with_config(git_config);

            // Create base directory if it doesn't exist
            if !local_clone_path.parent().unwrap().exists() {
                std::fs::create_dir_all(local_clone_path.parent().unwrap())
                    .map_err(|e| format!("Failed to create git-repos directory: {}", e))?;
            }

            // Clone or open repository
            let credential_provider = git_client.credential_provider();
            let repository = if local_clone_path.exists() && local_clone_path.join(".git").exists()
            {
                debug!(
                    "Opening existing templates repository at: {}",
                    local_clone_path.display()
                );
                GitRepository::open(&local_clone_path, credential_provider)
                    .map_err(|e| format!("Failed to open existing repository: {}", e))?
            } else {
                debug!(
                    "Cloning templates repository to: {}",
                    local_clone_path.display()
                );
                GitRepository::clone(
                    &repo_url_for_closure,
                    &local_clone_path,
                    credential_provider,
                )
                .map_err(|e| format!("Failed to clone repository: {}", e))?
            };

            // Fetch latest changes and pull
            repository
                .fetch(None)
                .map_err(|e| format!("Failed to fetch from repository: {}", e))?;

            repository
                .pull(None, Some(&branch))
                .map_err(|e| format!("Failed to pull changes: {}", e))?;

            debug!("Successfully synced templates repository");
            Ok::<(), String>(())
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
        .map_err(|e| anyhow!("{}", e))?;

        info!(
            "Templates synced from Git repository: {}",
            repo_url_for_logging
        );
        Ok(result)
    }

    /// Validate templates using MiniJinja syntax checking
    pub async fn validate_templates(
        &self,
        template_names: &[String],
    ) -> Result<Vec<(String, bool, Option<String>)>> {
        use crate::template::environment::TemplateEnvironment;

        let mut results = Vec::new();
        let env = TemplateEnvironment::new()?;

        for template_name in template_names {
            let (is_valid, error) = match self.load_template(template_name).await {
                Ok(content) => {
                    // Try to validate the template with MiniJinja
                    match env.validate_template(&content) {
                        Ok(_) => {
                            debug!("Template '{}' syntax validation passed", template_name);
                            (true, None)
                        }
                        Err(e) => {
                            warn!(
                                "Template '{}' syntax validation failed: {}",
                                template_name, e
                            );
                            (false, Some(format!("Syntax error: {}", e)))
                        }
                    }
                }
                Err(e) => {
                    warn!("Template '{}' could not be loaded: {}", template_name, e);
                    (false, Some(format!("Load error: {}", e)))
                }
            };

            results.push((template_name.clone(), is_valid, error));
        }

        Ok(results)
    }

    /// Validate a single template by name
    pub async fn validate_template(&self, template_name: &str) -> Result<(bool, Option<String>)> {
        let results = self
            .validate_templates(&[template_name.to_string()])
            .await?;
        Ok(results
            .into_iter()
            .next()
            .map(|(_, is_valid, error)| (is_valid, error))
            .unwrap_or((false, Some("Template not found".to_string()))))
    }

    /// Check if this loader is configured for Git repository source
    pub fn is_git_configured(&self) -> bool {
        matches!(&self.source, TemplateSource::Git { .. })
    }

    /// Extract template dependencies from template content
    /// Finds {% include %}, {% extends %}, and {% import %} statements
    pub fn extract_dependencies(&self, template_content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Simple regex-based extraction for Jinja2 includes/extends/imports
        let include_patterns = [
            r#"\{\%\s*include\s+["']([^"']+)["']\s*\%\}"#,
            r#"\{\%\s*extends\s+["']([^"']+)["']\s*\%\}"#,
            r#"\{\%\s*import\s+["']([^"']+)["']\s*.*\%\}"#,
        ];

        for pattern in &include_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for captures in regex.captures_iter(template_content) {
                    if let Some(template_name) = captures.get(1) {
                        dependencies.push(template_name.as_str().to_string());
                    }
                }
            }
        }

        dependencies
    }

    /// Resolve template dependencies recursively
    /// Returns a list of templates in dependency order (dependencies first)
    pub async fn resolve_dependencies(&self, template_name: &str) -> Result<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        self.resolve_dependencies_recursive(
            template_name.to_string(),
            &mut resolved,
            &mut visited,
            &mut visiting,
        )
        .await?;

        Ok(resolved)
    }

    /// Recursive helper for dependency resolution with cycle detection
    fn resolve_dependencies_recursive<'a>(
        &'a self,
        template_name: String,
        resolved: &'a mut Vec<String>,
        visited: &'a mut std::collections::HashSet<String>,
        visiting: &'a mut std::collections::HashSet<String>,
    ) -> futures::future::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if visited.contains(&template_name) {
                return Ok(());
            }

            if visiting.contains(&template_name) {
                return Err(anyhow!(
                    "Circular dependency detected involving template: {}",
                    template_name
                ));
            }

            visiting.insert(template_name.clone());

            // Load the template content
            let template_content = self.load_template(&template_name).await?;
            let dependencies = self.extract_dependencies(&template_content);

            // Recursively resolve dependencies
            for dependency in dependencies {
                self.resolve_dependencies_recursive(dependency, resolved, visited, visiting)
                    .await?;
            }

            visiting.remove(&template_name);
            visited.insert(template_name.clone());
            resolved.push(template_name);

            Ok(())
        })
    }

    /// Load all templates with their dependencies into the environment
    pub async fn load_templates_with_dependencies(
        &self,
        environment: &mut crate::template::environment::TemplateEnvironment,
    ) -> Result<Vec<String>> {
        let all_templates = self.list_templates().await?;
        let mut loaded_templates = Vec::new();

        // Clear existing templates in the environment
        environment.clear_templates();

        // Process each template with its dependencies
        for template_name in &all_templates {
            let dependency_chain = self.resolve_dependencies(template_name).await?;

            // Load each template in the dependency chain
            for dep_template_name in dependency_chain {
                if !loaded_templates.contains(&dep_template_name) {
                    let content = self.load_template(&dep_template_name).await?;
                    environment.add_template(dep_template_name.clone(), content)?;
                    loaded_templates.push(dep_template_name.clone());
                    debug!("Loaded template '{}' into environment", dep_template_name);
                }
            }
        }

        info!(
            "Loaded {} templates with dependencies into environment",
            loaded_templates.len()
        );
        Ok(loaded_templates)
    }

    /// Validate templates with dependency resolution
    pub async fn validate_templates_with_dependencies(
        &self,
        template_names: &[String],
    ) -> Result<Vec<(String, bool, Option<String>)>> {
        use crate::template::environment::TemplateEnvironment;

        let mut env = TemplateEnvironment::new()?;
        let mut results = Vec::new();

        // First, load all templates into the environment to support dependencies
        for template_name in template_names {
            if let Ok(content) = self.load_template(template_name).await {
                let _ = env.add_template(template_name.clone(), content);
            }
        }

        // Now validate each template with full dependency context
        for template_name in template_names {
            let (is_valid, error) = match self.load_template(template_name).await {
                Ok(_) => {
                    // Template exists and can be loaded, now try to get it from environment
                    match env.has_template(template_name) {
                        true => {
                            debug!(
                                "Template '{}' validation passed (with dependencies)",
                                template_name
                            );
                            (true, None)
                        }
                        false => {
                            let error_msg = "Template not found in environment".to_string();
                            warn!(
                                "Template '{}' validation failed: {}",
                                template_name, error_msg
                            );
                            (false, Some(error_msg))
                        }
                    }
                }
                Err(e) => {
                    let error_msg = format!("Load error: {}", e);
                    warn!("Template '{}' could not be loaded: {}", template_name, e);
                    (false, Some(error_msg))
                }
            };

            results.push((template_name.clone(), is_valid, error));
        }

        Ok(results)
    }
}

impl Default for TemplateLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateLoader")
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
    async fn test_template_loader_creation() {
        let loader = TemplateLoader::new();
        assert!(loader.is_ok());
    }

    #[tokio::test]
    async fn test_load_template() {
        let (_temp_dir, template_dir) = create_test_template_dir().await.unwrap();
        let loader = TemplateLoader::with_directories(vec![template_dir]);

        let content = loader.load_template("test").await.unwrap();
        assert_eq!(content, "Hello {{ name }}!");

        let content = loader.load_template("cisco").await.unwrap();
        assert_eq!(
            content,
            "interface {{ interface }}\n ip address {{ ip }} {{ netmask }}"
        );
    }

    #[tokio::test]
    async fn test_template_not_found() {
        let loader = TemplateLoader::with_directories(vec![PathBuf::from("/nonexistent")]);
        let result = loader.load_template("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_templates() {
        let (_temp_dir, template_dir) = create_test_template_dir().await.unwrap();
        let loader = TemplateLoader::with_directories(vec![template_dir]);

        let templates = loader.list_templates().await.unwrap();
        assert!(templates.contains(&"test".to_string()));
        assert!(templates.contains(&"cisco".to_string()));
    }

    #[tokio::test]
    async fn test_git_repository_creation() {
        let result = TemplateLoader::with_git_repository(
            "https://github.com/example/templates.git".to_string(),
            "main".to_string(),
            Some("templates".to_string()),
            PathBuf::from("/tmp/git-templates"),
        );

        assert!(result.is_ok());
        let loader = result.unwrap();

        // Verify the Git source is configured correctly
        match loader.source {
            TemplateSource::Git {
                url,
                ref_spec,
                template_dir,
                ..
            } => {
                assert_eq!(url, "https://github.com/example/templates.git");
                assert_eq!(ref_spec, "main");
                assert_eq!(template_dir, Some("templates".to_string()));
            }
            _ => panic!("Expected Git source configuration"),
        }
    }

    #[tokio::test]
    async fn test_template_exists() {
        let (_temp_dir, template_dir) = create_test_template_dir().await.unwrap();
        let loader = TemplateLoader::with_directories(vec![template_dir]);

        assert!(loader.template_exists("test").await);
        assert!(loader.template_exists("cisco").await);
        assert!(!loader.template_exists("nonexistent").await);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let (_temp_dir, template_dir) = create_test_template_dir().await.unwrap();
        let loader = TemplateLoader::with_directories(vec![template_dir]);

        // Load template (should cache it)
        let content = loader.load_template("test").await.unwrap();

        let (count, templates) = loader.cache_stats().await;
        assert_eq!(count, 1);
        assert!(templates.contains(&"test".to_string()));

        // Clear cache
        loader.clear_cache().await;
        let (count, _) = loader.cache_stats().await;
        assert_eq!(count, 0);
    }
}
