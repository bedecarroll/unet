//! Transaction and rollback support for policy actions
//!
//! Provides rollback functionality for policy actions to ensure atomicity
//! and allow recovery from failed policy executions.

/// Result of transaction rollback
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RollbackResult {
    /// Number of actions successfully rolled back
    pub actions_rolled_back: usize,
    /// Number of rollback operations that failed
    pub rollback_failures: usize,
    /// List of error messages from failed rollbacks
    pub error_messages: Vec<String>,
    /// Whether the rollback was fully successful
    pub success: bool,
}

impl RollbackResult {
    /// Create a new successful rollback result
    #[must_use]
    pub const fn success(actions_count: usize) -> Self {
        Self {
            actions_rolled_back: actions_count,
            rollback_failures: 0,
            error_messages: Vec::new(),
            success: true,
        }
    }

    /// Create a new failed rollback result
    #[must_use]
    pub const fn failure(
        actions_rolled_back: usize,
        rollback_failures: usize,
        error_messages: Vec<String>,
    ) -> Self {
        Self {
            actions_rolled_back,
            rollback_failures,
            error_messages,
            success: false,
        }
    }

    /// Add an error message to the rollback result
    pub fn add_error(&mut self, error: String) {
        self.error_messages.push(error);
        self.rollback_failures += 1;
        self.success = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_result_success() {
        let result = RollbackResult::success(5);
        assert_eq!(result.actions_rolled_back, 5);
        assert_eq!(result.rollback_failures, 0);
        assert!(result.error_messages.is_empty());
        assert!(result.success);
    }

    #[test]
    fn test_rollback_result_failure() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = RollbackResult::failure(3, 2, errors.clone());
        assert_eq!(result.actions_rolled_back, 3);
        assert_eq!(result.rollback_failures, 2);
        assert_eq!(result.error_messages, errors);
        assert!(!result.success);
    }

    #[test]
    fn test_rollback_result_add_error() {
        let mut result = RollbackResult::success(1);
        assert!(result.success);

        result.add_error("Something went wrong".to_string());
        assert!(!result.success);
        assert_eq!(result.rollback_failures, 1);
        assert_eq!(result.error_messages.len(), 1);
    }
}
