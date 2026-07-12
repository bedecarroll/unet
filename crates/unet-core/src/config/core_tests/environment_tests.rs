//! Tests for environment variable handling

use super::super::{core::Config, env::collect_env_vars};
use std::collections::HashMap;
use std::env;

#[test]
fn test_config_from_env_empty() {
    let env_source = |_key: &str| Err(env::VarError::NotPresent);
    let result = Config::from_env_with_source(env_source);

    // Empty env should fail since no defaults are set in from_env_with_source
    // The function only applies overrides but doesn't start with defaults
    assert!(result.is_err());
}

#[test]
fn test_config_from_env_incomplete_fails() {
    let mut env_vars = HashMap::new();
    // Only provide some fields - should fail because from_env_with_source
    // doesn't provide defaults and requires all fields to be present
    env_vars.insert("UNET_DATABASE__URL", "postgres://test:test@localhost/test");
    env_vars.insert("UNET_LOGGING__LEVEL", "debug");
    env_vars.insert("UNET_SERVER__PORT", "9090");

    let env_source = |key: &str| {
        env_vars
            .get(key)
            .map(|v| (*v).to_string())
            .ok_or(env::VarError::NotPresent)
    };

    let result = Config::from_env_with_source(env_source);

    // Should fail because not all required fields are provided
    assert!(result.is_err());
    let error = result.unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains("missing configuration field") || message.contains("missing field"),
        "expected missing-field error, got: {message}"
    );
}

#[test]
fn test_config_from_env_invalid_values() {
    let mut env_vars = HashMap::new();
    env_vars.insert("UNET_SERVER__PORT", "invalid_port");

    let env_source = |key: &str| {
        env_vars
            .get(key)
            .map(|v| (*v).to_string())
            .ok_or(env::VarError::NotPresent)
    };

    let result = Config::from_env_with_source(env_source);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Failed to deserialize config from environment")
    );
}

#[test]
fn test_collect_env_vars_empty() {
    let env_source = |_key: &str| Err(env::VarError::NotPresent);
    let vars = collect_env_vars(&env_source);
    assert!(vars.is_empty());
}

#[test]
fn test_collect_env_vars_with_values() {
    let mut env_vars = HashMap::new();
    env_vars.insert("UNET_DATABASE__URL", "postgres://test");
    env_vars.insert("UNET_LOGGING__LEVEL", "debug");
    env_vars.insert("UNET_SERVER__PORT", "8080");

    let env_source = |key: &str| {
        env_vars
            .get(key)
            .map(|v| (*v).to_string())
            .ok_or(env::VarError::NotPresent)
    };

    let vars = collect_env_vars(&env_source);
    assert_eq!(vars.len(), 3);

    let vars_map: HashMap<String, String> = vars.into_iter().collect();
    assert_eq!(
        vars_map.get("database.url"),
        Some(&"postgres://test".to_string())
    );
    assert_eq!(vars_map.get("logging.level"), Some(&"debug".to_string()));
    assert_eq!(vars_map.get("server.port"), Some(&"8080".to_string()));
}

#[test]
fn test_collect_env_vars_partial_values() {
    let mut env_vars = HashMap::new();
    env_vars.insert("UNET_DATABASE__URL", "postgres://test");
    // Missing UNET_LOGGING__LEVEL and other vars

    let env_source = |key: &str| {
        env_vars
            .get(key)
            .map(|v| (*v).to_string())
            .ok_or(env::VarError::NotPresent)
    };

    let vars = collect_env_vars(&env_source);
    assert_eq!(vars.len(), 1);

    let vars_map: HashMap<String, String> = vars.into_iter().collect();
    assert_eq!(
        vars_map.get("database.url"),
        Some(&"postgres://test".to_string())
    );
}

#[test]
fn test_collect_env_vars_includes_auth_token() {
    let mut env_vars = HashMap::new();
    env_vars.insert("UNET_AUTH__TOKEN", "bed-24-secret");

    let env_source = |key: &str| {
        env_vars
            .get(key)
            .map(|v| (*v).to_string())
            .ok_or(env::VarError::NotPresent)
    };

    let vars = collect_env_vars(&env_source);
    let vars_map: HashMap<String, String> = vars.into_iter().collect();
    assert_eq!(
        vars_map.get("auth.token"),
        Some(&"bed-24-secret".to_string())
    );
}

#[test]
fn test_config_from_env_with_cors_lists() {
    let mut env_vars = HashMap::new();
    env_vars.insert("UNET_DATABASE__URL", "sqlite://test.db");
    env_vars.insert("UNET_DATABASE__MAX_CONNECTIONS", "10");
    env_vars.insert("UNET_DATABASE__TIMEOUT", "30");
    env_vars.insert("UNET_LOGGING__LEVEL", "info");
    env_vars.insert("UNET_LOGGING__FORMAT", "text");
    env_vars.insert("UNET_SNMP__COMMUNITY", "public");
    env_vars.insert("UNET_SNMP__TIMEOUT", "5");
    env_vars.insert("UNET_SNMP__RETRIES", "3");
    env_vars.insert("UNET_SERVER__HOST", "127.0.0.1");
    env_vars.insert("UNET_SERVER__PORT", "8080");
    env_vars.insert("UNET_SERVER__MAX_REQUEST_SIZE", "1048576");
    env_vars.insert(
        "UNET_SERVER__CORS_ORIGINS",
        "https://dashboard.corp.local,http://localhost:3000",
    );
    env_vars.insert("UNET_SERVER__CORS_METHODS", "GET,POST");
    env_vars.insert("UNET_SERVER__CORS_HEADERS", "authorization,content-type");
    env_vars.insert("UNET_GIT__BRANCH", "main");
    env_vars.insert("UNET_GIT__SYNC_INTERVAL", "300");
    env_vars.insert("UNET_DOMAIN__SEARCH_DOMAINS", "corp.local,lab.local");
    env_vars.insert("UNET_AUTH__ENABLED", "false");

    let env_source = |key: &str| {
        env_vars
            .get(key)
            .map(|v| (*v).to_string())
            .ok_or(env::VarError::NotPresent)
    };

    let config = Config::from_env_with_source(env_source).expect("config should load");

    assert_eq!(
        config.server.cors_origins,
        vec![
            "https://dashboard.corp.local".to_string(),
            "http://localhost:3000".to_string()
        ]
    );
    assert_eq!(
        config.server.cors_methods,
        vec!["GET".to_string(), "POST".to_string()]
    );
    assert_eq!(
        config.server.cors_headers,
        vec!["authorization".to_string(), "content-type".to_string()]
    );
    assert_eq!(
        config.domain.search_domains,
        vec!["corp.local".to_string(), "lab.local".to_string()]
    );
}

#[test]
fn test_config_from_env_partial_override() {
    let mut env_vars = HashMap::new();
    env_vars.insert("UNET_DATABASE__URL", "sqlite://test.db");

    let env_source = |key: &str| {
        env_vars
            .get(key)
            .map(|v| (*v).to_string())
            .ok_or(env::VarError::NotPresent)
    };

    // Partial override should fail since from_env_with_source doesn't provide defaults
    let result = Config::from_env_with_source(env_source);
    assert!(result.is_err());
}
