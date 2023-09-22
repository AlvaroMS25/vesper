use thiserror::Error;
use twilight_validate::command::CommandValidationError;
use twilight_http::{Error as HttpError, response::DeserializeBodyError};

#[non_exhaustive]
#[derive(Debug, Error)]
#[error(transparent)]
pub enum CreateCommandError {
    Validation(#[from] CommandValidationError),
    Http(#[from] HttpError),
    Deserialize(#[from] DeserializeBodyError)
}
