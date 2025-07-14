//! Configuration default values and constants

/// Network configuration constants
pub mod network {
    /// Default HTTP server port
    pub const DEFAULT_SERVER_PORT: u16 = 8080;
    /// Default SNMP port
    pub const SNMP_DEFAULT_PORT: u16 = 161;
    /// Default SNMP trap port
    pub const SNMP_TRAP_PORT: u16 = 162;
    /// Localhost IP address
    pub const LOCALHOST: &str = "127.0.0.1";
    /// Default localhost with SNMP port
    pub const LOCALHOST_SNMP: &str = "127.0.0.1:161";
}

/// Database configuration constants
pub mod database {
    /// Default database URL for `SQLite`
    pub const DEFAULT_DATABASE_URL: &str = "sqlite:./unet.db?mode=rwc";
    /// Default maximum database connections
    pub const DEFAULT_DB_MAX_CONNECTIONS: u32 = 10;
    /// Default database timeout in seconds
    pub const DEFAULT_DB_TIMEOUT_SECONDS: u64 = 30;
    /// Minimum allowed database connections
    pub const MIN_DB_CONNECTIONS: u32 = 1;
    /// Maximum allowed database connections
    pub const MAX_DB_CONNECTIONS: u32 = 100;
}

/// SNMP configuration constants
pub mod snmp {
    /// Default SNMP community string
    pub const DEFAULT_SNMP_COMMUNITY: &str = "public";
    /// Default SNMP timeout in seconds
    pub const DEFAULT_SNMP_TIMEOUT_SECONDS: u64 = 5;
    /// Default SNMP retries
    pub const DEFAULT_SNMP_RETRIES: u8 = 3;
    /// Minimum SNMP timeout in seconds
    pub const MIN_SNMP_TIMEOUT_SECONDS: u64 = 1;
    /// Maximum SNMP timeout in seconds
    pub const MAX_SNMP_TIMEOUT_SECONDS: u64 = 60;
    /// Maximum allowed SNMP retries
    pub const MAX_SNMP_RETRIES: u8 = 10;
}

/// Server configuration constants
pub mod server {
    /// Default server host
    pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
    /// Default maximum request size in bytes (16MB)
    pub const DEFAULT_MAX_REQUEST_SIZE: usize = 16 * 1024 * 1024;
    /// Minimum request size in bytes (1KB)
    pub const MIN_REQUEST_SIZE: usize = 1024;
    /// Maximum request size in bytes (100MB)
    pub const MAX_REQUEST_SIZE: usize = 100 * 1024 * 1024;
}

/// Performance tuning constants
pub mod performance {
    /// Default batch size for bulk operations
    pub const DEFAULT_BATCH_SIZE: usize = 100;
    /// Maximum batch size for bulk operations
    pub const MAX_BATCH_SIZE: usize = 1000;
    /// Default worker thread count
    pub const DEFAULT_WORKER_THREADS: usize = 4;
    /// Default SNMP connection pool size
    pub const DEFAULT_CONNECTION_POOL_SIZE: usize = 10;
    /// Maximum connection pool size
    pub const MAX_CONNECTION_POOL_SIZE: usize = 100;
}

/// Cache configuration constants
pub mod cache {
    /// Default cache TTL in seconds (5 minutes)
    pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 300;
    /// Default cache size limit (10MB)
    pub const DEFAULT_CACHE_SIZE_BYTES: usize = 10 * 1024 * 1024;
}

/// Logging configuration constants
pub mod logging {
    /// Default log level
    pub const DEFAULT_LOG_LEVEL: &str = "info";
    /// Default log format
    pub const DEFAULT_LOG_FORMAT: &str = "text";
}
