use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
#[error(
    "Authorization failed due to {missing} missing.\
     Please configure {missing} in configuration file."
)]
pub struct NoAuthorizationError {
    pub missing: String,
}

impl NoAuthorizationError {
    pub fn new(missing: impl ToString) -> Self {
        Self {
            missing: missing.to_string(),
        }
    }
}
