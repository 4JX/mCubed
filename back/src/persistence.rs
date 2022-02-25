use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    error::{self, LibResult},
    mod_entry::ModEntry,
};

use tracing::error;

const APP_CACHE_FILE_NAME: &str = "mods.mCubed.json";

pub struct CacheStorage {
    json_filepath: PathBuf,
    cache: Vec<ModEntry>,
}

impl CacheStorage {
    pub fn new(folder_path: &Path) -> Self {
        let json_filepath: PathBuf = folder_path.join(APP_CACHE_FILE_NAME);
        Self {
            json_filepath,
            cache: Vec::new(),
        }
    }

    pub fn load_list_cache(&mut self) -> LibResult<()> {
        match std::fs::File::open(&self.json_filepath) {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                match serde_json::de::from_reader(reader) {
                    Ok(value) => {
                        self.cache = value;
                        Ok(())
                    }
                    Err(err) => {
                        error!("Failed to parse cache: {}", err);
                        Err(error::Error::FailedToParseEntryCache { err })
                    }
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    let new_vec = Vec::new();
                    let mut file = File::create(&self.json_filepath)?;
                    file.write_all(serde_json::to_string(&new_vec)?.as_bytes())?;
                    self.cache = new_vec;
                    Ok(())
                }

                _ => Err(err.into()),
            },
        }
    }

    pub fn save_list_cache(&self) -> LibResult<()> {
        let mut file = File::create(&self.json_filepath)?;
        file.write_all(serde_json::to_string(&self.cache)?.as_bytes())?;
        Ok(())
    }

    pub fn set_cache(&mut self, new_list: Vec<ModEntry>) {
        self.cache = new_list;
    }

    pub fn get_cache(&self) -> &Vec<ModEntry> {
        &self.cache
    }
}
