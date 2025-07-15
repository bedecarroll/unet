//! Configuration utility functions

use super::core::Config;

impl Config {
    /// Get the effective database URL, considering environment variable overrides
    ///
    /// This method first checks for the `DATABASE_URL` environment variable,
    /// falling back to the configured database URL if not found.
    ///
    /// # Returns
    /// The database URL to use for connections
    #[must_use]
    pub fn database_url(&self) -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| self.database.url.clone())
    }

    /// Check if the configuration represents a development environment
    ///
    /// This is determined by checking if the database URL uses `SQLite`
    /// and the server is bound to localhost.
    ///
    /// # Returns
    /// `true` if this appears to be a development configuration
    #[must_use]
    pub fn is_development(&self) -> bool {
        self.database_url().starts_with("sqlite:")
            && (self.server.host == "127.0.0.1" || self.server.host == "localhost")
    }

    /// Check if the configuration represents a production environment
    ///
    /// This is the inverse of `is_development()`.
    ///
    /// # Returns
    /// `true` if this appears to be a production configuration
    #[must_use]
    pub fn is_production(&self) -> bool {
        !self.is_development()
    }
}
