use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct CliConfig {
    pub client: Option<ClientSection>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ClientSection {
    pub server_url: Option<String>,
    pub timeout_secs: Option<u64>,
    pub output_format: Option<String>,
    pub auth_token_file: Option<String>,
}

pub fn load_config(explicit: Option<&Path>) -> Option<CliConfig> {
    let mut paths = Vec::new();

    if let Some(p) = explicit {
        paths.push(p.to_path_buf());
    }

    let cwd = std::env::current_dir().ok();
    if let Some(cwd) = cwd {
        paths.push(cwd.join("unet-cli.toml"));
        paths.push(cwd.join("config/unet-cli.toml"));
    }

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        paths.push(PathBuf::from(xdg).join("unet/config.toml"));
    } else if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".config/unet/config.toml"));
    }

    paths.push(PathBuf::from("/etc/unet/cli.toml"));

    for path in paths {
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<CliConfig>(&content) {
                    Ok(cfg) => return Some(cfg),
                    Err(_) => continue,
                },
                Err(_) => continue,
            }
        }
    }

    None
}
