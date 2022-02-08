use mc_mod_meta::error::Error as MetaError;
use std::io::Error as IoError;
use thiserror::Error;

pub type LibResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("Encountered an I/O error while handling the file: {}", .0)]
    IoError(#[from] IoError),
  
    #[error("SerdeError: {}",match .0 {
        serde_json::error::Category::Syntax => {
            "The file being parsed contains syntax errors"
        },
        serde_json::error::Category::Io => {
            "Encountered an I/O error while handling JSON"
        },
        serde_json::error::Category::Data => {
            "Data error"
        },
        serde_json::error::Category::Eof => {
            "Found an unexpected end of file"
        }, 
    })]
    SerdeError( serde_json::error::Category),

    #[error("There was an error parsing the Forge manifest")]
    MetadataTomlError,
    #[error("Failed to parse the ZIP file")]
    MetadataZipError,
    #[error("The archive provided is not a valid mod file")]
    MetadataInvalidModFile,
    #[error("The archive does not correspond to this modloader")]
    MetadataIncorrectModloader,
    #[error("Invalid slug or ID provided")]
    FerinthBase62Error,
    #[error("Invalid SHA1 hash")]
    FerinthNotSHA1Error,
    #[error("Failed to send/process an HTTP(S) request")]
    FerinthReqwestError,
    #[error("Could not parse url")]
    FerinthURLParseError,
    #[error("The entry does not contain any Modrinth data")]
    NoModrinthDataError,
    #[error("There is no valid version for this entry")]
    InvalidLatestVersionError,
    #[error("Failed to validate file checksum at url {url} with hash {hash} after {tries} tries")]
    ChecksumFailure {
        hash: String,

        url: String,

        tries: u32,
    },

    #[error("Unable to fetch {item}")]
    FetchError { inner: reqwest::Error, item: String },

    #[error("Error while managing asynchronous tasks")]
    TaskError(#[from] tokio::task::JoinError),

    #[error("{0}")]
    ParseError(String),
}

impl From<MetaError> for Error {
    fn from(error: MetaError) -> Self {
        match error {
            MetaError::IoError(err) => Self::IoError(err),
            MetaError::SerdeError(category) => Self::SerdeError(category),
            MetaError::TomlError(_) => Self::MetadataTomlError,
            MetaError::ZipError(_) => Self::MetadataZipError,
            MetaError::InvalidModFile => Self::MetadataInvalidModFile,
            MetaError::IncorrectModloader => Self::MetadataIncorrectModloader,
        }
    }
}

impl From<ferinth::Error> for Error {
    fn from(err: ferinth::Error) -> Self {
        match err {
            ferinth::Error::NotBase62 => Self::FerinthBase62Error,
            ferinth::Error::NotSHA1 => Self::FerinthNotSHA1Error,
            ferinth::Error::ReqwestError(_) => Self::FerinthReqwestError,
            ferinth::Error::URLParseError(_) => Self::FerinthURLParseError,
        }
    }
}

impl From<daedalus::Error> for Error {
    fn from(err: daedalus::Error) -> Self {
        match err {
            daedalus::Error::ChecksumFailure { hash, url, tries } => {
                Self::ChecksumFailure { hash, url, tries }
            }
            daedalus::Error::SerdeError(err) => Self::SerdeError(err.classify()),
            daedalus::Error::FetchError { inner, item } => Self::FetchError { inner, item },
            daedalus::Error::TaskError(err) => Self::TaskError(err),
            daedalus::Error::ParseError(string) => Self::ParseError(string),
        }
    }
}
