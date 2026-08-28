use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub subscription_url: String,
    pub hwid: String,
}

pub fn config_dir() -> Result<PathBuf, String> {
    let base =
        dirs::config_dir().ok_or_else(|| "Could not determine config directory".to_string())?;

    let dir = base.join("xrayctl");

    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create config directory: {error}"))?;

    Ok(dir)
}

pub fn config_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn load() -> Result<Config, String> {
    let file = config_file()?;

    let content = std::fs::read_to_string(&file)
        .map_err(|error| format!("Failed to read config: {error}"))?;

    toml::from_str(&content).map_err(|error| format!("Failed to parse config: {error}"))
}

pub fn subscription_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("subscription.json"))
}

pub fn active_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("active.json"))
}

pub fn xray_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("xray.json"))
}

pub fn pid_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("xray.pid"))
}

pub fn log_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("xray.log"))
}
