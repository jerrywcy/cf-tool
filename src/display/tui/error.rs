use thiserror::Error;

#[derive(Debug, Error)]
#[error("{item} not configured.\nPlease configure {item} in your configuration file or use `cf-tui config.`")]
pub struct NoConfigItemError {
    pub item: String,
}
