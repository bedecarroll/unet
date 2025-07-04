use std::fs;

#[allow(dead_code)]
#[path = "../src/cli_config.rs"]
mod cli_config;
use cli_config::load_config;

#[test]
fn load_config_from_xdg() {
    let dir = tempfile::tempdir().unwrap();
    let cfg_dir = dir.path().join("unet");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("config.toml"),
        "[client]\nserver_url = 'http://localhost:1234'",
    )
    .unwrap();
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
    }
    let cfg = load_config(None).unwrap();
    assert_eq!(
        cfg.client.unwrap().server_url,
        Some("http://localhost:1234".to_string())
    );
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
fn explicit_path_overrides() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("custom.toml"),
        "[client]\nserver_url = 'http://example.com'",
    )
    .unwrap();
    let cfg = load_config(Some(&dir.path().join("custom.toml"))).unwrap();
    assert_eq!(
        cfg.client.unwrap().server_url,
        Some("http://example.com".to_string())
    );
}
