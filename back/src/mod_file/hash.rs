use std::{
    fs::File,
    io::{self, Read},
};

use bytes::Bytes;
use ferinth::structures::version_structs::Hashes as FerinthHashes;
use serde::{Deserialize, Serialize};
use sha1::Digest;
use thiserror::Error;
use tracing::instrument;

type HashResult<T> = Result<T, HashError>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Hashes {
    pub sha1: String,
    pub sha512: String,
}

impl Hashes {
    #[instrument(skip(file), level = "trace")]
    pub(crate) fn get_hashes_from_file(file: &mut File) -> HashResult<Self> {
        let metadata = file.metadata().map_err(HashError::FileMetadata)?;
        let mut buf = vec![0; metadata.len() as usize];

        std::fs::File::read(file, &mut buf).map_err(HashError::Read)?;

        Ok(get_hashes_from_vec(buf))
    }

    #[allow(dead_code)]
    #[instrument(skip(bytes))]
    pub(crate) fn get_hashes_from_bytes(bytes: &Bytes) -> Self { get_hashes_from_vec(bytes) }

    pub(crate) fn ferinth(fer_hashes: FerinthHashes) -> Option<Self> {
        if fer_hashes.sha1.is_none() || fer_hashes.sha512.is_none() {
            None
        } else {
            Some(Self {
                sha1: fer_hashes.sha1.unwrap(),
                sha512: fer_hashes.sha512.unwrap(),
            })
        }
    }
}

fn get_hashes_from_vec(vec: impl AsRef<[u8]>) -> Hashes {
    let sha1 = hex::encode(sha1::Sha1::digest(&vec));
    let sha512 = hex::encode(sha2::Sha512::digest(&vec));
    Hashes { sha1, sha512 }
}

#[derive(Debug, Error)]
pub enum HashError {
    #[error("Could not get file metadata: {0}")]
    FileMetadata(#[source] io::Error),

    #[error("Error while reading file: {0}")]
    Read(#[source] io::Error),
}
