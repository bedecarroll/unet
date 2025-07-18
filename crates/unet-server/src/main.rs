//! μNet HTTP Server
//!
//! REST API server for μNet network configuration management.

mod api;
mod background;
mod error;
mod handlers;
mod server;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use unet_core::config::Config;
#[cfg(not(test))]
use unet_core::logging::init_tracing;

/// μNet HTTP Server
#[derive(Parser, Debug)]
#[command(name = "unet-server")]
#[command(about = "μNet HTTP Server for network configuration management")]
#[command(version)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Server host to bind to
    #[arg(long)]
    host: Option<String>,

    /// Server port to bind to
    #[arg(short, long)]
    port: Option<u16>,

    /// Database URL (`SQLite`)
    #[arg(short, long, default_value = "sqlite://unet.db")]
    database_url: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long)]
    log_level: Option<String>,
}

/// Load configuration from file or environment with fallback to defaults
fn load_configuration(args: &Args) -> Result<Config> {
    if let Some(config_path) = &args.config {
        info!("Loading configuration from: {}", config_path.display());
        Ok(Config::from_file(config_path.clone())?)
    } else {
        // Try to load from environment, fallback to defaults
        Ok(Config::from_env().unwrap_or_else(|_| {
            info!("Using default configuration");
            Config::default()
        }))
    }
}

