/// Import statistics tracking and reporting
use anyhow::Result;

/// Statistics tracking for import operations
#[derive(Clone)]
pub struct ImportStats {
    success_count: usize,
    error_count: usize,
    errors: Vec<String>,
}

impl ImportStats {
    /// Create new import statistics tracker
    pub const fn new() -> Self {
        Self {
            success_count: 0,
            error_count: 0,
            errors: Vec::new(),
        }
    }

    /// Record a successful import operation
    pub const fn record_success(&mut self) {
        self.success_count += 1;
    }

    /// Record a failed import operation
    pub fn record_error(&mut self, error_msg: String) {
        self.error_count += 1;
        self.errors.push(error_msg);
    }

    /// Get success count
    pub const fn success_count(&self) -> usize {
        self.success_count
    }

    /// Get error count
    pub const fn error_count(&self) -> usize {
        self.error_count
    }
}

/// Summary of import operation for output
#[derive(serde::Serialize)]
pub struct ImportSummary {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
    pub dry_run: bool,
}

impl ImportSummary {
    /// Create import summary from stats and `dry_run` flag
    pub fn new(stats: &ImportStats, dry_run: bool) -> Self {
        Self {
            success_count: stats.success_count,
            error_count: stats.error_count,
            errors: stats.errors.clone(),
            dry_run,
        }
    }
}

/// Process a single import item with error handling
pub async fn process_import_item<F, Fut>(
    import_fn: F,
    item_description: &str,
    continue_on_error: bool,
    stats: &mut ImportStats,
) -> Result<()>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    match import_fn().await {
        Ok(()) => stats.record_success(),
        Err(e) => {
            let error_msg = format!("Failed to import {item_description}: {e}");
            stats.record_error(error_msg.clone());
            tracing::warn!("{}", error_msg);

            if !continue_on_error {
                return Err(anyhow::anyhow!("Import failed: {}", error_msg));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_import_stats_new() {
        let stats = ImportStats::new();
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.error_count, 0);
        assert!(stats.errors.is_empty());
    }

    #[tokio::test]
    async fn test_import_stats_record_success() {
        let mut stats = ImportStats::new();
        stats.record_success();
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.error_count, 0);
        assert!(stats.errors.is_empty());
    }

    #[tokio::test]
    async fn test_import_stats_record_error() {
        let mut stats = ImportStats::new();
        stats.record_error("Test error".to_string());
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.errors.len(), 1);
        assert_eq!(stats.errors[0], "Test error");
    }

    #[tokio::test]
    async fn test_import_summary_new() {
        let mut stats = ImportStats::new();
        stats.record_success();
        stats.record_error("Test error".to_string());

        let summary = ImportSummary::new(&stats, true);
        assert_eq!(summary.success_count, 1);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.errors.len(), 1);
        assert!(summary.dry_run);
    }
}
