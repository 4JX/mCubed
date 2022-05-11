use serde::{Deserialize, Serialize};

use crate::{
    error::{self, LibResult},
    mod_file::ModFile,
};

use super::{settings::CONF, storage_trait::StorageTrait};

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct CacheStorage {
    pub storage: Vec<ModFile>,
}

impl<'a> StorageTrait<'a> for CacheStorage {
    const FILE_NAME: &'static str = "mods.mCubed.json";

    type Result = LibResult<Self>;
}

impl CacheStorage {
    pub fn load_list_cache(&mut self) -> LibResult<()> {
        match Self::load(&CONF.lock().mod_folder_path) {
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
        self.save(&CONF.lock().mod_folder_path)
    }

    pub fn get_cache(&self) -> &Vec<ModFile> {
        &self.storage
    }

    pub fn get_cache_mut(&mut self) -> &mut Vec<ModFile> {
        &mut self.storage
    }
}
