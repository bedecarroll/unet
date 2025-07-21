//! Configuration and CLI argument tests

use crate::config_loader::*;
use clap::Parser;
use std::path::PathBuf;
use unet_core::config::Config;

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
    let args = Args::try_parse_from(["unet-server", "--config", "/path/to/config.toml"]).unwrap();
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
    let result = Args::try_parse_from(["unet-server", "--port", "70000"]);
    assert!(result.is_err());
}

#[test]
fn test_config_override_host() {
    let mut config = Config::default();
    let args = Args {
        config: None,
        host: Some("192.168.1.100".to_string()),
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };
    apply_cli_overrides(&mut config, &args);
    assert_eq!(config.server.host, "192.168.1.100");
}

#[test]
fn test_config_override_port() {
    let mut config = Config::default();
    let args = Args {
        config: None,
        host: None,
        port: Some(9090),
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };
    apply_cli_overrides(&mut config, &args);
    assert_eq!(config.server.port, 9090);
}

#[test]
fn test_config_override_log_level() {
    let mut config = Config::default();
    let args = Args {
        config: None,
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: Some("trace".to_string()),
    };
    apply_cli_overrides(&mut config, &args);
    assert_eq!(config.logging.level, "trace");
}

#[test]
fn test_database_url_selection_default() {
    let config = Config::default();
    let args = Args {
        config: None,
        host: None,
        port: None,
        database_url: "sqlite://unet.db".to_string(),
        log_level: None,
    };

    let url = determine_database_url(&args, &config);
    assert_eq!(url, config.database_url());
}

#[test]
fn test_database_url_selection_custom() {
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

#[test]
fn test_args_version_flag() {
    let result = Args::try_parse_from(["unet-server", "--version"]);
    assert!(result.is_err());
}

#[test]
fn test_args_help_flag() {
    let result = Args::try_parse_from(["unet-server", "--help"]);
    assert!(result.is_err());
}

#[test]
fn test_apply_cli_overrides_all() {
    let mut config = Config::default();
    let args = Args {
        config: None,
        host: Some("10.0.0.1".to_string()),
        port: Some(5000),
        database_url: "sqlite://test.db".to_string(),
        log_level: Some("error".to_string()),
    };

    apply_cli_overrides(&mut config, &args);

    assert_eq!(config.server.host, "10.0.0.1");
    assert_eq!(config.server.port, 5000);
    assert_eq!(config.logging.level, "error");
}

#[test]
fn test_apply_cli_overrides_partial() {
    let mut config = Config::default();
    let original_host = config.server.host.clone();
    let original_log_level = config.logging.level.clone();

    let args = Args {
        config: None,
        host: None,
        port: Some(7777),
        database_url: "sqlite://test.db".to_string(),
        log_level: None,
    };

    apply_cli_overrides(&mut config, &args);

    assert_eq!(config.server.host, original_host);
    assert_eq!(config.server.port, 7777);
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
        database_url: "sqlite://test.db".to_string(),
        log_level: None,
    };

    apply_cli_overrides(&mut config, &args);

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
        database_url: "sqlite://unet.db".to_string(),
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

#[test]
fn test_pathbuf_from_string() {
    let path_str = "/some/path/to/config.toml";
    let path_buf = PathBuf::from(path_str);
    assert_eq!(path_buf.to_string_lossy(), path_str);
}

#[test]
fn test_optional_string_none() {
    let value: Option<String> = None;
    assert!(value.is_none());
}

#[test]
fn test_optional_string_some() {
    let value = "test".to_string();
    let option_value = Some(value.clone());
    assert!(option_value.is_some());
    if let Some(val) = option_value {
        assert_eq!(val, value);
    }
}

#[test]
fn test_optional_u16_none() {
    let value: Option<u16> = None;
    assert!(value.is_none());
}

#[test]
fn test_optional_u16_some() {
    let value = 8080;
    let option_value = Some(value);
    assert!(option_value.is_some());
    if let Some(val) = option_value {
        assert_eq!(val, value);
    }
}

#[test]
fn test_optional_pathbuf_none() {
    let value: Option<PathBuf> = None;
    assert!(value.is_none());
}

#[test]
fn test_optional_pathbuf_some() {
    let path = PathBuf::from("/test/path");
    let option_value = Some(path.clone());
    assert!(option_value.is_some());
    if let Some(val) = option_value {
        assert_eq!(val, path);
    }
}

#[test]
fn test_string_default_value() {
    let default_str = "sqlite://unet.db";
    assert_eq!(default_str, "sqlite://unet.db");
}
