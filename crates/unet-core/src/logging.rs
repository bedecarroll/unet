//! Logging and tracing infrastructure for μNet Core
//!
//! This module provides structured logging and tracing capabilities using the `tracing`
//! ecosystem with support for multiple output formats and log levels.

use crate::config::LoggingConfig;
use crate::error::Result;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

/// Initializes the global tracing subscriber based on configuration
pub fn init_tracing(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = Registry::default().with(env_filter);

    match config.format.as_str() {
        "json" => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true);

            registry.with(json_layer).init();
        }
        "pretty" | _ => {
            let pretty_layer = tracing_subscriber::fmt::layer()
                .pretty()
                .with_thread_ids(true)
                .with_thread_names(true);

            registry.with(pretty_layer).init();
        }
    }

    tracing::info!("Tracing initialized with level: {}", config.level);
    Ok(())
}

/// Initializes tracing with default pretty format and info level
pub fn init_default_tracing() -> Result<()> {
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "pretty".to_string(),
        file: None,
    };
    init_tracing(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_default_tracing() {
        // This test just ensures the function doesn't panic
        // We can't easily test the actual output without more complex setup
        let result = init_default_tracing();
        assert!(result.is_ok());
    }
}