// https://github.com/MinecraftForge/Documentation/blob/1.18.x/docs/gettingstarted/structuring.md

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
};

use serde::Deserialize;

use crate::{
    error::{Error, LibResult},
    get_modloaders, ModLoader,
};

pub const FORGE_META_PATH: &str = "META-INF/mods.toml";

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ForgeManifest {
    #[serde(rename = "modLoader")]
    pub mod_loader: String,
    #[serde(rename = "loaderVersion")]
    pub loader_version: String,
    pub license: String,
    #[serde(rename = "issueTrackerURL")]
    pub issue_tracker_url: String,
    #[serde(rename = "showAsResourcePack")]
    pub show_as_resource_pack: Option<bool>,
    pub mods: Vec<ForgeModEntry>,
    pub dependencies: HashMap<String, Vec<Dependency>>,
}

impl ForgeManifest {
    pub fn from_buffer(buf: &str) -> LibResult<Self> {
        match toml::from_str(buf) {
            Ok(metadata) => Ok(metadata),
            Err(err) => Err(err.into()),
        }
    }

    pub fn from_file(file: &mut File) -> LibResult<Self> {
        let modloaders = get_modloaders(file)?;

        if modloaders.contains(&ModLoader::Forge) {
            let reader = BufReader::new(file);

            let mut archive = zip::ZipArchive::new(reader)?;

            let file = archive.by_name(FORGE_META_PATH);

            match file {
                Ok(mut zip_file) => {
                    let mut buf = String::new();
                    zip_file.read_to_string(&mut buf)?;

                    Ok(Self::from_buffer(buf.as_str())?)
                }
                Err(err) => Err(err.into()),
            }
        } else {
            Err(Error::IncorrectModloader)
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct ForgeModEntry {
    #[serde(rename = "modId")]
    pub mod_id: String,
    pub version: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "updateJSONURL")]
    pub update_json_url: Option<String>,
    #[serde(rename = "displayURL")]
    pub display_url: Option<String>,
    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,
    pub credits: Option<String>,
    pub authors: Option<String>,
    pub description: String,
}

#[derive(Deserialize, Clone)]
pub struct Dependency {
    #[serde(rename = "modId")]
    pub mod_id: String,
    pub mandatory: bool,
    #[serde(rename = "versionRange")]
    pub version_range: String,
    pub ordering: Ordering,
}

#[derive(Deserialize, Clone)]
pub enum Ordering {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "BEFORE")]
    Before,
    #[serde(rename = "AFTER")]
    After,
}

#[derive(Deserialize)]
pub enum Environment {
    #[serde(rename = "BOTH")]
    Both,
    #[serde(rename = "CLIENT")]
    Client,
    #[serde(rename = "SERVER")]
    Server,
}
