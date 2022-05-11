use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ferinth::structures::version_structs::VersionType;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::{
    error::LibResult,
    paths::{self, CONFIG_DIR},
};

use super::storage_trait::StorageTrait;

lazy_static! {
    pub static ref CONF: Arc<Mutex<SettingsBuilder>> = Arc::new(Mutex::new(
        SettingsBuilder::load_from_file(CONFIG_DIR.as_path()).unwrap_or_default()
    ));
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SettingsBuilder {
    /// The size of the images the icon of a mod will be resized to
    pub icon_resize_size: u32,
    /// The minimum release type when fetching from modrinth
    pub modrinth_version_type: VersionType,
    /// The path to the "mods" folder
    pub mod_folder_path: PathBuf,
}

impl SettingsBuilder {
    /// Create a new [SettingsBuilder](SettingsBuilder) off of the default struct values
    #[must_use]
    pub fn new() -> Self {
        Self::from_current()
    }

    /// Create a new [SettingsBuilder](SettingsBuilder) from the current values
    #[must_use]
    pub fn from_current() -> Self {
        CONF.lock().clone()
    }

    /// Create a new [SettingsBuilder](SettingsBuilder) from a file on the disk
    pub fn load_from_file(folder_path: &Path) -> LibResult<Self> {
        Self::load(folder_path)
    }

    /// Save the current configuration to disk
    pub fn save_config(&self, folder_path: &Path) -> LibResult<()> {
        self.save(folder_path)
    }

    /// Set the icon resize size
    #[must_use]
    pub const fn icon_resize_size(mut self, size: u32) -> Self {
        self.icon_resize_size = size;
        self
    }

    /// Set the modrinth release type
    #[must_use]
    pub const fn modrinth_version_type(mut self, version_type: VersionType) -> Self {
        self.modrinth_version_type = version_type;
        self
    }

    /// Apply the configuration
    pub fn apply(self) {
        let mut changer = CONF.lock();
        *changer = self;
    }
}

impl Default for SettingsBuilder {
    fn default() -> Self {
        Self {
            icon_resize_size: 128,
            modrinth_version_type: VersionType::Release,
            mod_folder_path: paths::default_mod_dir(),
        }
    }
}

impl<'a> StorageTrait<'a> for SettingsBuilder {
    const FILE_NAME: &'static str = "settings.json";

    type Result = LibResult<Self>;
}
