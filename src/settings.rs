use std::ffi::OsString;

use anyhow::{Context, Ok, Result};
use config::{Config, File, FileFormat};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CFTemplate {
    pub alias: String,
    pub lang: String,
    pub path: String,
    pub before_script: String,
    pub script: String,
    pub after_script: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CFSettings {
    pub username: Option<String>,
    pub key: Option<String>,
    pub secret: Option<String>,
    pub template: Option<Vec<CFTemplate>>,
    pub home_dir: Option<String>,
}

lazy_static! {
    pub static ref SETTINGS: CFSettings = load_settings().unwrap();
}

pub fn load_settings() -> Result<CFSettings> {
    let config_file_path = get_config_file_path()?;
    let config = get_config(config_file_path)?;
    let settings = deserialize_config_into_settings(config)?;
    Ok(settings)
}

fn get_config_file_path() -> Result<OsString> {
    let config_dir = dirs::config_dir().context("Configuration directory not defined")?;
    let config_file_path = config_dir.join("cf").join("cf.json").into_os_string();
    Ok(config_file_path)
}

fn get_config(config_file_path: OsString) -> Result<Config> {
    let config = Config::builder()
        .add_source(File::new(
            config_file_path.to_str().context(format!(
                "Configuration directory contains non-unicode characters: {:?}",
                config_file_path
            ))?,
            FileFormat::Json,
        ))
        .build()
        .context("Failed to build config")?;
    Ok(config)
}

fn deserialize_config_into_settings(config: Config) -> Result<CFSettings> {
    let settings: CFSettings = config
        .try_deserialize()
        .context("Failed to deserialize configuration file")?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_normal() {
        let config = Config::builder().build().unwrap();
        let _settings = deserialize_config_into_settings(config).unwrap();
    }

    #[test]
    #[should_panic]
    fn config_missing() {
        let config = Config::builder().build().unwrap();
        let _settings = deserialize_config_into_settings(config).unwrap();
    }
}
