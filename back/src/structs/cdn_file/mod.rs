use std::{collections::HashSet, path::PathBuf};

use ferinth::structures::version_structs::VersionFile;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use self::downloader::FileDownloader;
use crate::{mod_file::Hashes, settings::CONF};

mod downloader;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CdnFile {
    /// The url where the file is hosted
    pub url: String,
    /// The file's name
    pub filename: String,
    /// The file's size
    pub size: u64,
    /// The file's hashes
    pub hashes: Option<Hashes>,
}

impl From<VersionFile> for CdnFile {
    fn from(ver_file: VersionFile) -> Self {
        let VersionFile {
            hashes,
            url,
            filename,
            primary: _,
            size,
        } = ver_file;

        Self {
            url,
            filename,
            size: size as u64,
            hashes: Hashes::ferinth(hashes),
        }
    }
}

impl std::fmt::Display for CdnFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Debug::fmt(self, f) }
}

impl CdnFile {
    pub async fn download(self, folder: PathBuf) -> HashSet<std::sync::Arc<CdnFile>> {
        let url = self.url.clone();

        let downloader = FileDownloader::default();
        let (fetcher, mut events) = downloader.download(folder, self);

        tokio::spawn(fetcher);

        let mut failed = HashSet::new();

        while let Some(event) = events.recv().await {
            match event.kind {
                downloader::EventKind::Fetching => {
                    info!("Downloading file: {}", url)
                }
                downloader::EventKind::Fetched => {}
                downloader::EventKind::Error(err) => {
                    error!("Error while downloading {}: {}", event.package.filename, err);
                    failed.insert(event.package.clone());
                }
                downloader::EventKind::Validated => {}
                downloader::EventKind::Retrying => {
                    info!("Retrying download for {}", event.package.filename);
                }
            }
        }

        failed
    }

    pub async fn download_to_mod_folder(self) -> HashSet<std::sync::Arc<CdnFile>> {
        let mod_folder_path = CONF.lock().mod_folder_path.clone();
        self.download(mod_folder_path).await
    }
}
