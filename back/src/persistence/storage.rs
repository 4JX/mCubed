use std::{fs::File, io::Write, path::PathBuf};

use serde::{de::DeserializeOwned, Serialize};

use crate::error::LibResult;

#[derive(Debug)]
pub struct Storage<T> {
    pub json_filepath: PathBuf,
    pub storage: T,
}

#[allow(dead_code)]
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
        let stringified_json = serde_json::to_string(&self.storage)?;
        file.write_all(stringified_json.as_bytes())?;
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
