use crate::{fabric::FabricManifest, forge::ForgeModEntry, ModLoader};

#[derive(Debug, Clone)]
pub struct MinecraftMod {
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub modloader: ModLoader,
}

impl From<FabricManifest> for MinecraftMod {
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

impl From<ForgeModEntry> for MinecraftMod {
    fn from(mod_entry: ForgeModEntry) -> Self {
        Self {
            id: mod_entry.mod_id,
            version: mod_entry.version,
            display_name: mod_entry.display_name,
            modloader: ModLoader::Forge,
        }
    }
}
