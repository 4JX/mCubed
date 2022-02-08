// Using https://github.com/theRookieCoder/ferium/blob/main/src/util/ferium_error.rs as a base

use serde_json::error as JsonError;
use toml::de::Error as TomlError;
use std::io::Error as IoError;
use zip::result::ZipError;
use thiserror::Error;

pub type AppResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("IoError: {}", .0)]
    IoError(#[from] IoError),
    #[error("SerdeError: {}",match .0 {
        JsonError::Category::Syntax => {
            "The file being parsed contains syntax errors"
        },
        JsonError::Category::Io => {
            "Encountered an I/O error while handling JSON"
        },
        JsonError::Category::Data => {
            "Data error"
        },
        JsonError::Category::Eof => {
            "Found an unexpected end of file"
        }, 
    }
    )]
    SerdeError(JsonError::Category),
    #[error("TomlError: {}", .0)]
    TomlError(#[from] TomlError),
    #[error("ZipError: {}", .0)]
    ZipError(#[from] ZipError),
    #[error("The archive provided is not a valid mod file")]
    InvalidModFile,
    #[error("The archive does not correspond to this modloader")]
    IncorrectModloader

}

impl From<JsonError::Error> for Error {
	fn from(err: serde_json::error::Error) -> Self {
		Self::SerdeError(err.classify())
	}
}