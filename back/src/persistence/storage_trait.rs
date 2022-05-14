use crate::error::LibResult;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

pub(super) trait StorageTrait<'a>
where
    Self: DeserializeOwned + Serialize + Default + Sized,
    for<'de> Self: Deserialize<'de> + 'a,
{
    const FILE_NAME: &'static str;

    fn get_folder() -> PathBuf;

    fn load() -> LibResult<Self> {
        let folder_path = Self::get_folder();

        if !folder_path.exists() {
            fs::create_dir_all(&folder_path)?;
        }
        let path = folder_path.join(Self::FILE_NAME);
        match std::fs::File::open(&path) {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                Ok(serde_json::de::from_reader(reader)?)
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    let new_value = Self::default();
                    let mut file = File::create(path)?;
                    file.write_all(serde_json::to_string(&new_value)?.as_bytes())?;
                    Ok(new_value)
                }

                _ => Err(err.into()),
            },
        }
    }

    fn save(&self) -> LibResult<()> {
        let folder_path = Self::get_folder();

        if !folder_path.exists() {
            fs::create_dir_all(&folder_path)?;
        }
        let path = folder_path.join(Self::FILE_NAME);
        let mut file = File::create(path)?;
        let stringified_json = serde_json::to_string(&self)?;
        file.write_all(stringified_json.as_bytes())?;
        Ok(())
    }
}
