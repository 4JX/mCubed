// Heavily copied from https://crates.io/crates/apt-cmd (0.3.0) by Michael Murphy under MPL-2.0
// Thanks for the abstraction layer

use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_fetcher::{Fetcher, Source};
use futures::{stream, StreamExt};
use thiserror::Error;
use tokio::sync::mpsc;

use super::CdnFile;
use crate::mod_file::{hash::HashError, Hashes};

#[derive(Default)]
pub struct FileDownloader {
    pub fetcher: Fetcher<CdnFile>,
}

impl FileDownloader {
    pub fn download(
        self,
        folder: PathBuf,
        file: CdnFile,
    ) -> (
        impl std::future::Future<Output = ()> + Send + 'static,
        mpsc::UnboundedReceiver<FetchEvent>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel::<FetchEvent>();
        let (events_tx, mut events_rx) = mpsc::unbounded_channel();

        let path = folder.join(&file.filename);
        let mut source = Source::new(
            Arc::from(vec![Box::from(&*file.url)].into_boxed_slice()),
            Arc::from(path.clone()),
        );

        source.set_part(Some(Arc::from(path.with_extension("part"))));

        let mut results = self
            .fetcher
            .events(events_tx)
            .build()
            .stream_from(stream::iter(vec![(source, Arc::new(file))]), 1);

        let event_handler = {
            let tx = tx.clone();
            async move {
                while let Some((dest, package, event)) = events_rx.recv().await {
                    match event {
                        async_fetcher::FetchEvent::Fetching => {
                            let _ = tx.send(FetchEvent::new(package, EventKind::Fetching));
                        }

                        async_fetcher::FetchEvent::Fetched => {
                            let _ = tx.send(FetchEvent::new(package.clone(), EventKind::Fetched));
                            let tx = tx.clone();

                            if let Some(hashes) = &package.hashes {
                                let event = match compare_hash(&dest, package.size, hashes) {
                                    Ok(()) => EventKind::Validated,
                                    Err(source) => {
                                        let _ = std::fs::remove_file(&dest);
                                        EventKind::Error(FetchError::Checksum {
                                            package: package.url.clone(),
                                            source,
                                        })
                                    }
                                };

                                let _ = tx.send(FetchEvent::new(package, event));
                            }
                        }

                        async_fetcher::FetchEvent::Retrying => {
                            let _ = tx.send(FetchEvent::new(package, EventKind::Retrying));
                        }

                        _ => (),
                    }
                }
            }
        };

        let fetcher = async move {
            while let Some((path, file, result)) = results.next().await {
                if let Err(source) = result {
                    let _ = tx.send(FetchEvent::new(
                        file.clone(),
                        EventKind::Error(FetchError::Fetch {
                            package: file.url.clone(),
                            source,
                        }),
                    ));

                    let _ = tokio::fs::remove_file(&path).await;
                }
            }
        };

        let future = async move {
            let _ = futures::future::join(event_handler, fetcher).await;
        };

        (future, rx)
    }
}

#[derive(Debug)]
pub struct FetchEvent {
    pub package: Arc<CdnFile>,
    pub kind: EventKind,
}

impl FetchEvent {
    pub fn new(package: Arc<CdnFile>, kind: EventKind) -> Self { Self { package, kind } }
}

#[derive(Debug)]
pub enum EventKind {
    /// Request to download package is being initiated
    Fetching,

    /// Package was downloaded successfully
    Fetched,

    /// An error occurred fetching package
    Error(FetchError),

    /// The package has been validated
    Validated,

    // Package is being retried
    Retrying,
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("{}: fetched package had checksum error", package)]
    Checksum { package: String, source: ChecksumError },

    #[error("{}: download failed", package)]
    Fetch {
        package: String,
        source: async_fetcher::Error,
    },
}

#[derive(Debug, Error)]
pub enum ChecksumError {
    #[error("unable to open the file to validate")]
    FileOpen(#[source] io::Error),

    #[error(
        "file does not match expected size: found {} KiB but expected {} KiB",
        found,
        expected
    )]
    InvalidSize { found: u64, expected: u64 },

    #[error("Error while generating hashes: {0}")]
    HashGen(#[from] HashError),

    #[error("checksum mismatch")]
    Mismatch,
}

pub fn compare_hash(path: &Path, expected_size: u64, expected_hash: &Hashes) -> Result<(), ChecksumError> {
    let mut file = std::fs::File::open(path).map_err(ChecksumError::FileOpen)?;

    let file_size = file.metadata().unwrap().len();
    if file_size != expected_size {
        return Err(ChecksumError::InvalidSize {
            found: file_size / 1024,
            expected: expected_size / 1024,
        });
    }

    let file_hashes = Hashes::get_hashes_from_file(&mut file)?;

    if expected_hash != &file_hashes {
        return Err(ChecksumError::Mismatch);
    }

    Ok(())
}
