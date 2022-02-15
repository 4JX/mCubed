use mc_mod_meta::error::Error as MetaError;
use std::io::Error as IoError;
use thiserror::Error;

pub type LibResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    // Library errors
    #[error("The entry does not contain any Modrinth data")]
    NoModrinthDataError,

    #[error("There is no valid version for this entry")]
    InvalidLatestVersionError,

    #[error("No mods were found meeting the specified requirements")]
    ModrinthEmptyVersionSearchResult,

    #[error("No mods were found meeting the specified requirements")]
    ModrinthEmptyFileList,

    #[error("The mod already exists on the folder")]
    EntryAlreadyInList,

    #[error("The id or slug provided is not valid")]
    NotValidModrinthId,

    // Shared errors
    #[error("Encountered an I/O error while handling the file: {}", .0)]
    IoError(#[from] IoError),

    #[error("Failed to parse JSON: {}", .0)]
    SerdeError(#[from] serde_json::error::Error),

    #[error("Unable to fetch {item}")]
    ReqwestError { inner: reqwest::Error, item: String },

    // Manifest metadata errors
    #[error("There was an error parsing the Forge manifest")]
    MetadataTomlError,
    #[error("Failed to parse the ZIP file")]
    MetadataZipError,
    #[error("The archive provided is not a valid mod file")]
    MetadataInvalidModFile,
    #[error("The archive does not correspond to this modloader")]
    MetadataIncorrectModloader,

    // Ferinth errors
    #[error("Invalid slug or ID provided")]
    FerinthBase62Error,
    #[error("Invalid SHA1 hash")]
    FerinthNotSHA1Error,
    #[error("Could not parse url")]
    FerinthURLParseError,

    // Daedalus errors
    #[error("Failed to validate file checksum at url {url} with hash {hash} after {tries} tries")]
    DaedalusChecksumFailure {
        hash: String,

        url: String,

        tries: u32,
    },
    #[error("Error while managing asynchronous tasks")]
    DaedalusTaskError(#[from] tokio::task::JoinError),
    #[error("{0}")]
    DaedalusParseError(String),
}

impl From<MetaError> for Error {
    fn from(error: MetaError) -> Self {
        match error {
            MetaError::IoError(err) => Self::IoError(err),
            MetaError::SerdeError(error) => Self::SerdeError(error),
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
            ferinth::Error::ReqwestError(inner) => Self::ReqwestError {
                inner,
                item: "Unknown (Ferinth)".to_string(),
            },
            ferinth::Error::URLParseError(_) => Self::FerinthURLParseError,
        }
    }
}

impl From<daedalus::Error> for Error {
    fn from(err: daedalus::Error) -> Self {
        match err {
            daedalus::Error::ChecksumFailure { hash, url, tries } => {
                Self::DaedalusChecksumFailure { hash, url, tries }
            }
            daedalus::Error::SerdeError(error) => Self::SerdeError(error),
            daedalus::Error::FetchError { inner, item } => Self::ReqwestError { inner, item },
            daedalus::Error::TaskError(err) => Self::DaedalusTaskError(err),
            daedalus::Error::ParseError(string) => Self::DaedalusParseError(string),
        }
    }
}
