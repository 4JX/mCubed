// Using https://github.com/theRookieCoder/ferium/blob/main/src/util/ferium_error.rs as a base

use std::io::Error as IoError;

use serde_json::error::Error as JsonError;
use thiserror::Error;
use toml::de::Error as TomlError;
use zip::result::ZipError;

pub type LibResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("IoError: {}", .0)]
    IoError(#[from] IoError),
    #[error("Error while deserializing JSON")]
    SerdeError(#[from] JsonError),
    #[error("TomlError: {}", .0)]
    TomlError(#[from] TomlError),
    #[error("ZipError: {}", .0)]
    ZipError(#[from] ZipError),
    #[error("The archive provided is not a valid mod file")]
    InvalidModFile,
    #[error("The archive does not correspond to this modloader")]
    IncorrectModloader,
}
