use core::fmt;
use std::{fmt::Debug, path::PathBuf};

use ferinth::structures::version_structs::{ModLoader as FeModLoader, VersionFile};
use mc_mod_meta::{fabric::FabricManifest, forge::ForgeModEntry, ModLoader as McModLoader};

use serde::{Deserialize, Serialize};
use tracing::instrument;

pub use self::hash::Hashes;

pub mod from_file;
pub mod hash;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModEntry {
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub description: Option<String>,
    pub authors: Option<String>,
    pub modloader: ModLoader,
    pub hashes: Hashes,
    pub sources: Sources,
    pub state: FileState,
    pub sourced_from: CurrentSource,
    pub path: PathBuf,
    #[serde(skip_serializing, skip_deserializing)]
    pub icon: Option<Vec<u8>>,
}

// Middleman "ModLoader" enum to convert between those of the other crates
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModLoader {
    Forge,
    Fabric,
    Both,
}

impl Default for ModLoader {
    fn default() -> Self {
        Self::Both
    }
}

impl fmt::Display for ModLoader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<ModLoader> for FeModLoader {
    fn from(modloader: ModLoader) -> Self {
        match modloader {
            ModLoader::Forge => Self::Forge,
            ModLoader::Fabric => Self::Fabric,
            ModLoader::Both => Self::Fabric,
        }
    }
}

impl From<McModLoader> for ModLoader {
    fn from(modloader: McModLoader) -> Self {
        match modloader {
            McModLoader::Forge => Self::Forge,
            McModLoader::Fabric => Self::Fabric,
            McModLoader::Both => Self::Both,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Sources {
    pub curseforge: Option<CurseForgeData>,
    pub modrinth: Option<ModrinthData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurseForgeData;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModrinthData {
    pub id: String,
    pub latest_valid_version: Option<VersionFile>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileState {
    Current,
    Outdated,
    Invalid,
    Local,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum CurrentSource {
    None,
    Local,
    Modrinth,
    CurseForge,
}

impl fmt::Display for CurrentSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ModEntry {
    #[instrument(skip(forge_mod_entry, hashes, sources, path), level = "debug")]
    fn from_forge_manifest(
        forge_mod_entry: ForgeModEntry,
        hashes: Hashes,
        sources: Option<Sources>,
        path: PathBuf,
    ) -> Self {
        Self {
            id: forge_mod_entry.mod_id,
            version: forge_mod_entry.version,
            display_name: forge_mod_entry.display_name,
            description: Some(forge_mod_entry.description),
            authors: forge_mod_entry.authors,
            modloader: ModLoader::Forge,
            hashes,
            sources: sources.unwrap_or_default(),
            state: FileState::Local,
            sourced_from: CurrentSource::None,
            path,
            icon: None,
        }
    }

    #[instrument(skip(fabric_manifest, hashes, sources, path), level = "debug")]
    fn from_fabric_manifest(
        fabric_manifest: FabricManifest,
        hashes: Hashes,
        sources: Option<Sources>,
        path: PathBuf,
    ) -> Self {
        let mod_name = fabric_manifest
            .name
            .unwrap_or_else(|| fabric_manifest.id.clone());

        let parsed_authors = fabric_manifest.authors.map_or_else(
            || None,
            |authors| {
                Some(
                    authors
                        .iter()
                        .map(|author| match author {
                            mc_mod_meta::fabric::Author::Name(name) => name,
                            mc_mod_meta::fabric::Author::AuthorObject(object) => &object.name,
                        })
                        .cloned()
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            },
        );

        Self {
            id: fabric_manifest.id,
            version: fabric_manifest.version,
            display_name: mod_name,
            description: fabric_manifest.description,
            authors: parsed_authors,
            modloader: ModLoader::Fabric,
            hashes,
            sources: sources.unwrap_or_default(),
            state: FileState::Local,
            sourced_from: CurrentSource::None,
            path,
            icon: None,
        }
    }
}
