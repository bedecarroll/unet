use std::sync::Once;
use tracing_subscriber::EnvFilter;

static INIT: Once = Once::new();

/// Initialize tracing subscriber once for tests.
pub fn init_tracing_once() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

