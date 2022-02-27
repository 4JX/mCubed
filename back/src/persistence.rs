use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    error::{self, LibResult},
    mod_entry::ModEntry,
};

use serde::{de::DeserializeOwned, Serialize};
use tracing::error;

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

pub struct Storage<T> {
    json_filepath: PathBuf,
    storage: T,
}

impl<T> Storage<T>
where
    T: DeserializeOwned + Serialize + Default,
{
    pub fn load(&mut self) -> LibResult<()> {
        match std::fs::File::open(&self.json_filepath) {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                self.storage = serde_json::de::from_reader(reader)?;
                Ok(())
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    let new_value = T::default();
                    let mut file = File::create(&self.json_filepath)?;
                    file.write_all(serde_json::to_string(&new_value)?.as_bytes())?;
                    self.storage = new_value;
                    Ok(())
                }

                _ => Err(err.into()),
            },
        }
    }

    pub fn save(&self) -> LibResult<()> {
        let mut file = File::create(&self.json_filepath)?;
        file.write_all(serde_json::to_string(&self.storage)?.as_bytes())?;
        Ok(())
    }

    pub fn set(&mut self, new_storage: T) {
        self.storage = new_storage;
    }

    pub fn get(&self) -> &T {
        &self.storage
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.storage
    }
}
