use core::fmt;
use std::{fmt::Debug, fs::File, path::PathBuf};

use ferinth::structures::version_structs::{ModLoader as FeModLoader, VersionFile};
use lazy_static::lazy_static;
use mc_mod_meta::{
    common::MinecraftMod, fabric::FabricManifest, forge::ForgeManifest, ModLoader as McModLoader,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{error::LibResult, hash::Hashes};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModEntry {
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub modloader: ModLoader,
    pub hashes: Hashes,
    pub sources: Sources,
    pub state: FileState,
    pub sourced_from: CurrentSource,
    pub path: Option<PathBuf>,
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
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CurrentSource {
    Local,
    ExplicitLocal,
    Modrinth,
    CurseForge,
}

impl Debug for CurrentSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::ExplicitLocal => write!(f, "Local"),
            Self::Modrinth => write!(f, "Modrinth"),
            Self::CurseForge => write!(f, "CurseForge"),
        }
    }
}

impl fmt::Display for CurrentSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ModEntry {
    fn new(
        mc_mod: MinecraftMod,
        hashes: &Hashes,
        sources: Option<Sources>,
        path: Option<PathBuf>,
    ) -> Self {
        let MinecraftMod {
            id,
            version,
            display_name,
            modloader,
        } = mc_mod;

        Self {
            id,
            version,
            display_name,
            modloader: modloader.into(),
            hashes: hashes.clone(),
            sources: sources.unwrap_or_default(),
            state: FileState::Local,
            sourced_from: CurrentSource::Local,
            path,
        }
    }

    pub fn from_file(file: &mut File, path: Option<PathBuf>) -> LibResult<Vec<Self>> {
        let hashes = Hashes::get_hashes_from_file(file)?;

        let mut mod_vec = Vec::new();

        let modloader = mc_mod_meta::get_modloader(file)?;
        match modloader {
            mc_mod_meta::ModLoader::Forge => {
                let forge_meta = ForgeManifest::from_file(file)?;
                for mod_meta in forge_meta.mods {
                    let mc_mod = MinecraftMod::from(mod_meta);
                    mod_vec.push(Self::new(mc_mod, &hashes, None, path.clone()));
                }
            }
            mc_mod_meta::ModLoader::Fabric => {
                let mod_meta = FabricManifest::from_file(file)?;
                let mc_mod = MinecraftMod::from(mod_meta);
                mod_vec.push(Self::new(mc_mod, &hashes, None, path));
            }

            mc_mod_meta::ModLoader::Both => {
                // Given the mod has entries for both forge and fabric, simplify things by just displaying one entry with the fabric data
                let mod_meta = FabricManifest::from_file(file)?;
                let mut mc_mod = MinecraftMod::from(mod_meta);

                // However, the modloader is replaced with the "Both" type
                mc_mod.modloader = mc_mod_meta::ModLoader::Both;
                mod_vec.push(Self::new(mc_mod, &hashes, None, path));
            }
        };

        Ok(mod_vec)
    }

    #[must_use]
    pub fn normalized_version(&self) -> String {
        lazy_static! {
            static ref VERSION_REGEX: Regex = Regex::new("[0-9]+\\.[0-9]+\\.[0-9]+").unwrap();
        };

        let version: String = VERSION_REGEX.captures(self.version.as_str()).map_or_else(
            || self.version.clone(),
            |matches| matches.get(0).unwrap().as_str().to_string(),
        );

        version
    }
}
