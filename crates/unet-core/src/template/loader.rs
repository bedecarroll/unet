//! Template loader for loading templates from various sources

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info};

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
#[derive(Debug)]
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
    /// Note: Full Git integration will be implemented in Milestone 6
    pub fn with_git_repository(
        url: String,
        ref_spec: String,
        template_dir: Option<String>,
        local_clone_path: PathBuf,
    ) -> Result<Self> {
        info!("Template loader configured for Git repository: {}", url);

        // TODO: Implement Git repository cloning and management in Milestone 6
        // For now, return an error indicating this feature is not yet implemented
        Err(anyhow::anyhow!(
            "Git repository loading is planned for Milestone 6. Use local directory loading for now."
        ))
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
            TemplateSource::Git { .. } => {
                // TODO: Implement Git template resolution in Milestone 6
                Err(anyhow!(
                    "Git repository template loading is not yet implemented. Use local directories for now."
                ))
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
            TemplateSource::Git { .. } => {
                // TODO: Implement Git template listing in Milestone 6
                Err(anyhow!(
                    "Git repository template listing is not yet implemented. Use local directories for now."
                ))
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
    async fn test_git_repository_not_implemented() {
        let result = TemplateLoader::with_git_repository(
            "https://github.com/example/templates.git".to_string(),
            "main".to_string(),
            Some("templates".to_string()),
            PathBuf::from("/tmp/git-templates"),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Milestone 6"));
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
        let _content = loader.load_template("test").await.unwrap();

        let (count, templates) = loader.cache_stats().await;
        assert_eq!(count, 1);
        assert!(templates.contains(&"test".to_string()));

        // Clear cache
        loader.clear_cache().await;
        let (count, _) = loader.cache_stats().await;
        assert_eq!(count, 0);
    }
}
