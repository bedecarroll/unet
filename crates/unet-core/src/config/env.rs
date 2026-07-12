//! Environment variable handling for configuration overrides.

use crate::error::{Error, Result};
use config::builder::{ConfigBuilder, DefaultState};

const SCALAR_ENV_VARS: [(&str, &str); 20] = [
    ("UNET_DATABASE__URL", "database.url"),
    ("UNET_DATABASE__MAX_CONNECTIONS", "database.max_connections"),
    ("UNET_DATABASE__TIMEOUT", "database.timeout"),
    ("UNET_LOGGING__LEVEL", "logging.level"),
    ("UNET_LOGGING__FORMAT", "logging.format"),
    ("UNET_LOGGING__FILE", "logging.file"),
    ("UNET_SNMP__COMMUNITY", "snmp.community"),
    ("UNET_SNMP__TIMEOUT", "snmp.timeout"),
    ("UNET_SNMP__RETRIES", "snmp.retries"),
    ("UNET_SERVER__HOST", "server.host"),
    ("UNET_SERVER__PORT", "server.port"),
    ("UNET_SERVER__MAX_REQUEST_SIZE", "server.max_request_size"),
    ("UNET_GIT__REPOSITORY_URL", "git.repository_url"),
    ("UNET_GIT__LOCAL_DIRECTORY", "git.local_directory"),
    ("UNET_GIT__BRANCH", "git.branch"),
    ("UNET_GIT__AUTH_TOKEN", "git.auth_token"),
    ("UNET_GIT__SYNC_INTERVAL", "git.sync_interval"),
    ("UNET_DOMAIN__DEFAULT_DOMAIN", "domain.default_domain"),
    ("UNET_AUTH__ENABLED", "auth.enabled"),
    ("UNET_AUTH__TOKEN", "auth.token"),
];

const LIST_ENV_VARS: [(&str, &str); 4] = [
    ("UNET_DOMAIN__SEARCH_DOMAINS", "domain.search_domains"),
    ("UNET_SERVER__CORS_ORIGINS", "server.cors_origins"),
    ("UNET_SERVER__CORS_METHODS", "server.cors_methods"),
    ("UNET_SERVER__CORS_HEADERS", "server.cors_headers"),
];

pub fn apply_env_overrides<F>(
    mut builder: ConfigBuilder<DefaultState>,
    env_source: &F,
) -> Result<ConfigBuilder<DefaultState>>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    for (key, value) in collect_env_vars(env_source) {
        builder = builder
            .set_override(&key, value)
            .map_err(|e| Error::config(format!("Failed to set config override for {key}: {e}")))?;
    }

    for (env_key, config_key) in LIST_ENV_VARS {
        if let Ok(value) = env_source(env_key) {
            builder = builder
                .set_override(config_key, split_csv(&value))
                .map_err(|e| {
                    Error::config(format!(
                        "Failed to set config override for {config_key}: {e}"
                    ))
                })?;
        }
    }

    Ok(builder)
}

pub fn collect_env_vars<F>(env_source: &F) -> Vec<(String, String)>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    SCALAR_ENV_VARS
        .iter()
        .filter_map(|(env_key, config_key)| {
            env_source(env_key)
                .ok()
                .map(|value| ((*config_key).to_string(), value))
        })
        .collect()
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(ToString::to_string)
        .collect()
}
