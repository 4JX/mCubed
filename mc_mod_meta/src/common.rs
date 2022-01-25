use lazy_static::lazy_static;
use regex::Regex;

use crate::{fabric::FabricManifest, forge::ForgeModEntry, ModLoader};

#[derive(Debug, Clone)]
pub struct McMod {
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub modloader: ModLoader,
}

impl McMod {
    pub fn normalized_version(&self) -> String {
        lazy_static! {
            static ref VERSION_REGEX: Regex = Regex::new("[0-9]+\\.[0-9]+\\.[0-9]+").unwrap();
        };

        let version: String = VERSION_REGEX.captures(self.version.as_str()).map_or_else(
            || "Invalid".to_string(),
            |matches| matches.get(0).unwrap().as_str().to_string(),
        );

        version
    }
}

impl From<FabricManifest> for McMod {
    fn from(manifest: FabricManifest) -> Self {
        let mod_name = manifest.name.unwrap_or_else(|| manifest.id.clone());

        Self {
            id: manifest.id,
            version: manifest.version,
            display_name: mod_name,
            modloader: ModLoader::Fabric,
        }
    }
}

impl From<ForgeModEntry> for McMod {
    fn from(mod_entry: ForgeModEntry) -> Self {
        Self {
            id: mod_entry.mod_id,
            version: mod_entry.version,
            display_name: mod_entry.display_name,
            modloader: ModLoader::Forge,
        }
    }
}
