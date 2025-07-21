//! Task scheduling and evaluation statistics

/// Statistics for policy evaluation cycles
pub struct EvaluationStats {
    total_results: usize,
    successful_evaluations: usize,
    failed_evaluations: usize,
}

impl EvaluationStats {
    pub const fn new() -> Self {
        Self {
            total_results: 0,
            successful_evaluations: 0,
            failed_evaluations: 0,
        }
    }

    pub const fn record_success(&mut self, result_count: usize) {
        self.total_results += result_count;
        self.successful_evaluations += 1;
    }

    pub const fn record_failure(&mut self) {
        self.failed_evaluations += 1;
    }

    pub const fn total_results(&self) -> usize {
        self.total_results
    }

    pub const fn successful_evaluations(&self) -> usize {
        self.successful_evaluations
    }

    pub const fn failed_evaluations(&self) -> usize {
        self.failed_evaluations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_evaluation_stats_new() {
        let stats = EvaluationStats::new();
        assert_eq!(stats.total_results(), 0);
        assert_eq!(stats.successful_evaluations(), 0);
        assert_eq!(stats.failed_evaluations(), 0);
    }

    #[tokio::test]
    async fn test_evaluation_stats_record_success() {
        let mut stats = EvaluationStats::new();
        stats.record_success(5);
        assert_eq!(stats.total_results(), 5);
        assert_eq!(stats.successful_evaluations(), 1);
        assert_eq!(stats.failed_evaluations(), 0);
    }

    #[tokio::test]
    async fn test_evaluation_stats_record_failure() {
        let mut stats = EvaluationStats::new();
        stats.record_failure();
        assert_eq!(stats.total_results(), 0);
        assert_eq!(stats.successful_evaluations(), 0);
        assert_eq!(stats.failed_evaluations(), 1);
    }
}