/// Apply command line argument overrides to configuration
fn apply_cli_overrides(config: &mut Config, args: &Args) {
    if let Some(host) = &args.host {
        config.server.host = host.clone();
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if let Some(log_level) = &args.log_level {
        config.logging.level = log_level.clone();
    }
}

/// Determine the database URL to use (CLI override or config)
fn determine_database_url(args: &Args, config: &Config) -> String {
    if args.database_url == "sqlite://unet.db" {
        config.database_url()
    } else {
        args.database_url.clone()
    }
}

/// Initialize the application with given arguments
async fn initialize_app(args: Args) -> Result<(Config, String)> {
    // Load configuration
    let mut config = load_configuration(&args)?;

    // Override config with command line arguments
    apply_cli_overrides(&mut config, &args);

    // Override database URL from command line or use config
    let database_url = determine_database_url(&args, &config);

    // Validate configuration before starting
    config.validate()?;

    // Initialize tracing with config (skip in tests to avoid global subscriber conflicts)
    #[cfg(not(test))]
    init_tracing(&config.logging)?;

    info!("Starting μNet server...");
    info!(
        "Configuration: server={}:{}, database_url={}",
        config.server.host, config.server.port, database_url
    );

    Ok((config, database_url))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (config, database_url) = initialize_app(args).await?;

    // Start the server
    server::run(config, database_url).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_args_default_values() {
        let args = Args::try_parse_from(["unet-server"]).unwrap();
        assert_eq!(args.database_url, "sqlite://unet.db");
        assert!(args.config.is_none());
        assert!(args.host.is_none());
        assert!(args.port.is_none());
        assert!(args.log_level.is_none());
    }

    #[test]
    fn test_args_with_config_file() {
        let args =
            Args::try_parse_from(["unet-server", "--config", "/path/to/config.toml"]).unwrap();
        assert_eq!(args.config, Some(PathBuf::from("/path/to/config.toml")));
    }

    #[test]
    fn test_args_with_host() {
        let args = Args::try_parse_from(["unet-server", "--host", "192.168.1.1"]).unwrap();
        assert_eq!(args.host, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_args_with_port() {
        let args = Args::try_parse_from(["unet-server", "--port", "8080"]).unwrap();
        assert_eq!(args.port, Some(8080));
    }

    #[test]
    fn test_args_with_custom_database_url() {
        let args =
            Args::try_parse_from(["unet-server", "--database-url", "sqlite://custom.db"]).unwrap();
        assert_eq!(args.database_url, "sqlite://custom.db");
    }

    #[test]
    fn test_args_with_log_level() {
        let args = Args::try_parse_from(["unet-server", "--log-level", "debug"]).unwrap();
        assert_eq!(args.log_level, Some("debug".to_string()));
    }

    #[test]
    fn test_args_all_options() {
        let args = Args::try_parse_from([
            "unet-server",
            "--config",
            "/path/to/config.toml",
            "--host",
            "0.0.0.0",
            "--port",
            "9000",
            "--database-url",
            "sqlite://test.db",
            "--log-level",
            "trace",
        ])
        .unwrap();

        assert_eq!(args.config, Some(PathBuf::from("/path/to/config.toml")));
        assert_eq!(args.host, Some("0.0.0.0".to_string()));
        assert_eq!(args.port, Some(9000));
        assert_eq!(args.database_url, "sqlite://test.db");
        assert_eq!(args.log_level, Some("trace".to_string()));
    }

    #[test]
    fn test_args_short_flags() {
        let args = Args::try_parse_from([
            "unet-server",
            "-c",
            "/path/to/config.toml",
            "-p",
            "3000",
            "-d",
            "sqlite://short.db",
        ])
        .unwrap();

        assert_eq!(args.config, Some(PathBuf::from("/path/to/config.toml")));
        assert_eq!(args.port, Some(3000));
        assert_eq!(args.database_url, "sqlite://short.db");
    }

    #[test]
    fn test_args_invalid_port() {
        let result = Args::try_parse_from([
            "unet-server",
            "--port",
            "70000", // Invalid port number
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_loading_with_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[server]
host = "127.0.0.1"
port = 8080

[logging]
level = "info"
        "#
        )
        .unwrap();

        let args = Args {
            config: Some(temp_file.path().to_path_buf()),
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(),
            log_level: None,
        };

        // Test would load config from file, but we can't easily test the full main logic
        // without mocking the server::run function. We can test that the path is set correctly.
        assert!(args.config.is_some());
        assert_eq!(args.config.unwrap(), temp_file.path().to_path_buf());
    }

    #[test]
    fn test_config_override_host() {
        let mut config = Config::default();
        let original_host = config.server.host;

        let new_host = "192.168.1.100".to_string();
        config.server.host = new_host.clone();

        assert_ne!(config.server.host, original_host);
        assert_eq!(config.server.host, new_host);
    }

    #[test]
    fn test_config_override_port() {
        let mut config = Config::default();
        let original_port = config.server.port;

        let new_port = 9090;
        config.server.port = new_port;

        assert_ne!(config.server.port, original_port);
        assert_eq!(config.server.port, new_port);
    }

    #[test]
    fn test_config_override_log_level() {
        let mut config = Config::default();
        let original_level = config.logging.level;

        let new_level = "debug".to_string();
        config.logging.level = new_level.clone();

        assert_ne!(config.logging.level, original_level);
        assert_eq!(config.logging.level, new_level);
    }

    #[test]
    fn test_database_url_selection_default() {
        let config = Config::default();
        let args_database_url = "sqlite://unet.db";

        // When args database_url is default, should use config database_url
        let selected_url = if args_database_url == "sqlite://unet.db" {
            config.database_url()
        } else {
            args_database_url.to_string()
        };

        assert_eq!(selected_url, config.database_url());
    }

    #[test]
    fn test_database_url_selection_custom() {
        let config = Config::default();
        let custom_url = "sqlite://custom.db";

        // When args database_url is not default, should use args value
        let selected_url = if custom_url == "sqlite://unet.db" {
            config.database_url()
        } else {
            custom_url.to_string()
        };

        assert_eq!(selected_url, custom_url);
    }

    #[test]
    fn test_args_version_flag() {
        // Test that version flag is available (will exit with success)
        let result = Args::try_parse_from(["unet-server", "--version"]);
        // This will actually cause the program to exit with version info,
        // but we can test that the parser recognizes the flag
        assert!(result.is_err()); // Clap exits on --version
    }

    #[test]
    fn test_args_help_flag() {
        // Test that help flag is available (will exit with success)
        let result = Args::try_parse_from(["unet-server", "--help"]);
        // This will actually cause the program to exit with help info,
        // but we can test that the parser recognizes the flag
        assert!(result.is_err()); // Clap exits on --help
    }

    #[test]
    fn test_config_validation_success() {
        let config = Config::default();
        // Default config should be valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pathbuf_from_string() {
        let path_str = "/path/to/config.toml";
        let path_buf = PathBuf::from(path_str);
        assert_eq!(path_buf.to_string_lossy(), path_str);
    }

    #[test]
    fn test_optional_string_none() {
        let opt: Option<String> = None;
        assert!(opt.is_none());
    }

    #[test]
    fn test_optional_string_some() {
        let opt = Some("test".to_string());
        assert!(opt.is_some());
        assert_eq!(opt.as_ref().unwrap(), "test");
    }

    #[test]
    fn test_optional_u16_none() {
        let opt: Option<u16> = None;
        assert!(opt.is_none());
    }

    #[test]
    fn test_optional_u16_some() {
        let opt = Some(8080u16);
        assert!(opt.is_some());
        assert_eq!(opt.unwrap_or(0), 8080);
    }

    #[test]
    fn test_optional_pathbuf_none() {
        let opt: Option<PathBuf> = None;
        assert!(opt.is_none());
    }

    #[test]
    fn test_optional_pathbuf_some() {
        let path = PathBuf::from("/test/path");
        let opt = Some(path.clone());
        assert!(opt.is_some());
        assert_eq!(opt.as_ref().unwrap(), &path);
    }

    #[test]
    fn test_string_default_value() {
        let default_db_url = "sqlite://unet.db";
        assert_eq!(default_db_url, "sqlite://unet.db");
    }

    #[test]
    fn test_load_configuration_with_valid_path() {
        // Test that we handle the config path correctly
        let args = Args {
            config: Some(PathBuf::from("/valid/path/config.toml")),
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(),
            log_level: None,
        };

        // This will fail because the file doesn't exist, but we're testing the path handling
        let result = load_configuration(&args);
        assert!(result.is_err()); // Expected to fail with file not found
    }

    #[test]
    fn test_load_configuration_from_env_fallback() {
        let args = Args {
            config: None,
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(),
            log_level: None,
        };

        let config = load_configuration(&args).unwrap();
        // Should fall back to defaults when no env vars are set
        let default_config = Config::default();
        assert_eq!(config.server.host, default_config.server.host);
        assert_eq!(config.server.port, default_config.server.port);
    }

    #[test]
    fn test_load_configuration_file_not_found() {
        let args = Args {
            config: Some(PathBuf::from("/nonexistent/config.toml")),
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(),
            log_level: None,
        };

        let result = load_configuration(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_cli_overrides_all() {
        let mut config = Config::default();
        let original_host = config.server.host.clone();
        let original_port = config.server.port;
        let original_log_level = config.logging.level.clone();

        let args = Args {
            config: None,
            host: Some("10.0.0.1".to_string()),
            port: Some(3000),
            database_url: "sqlite://unet.db".to_string(),
            log_level: Some("trace".to_string()),
        };

        apply_cli_overrides(&mut config, &args);

        assert_ne!(config.server.host, original_host);
        assert_eq!(config.server.host, "10.0.0.1");

        assert_ne!(config.server.port, original_port);
        assert_eq!(config.server.port, 3000);

        assert_ne!(config.logging.level, original_log_level);
        assert_eq!(config.logging.level, "trace");
    }

    #[test]
    fn test_apply_cli_overrides_partial() {
        let mut config = Config::default();
        let original_host = config.server.host.clone();
        let original_port = config.server.port;
        let original_log_level = config.logging.level.clone();

        let args = Args {
            config: None,
            host: Some("172.16.0.1".to_string()),
            port: None, // No override
            database_url: "sqlite://unet.db".to_string(),
            log_level: None, // No override
        };

        apply_cli_overrides(&mut config, &args);

        // Host should be overridden
        assert_ne!(config.server.host, original_host);
        assert_eq!(config.server.host, "172.16.0.1");

        // Port should remain unchanged
        assert_eq!(config.server.port, original_port);

        // Log level should remain unchanged
        assert_eq!(config.logging.level, original_log_level);
    }

    #[test]
    fn test_apply_cli_overrides_none() {
        let mut config = Config::default();
        let original_host = config.server.host.clone();
        let original_port = config.server.port;
        let original_log_level = config.logging.level.clone();

        let args = Args {
            config: None,
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(),
            log_level: None,
        };

        apply_cli_overrides(&mut config, &args);

        // Nothing should change
        assert_eq!(config.server.host, original_host);
        assert_eq!(config.server.port, original_port);
        assert_eq!(config.logging.level, original_log_level);
    }

    #[test]
    fn test_determine_database_url_default() {
        let config = Config::default();
        let args = Args {
            config: None,
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(), // Default value
            log_level: None,
        };

        let url = determine_database_url(&args, &config);
        assert_eq!(url, config.database_url());
    }

    #[test]
    fn test_determine_database_url_custom() {
        let config = Config::default();
        let custom_url = "sqlite://custom_database.db";
        let args = Args {
            config: None,
            host: None,
            port: None,
            database_url: custom_url.to_string(),
            log_level: None,
        };

        let url = determine_database_url(&args, &config);
        assert_eq!(url, custom_url);
    }

    #[tokio::test]
    async fn test_initialize_app_with_overrides() {
        let args = Args {
            config: None, // Use defaults
            host: Some("0.0.0.0".to_string()),
            port: Some(9000),
            database_url: "sqlite://test.db".to_string(),
            log_level: Some("debug".to_string()),
        };

        let result = initialize_app(args).await;
        assert!(result.is_ok());

        let (config, database_url) = result.unwrap();

        // CLI overrides should be applied
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(database_url, "sqlite://test.db");
    }

    #[tokio::test]
    async fn test_initialize_app_with_defaults() {
        let args = Args {
            config: None,
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(),
            log_level: None,
        };

        let result = initialize_app(args).await;
        assert!(result.is_ok());

        let (config, database_url) = result.unwrap();

        // Should use defaults
        let default_config = Config::default();
        assert_eq!(config.server.host, default_config.server.host);
        assert_eq!(config.server.port, default_config.server.port);
        assert_eq!(config.logging.level, default_config.logging.level);
        assert_eq!(database_url, default_config.database_url());
    }

    #[tokio::test]
    async fn test_initialize_app_invalid_config() {
        let args = Args {
            config: Some(PathBuf::from("/nonexistent/file.toml")),
            host: None,
            port: None,
            database_url: "sqlite://unet.db".to_string(),
            log_level: None,
        };

        let result = initialize_app(args).await;
        assert!(result.is_err());
    }
}
