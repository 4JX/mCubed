use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    error::{self, LibResult},
    mod_file::ModFile,
    paths,
};

use super::storage_trait::StorageTrait;

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct CacheStorage {
    pub storage: Vec<ModFile>,
}

impl<'a> StorageTrait<'a> for CacheStorage {
    const FILE_NAME: &'static str = "mods.mCubed.json";

    fn get_folder() -> PathBuf {
        paths::default_mod_dir()
    }
}

impl CacheStorage {
    pub fn load_list_cache(&mut self) -> LibResult<()> {
        match Self::load() {
            Ok(cache) => {
                self.storage = cache.storage;
                Ok(())
            }
            Err(error) => match error {
                error::Error::SerdeError(err) => {
                    tracing::error!("Failed to parse cache: {}", err);
                    Err(error::Error::FailedToParseEntryCache { err })
                }
                _ => Err(error),
            },
        }
    }

    pub fn save_list_cache(&self) -> LibResult<()> {
        self.save()
    }

    pub fn get_cache(&self) -> &Vec<ModFile> {
        &self.storage
    }

    pub fn get_cache_mut(&mut self) -> &mut Vec<ModFile> {
        &mut self.storage
    }
}
