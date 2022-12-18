use std::{
    collections::HashMap,
    fs::{DirBuilder, OpenOptions},
    io::ErrorKind,
    path::PathBuf,
};

use color_eyre::{
    eyre::{bail, eyre, Context},
    Result,
};
use config::{Config, File, FileFormat};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CFTemplate {
    pub alias: String,
    pub lang: String,
    pub path: PathBuf,
    pub ext: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Scripts {
    pub before_script: Option<String>,
    pub script: String,
    pub after_script: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CFSettings {
    pub username: Option<String>,
    pub key: Option<String>,
    pub secret: Option<String>,
    pub templates: Option<Vec<CFTemplate>>,
    pub commands: Option<HashMap<String, Scripts>>,
    pub home_dir: Option<PathBuf>,
}

lazy_static! {
    pub static ref SETTINGS: CFSettings = load_settings().unwrap();
}

pub fn load_settings() -> Result<CFSettings> {
    let config_file_path = get_config_file_path()?;
    let file = OpenOptions::new().read(true).open(&config_file_path);
    match file {
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => bail!("No configuration file found at {}.\nPlease add a configuration file or run `cf-tui config`.", config_file_path.display()),
                _ => return Err(err).wrap_err(format!("Failed when reading configuration from {}", config_file_path.display())),
            }
        }
        _ => drop(file),
    }
    let config = get_config(config_file_path)?;
    let settings = deserialize_config_into_settings(config)?;
    Ok(settings)
}

pub fn get_config_file_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or(eyre!("Configuration directory not defined"))?;
    DirBuilder::new().recursive(true).create(&config_dir)?;
    let config_file_path = config_dir.join("cf").join("cf.json");
    Ok(config_file_path)
}

pub fn get_config(config_file_path: PathBuf) -> Result<Config> {
    let config = Config::builder()
        .add_source(File::new(
            config_file_path.to_str().ok_or(eyre!(
                "Configuration directory contains non-unicode characters: {:?}",
                config_file_path
            ))?,
            FileFormat::Json,
        ))
        .build()
        .wrap_err("Failed to build config")?;
    Ok(config)
}

pub fn deserialize_config_into_settings(config: Config) -> Result<CFSettings> {
    let settings: CFSettings = config
        .try_deserialize()
        .wrap_err("Failed to deserialize configuration file")?;
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
