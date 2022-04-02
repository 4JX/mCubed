use std::path::{Path, PathBuf};

use crate::{
    error::{self, LibResult},
    mod_entry::ModEntry,
};

use tracing::{error, instrument};

use super::storage::Storage;

const APP_CACHE_FILE_NAME: &str = "mods.mCubed.json";

#[derive(Debug)]
pub struct CacheStorage {
    inner: Storage<Vec<ModEntry>>,
}

impl CacheStorage {
    #[instrument(level = "trace")]
    pub fn new(folder_path: &Path) -> Self {
        let json_filepath: PathBuf = folder_path.join(APP_CACHE_FILE_NAME);

        Self {
            inner: Storage {
                json_filepath,
                storage: Vec::default(),
            },
        }
    }

    #[instrument(skip(self))]
    pub fn load_list_cache(&mut self) -> LibResult<()> {
        match self.inner.load() {
            Ok(()) => Ok(()),
            Err(error) => match error {
                error::Error::SerdeError(err) => {
                    error!("Failed to parse cache: {}", err);
                    Err(error::Error::FailedToParseEntryCache { err })
                }
                _ => Err(error),
            },
        }
    }

    #[instrument(skip(self))]
    pub fn save_list_cache(&self) -> LibResult<()> {
        self.inner.save()
    }

    #[instrument(skip(self, new_list))]
    pub fn set_cache(&mut self, new_list: Vec<ModEntry>) {
        self.inner.set(new_list);
    }

    #[instrument(skip(self))]
    pub fn get_cache(&self) -> &Vec<ModEntry> {
        self.inner.get()
    }

    #[instrument(skip(self))]
    pub fn get_cache_mut(&mut self) -> &mut Vec<ModEntry> {
        self.inner.get_mut()
    }
}
