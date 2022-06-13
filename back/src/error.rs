use mc_mod_meta::error::Error as MetaError;
use thiserror::Error;

use crate::mod_file::hash::HashError;

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

    #[error("Failed to parse cache file:  {}", err)]
    FailedToParseEntryCache { err: serde_json::Error },

    #[error("Could not get file hash:  {0}")]
    ParseHash(#[from] HashError),

    // Shared errors
    #[error("Encountered an I/O error while handling the file: {}", .0)]
    IoError(#[from] std::io::Error),

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
    #[error("The ratelimit was exceeded, please wait {} seconds before trying again", .0)]
    FerinthRatelimitExceeded(usize),

    // Daedalus errors
    #[error("Failed to validate file checksum at url {url} with hash {hash} after {tries} tries")]
    DaedalusChecksumFailure { hash: String, url: String, tries: u32 },
    #[error("Error while managing asynchronous tasks")]
    DaedalusTaskError(#[from] tokio::task::JoinError),
    #[error("{0}")]
    DaedalusParseError(String),

    // Send to trash errors
    #[error("Could not delete file")]
    TrashFailedDelete(#[from] trash::Error),

    // Zip parsing errors
    #[error("Could parse the zip file")]
    ZipError(#[from] zip::result::ZipError),

    #[error("There was an error when working with an image")]
    ImageError(#[from] image::ImageError),
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
            ferinth::Error::ReqwestError(inner) => {
                let item = inner
                    .url()
                    .map_or("Unknown (Ferinth)".to_string(), |url| url.as_str().to_string());

                Self::ReqwestError { inner, item }
            }
            ferinth::Error::RateLimitExceeded(seconds) => Self::FerinthRatelimitExceeded(seconds),
            ferinth::Error::JSONError(err) => Self::SerdeError(err),
        }
    }
}

impl From<daedalus::Error> for Error {
    fn from(err: daedalus::Error) -> Self {
        match err {
            daedalus::Error::ChecksumFailure { hash, url, tries } => Self::DaedalusChecksumFailure { hash, url, tries },
            daedalus::Error::SerdeError(error) => Self::SerdeError(error),
            daedalus::Error::FetchError { inner, item } => Self::ReqwestError { inner, item },
            daedalus::Error::TaskError(err) => Self::DaedalusTaskError(err),
            daedalus::Error::ParseError(string) => Self::DaedalusParseError(string),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let item = err.url().map_or("Unknown".to_string(), |url| url.as_str().to_string());
        Self::ReqwestError { inner: err, item }
    }
}
