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

#[cfg(test)]
mod tests {
    use super::super::types::{DatabaseConfig, ServerConfig};
    use super::Config;

    fn create_test_config() -> Config {
        Config {
            database: DatabaseConfig {
                url: "sqlite:test.db".to_string(),
                max_connections: Some(10),
                timeout: Some(30),
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_request_size: 1024,
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_database_url_from_config() {
        let config = create_test_config();
        assert_eq!(config.database_url(), "sqlite:test.db");
    }

    #[test]
    fn test_database_url_from_env() {
        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        }
        let config = create_test_config();
        assert_eq!(config.database_url(), "postgres://localhost/test");
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn test_database_url_fallback() {
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
        let config = create_test_config();
        assert_eq!(config.database_url(), "sqlite:test.db");
    }

    #[test]
    fn test_is_development_sqlite_localhost() {
        let mut config = create_test_config();
        config.database.url = "sqlite:dev.db".to_string();
        config.server.host = "127.0.0.1".to_string();
        assert!(config.is_development());
    }

    #[test]
    fn test_is_development_sqlite_localhost_name() {
        let mut config = create_test_config();
        config.database.url = "sqlite:dev.db".to_string();
        config.server.host = "localhost".to_string();
        assert!(config.is_development());
    }

    #[test]
    fn test_is_development_postgres_localhost() {
        let mut config = create_test_config();
        config.database.url = "postgres://localhost/test".to_string();
        config.server.host = "127.0.0.1".to_string();
        assert!(!config.is_development());
    }

    #[test]
    fn test_is_development_sqlite_remote() {
        let mut config = create_test_config();
        config.database.url = "sqlite:prod.db".to_string();
        config.server.host = "0.0.0.0".to_string();
        assert!(!config.is_development());
    }

    #[test]
    fn test_is_development_postgres_remote() {
        let mut config = create_test_config();
        config.database.url = "postgres://prod-db/app".to_string();
        config.server.host = "0.0.0.0".to_string();
        assert!(!config.is_development());
    }

    #[test]
    fn test_is_production_inverse_of_development() {
        let dev_config = create_test_config();
        assert_eq!(dev_config.is_production(), !dev_config.is_development());

        let mut prod_config = create_test_config();
        prod_config.database.url = "postgres://prod/app".to_string();
        prod_config.server.host = "0.0.0.0".to_string();
        assert_eq!(prod_config.is_production(), !prod_config.is_development());
    }

    #[test]
    fn test_database_url_with_env_override() {
        // Test that DATABASE_URL env var takes precedence
        unsafe {
            std::env::set_var("DATABASE_URL", "mysql://env-override/db");
        }

        let mut config = create_test_config();
        config.database.url = "sqlite:config.db".to_string();

        assert_eq!(config.database_url(), "mysql://env-override/db");

        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
        assert_eq!(config.database_url(), "sqlite:config.db");
    }

    #[test]
    fn test_edge_cases_host_matching() {
        let mut config = create_test_config();
        config.database.url = "sqlite:test.db".to_string();

        // Test various host formats
        config.server.host = "127.0.0.1".to_string();
        assert!(config.is_development());

        config.server.host = "localhost".to_string();
        assert!(config.is_development());

        config.server.host = "127.0.0.2".to_string();
        assert!(!config.is_development());

        config.server.host = "localdev".to_string();
        assert!(!config.is_development());
    }

    #[test]
    fn test_database_url_cases() {
        let mut config = create_test_config();

        // Test SQLite variations
        config.database.url = "sqlite:memory:".to_string();
        assert!(config.database_url().starts_with("sqlite:"));

        config.database.url = "sqlite:///path/to/db".to_string();
        assert!(config.database_url().starts_with("sqlite:"));

        // Test other database types
        config.database.url = "postgresql://localhost/db".to_string();
        assert!(config.database_url().starts_with("postgresql:"));

        config.database.url = "mysql://localhost/db".to_string();
        assert!(config.database_url().starts_with("mysql:"));
    }
}
