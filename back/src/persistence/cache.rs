use std::path::{Path, PathBuf};

use crate::{
    error::{self, LibResult},
    mod_entry::ModEntry,
};

use tracing::error;

use super::storage::Storage;

const APP_CACHE_FILE_NAME: &str = "mods.mCubed.json";

pub struct CacheStorage {
    inner: Storage<Vec<ModEntry>>,
}

impl CacheStorage {
    pub fn new(folder_path: &Path) -> Self {
        let json_filepath: PathBuf = folder_path.join(APP_CACHE_FILE_NAME);

        Self {
            inner: Storage {
                json_filepath,
                storage: Default::default(),
            },
        }
    }

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

    pub fn save_list_cache(&self) -> LibResult<()> {
        self.inner.save()
    }

    pub fn set_cache(&mut self, new_list: Vec<ModEntry>) {
        self.inner.set(new_list);
    }

    pub fn get_cache(&self) -> &Vec<ModEntry> {
        self.inner.get()
    }

    pub fn get_cache_mut(&mut self) -> &mut Vec<ModEntry> {
        self.inner.get_mut()
    }
}
