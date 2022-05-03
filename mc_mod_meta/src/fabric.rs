// https://fabricmc.net/wiki/documentation:fabric_mod_json

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
};

use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::{Error, LibResult},
    get_modloader, ModLoader,
};

pub const FABRIC_META_PATH: &str = "fabric.mod.json";

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FabricManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i32,
    pub id: String,
    pub version: String,

    //Mod Loading
    pub environment: Option<Environment>,
    pub entrypoints: Option<Entrypoints>,
    pub jars: Option<Vec<JarFilePath>>,
    #[serde(rename = "languageAdapters")]
    pub language_adapters: Option<HashMap<String, String>>,
    pub mixins: Option<Vec<Mixin>>,
    #[serde(rename = "accessWidener")]
    pub access_widener: Option<String>,

    //Dependency resolution
    pub depends: Option<HashMap<String, DependencyVersion>>,
    pub recommends: Option<HashMap<String, DependencyVersion>>,
    pub suggests: Option<HashMap<String, DependencyVersion>>,
    pub breaks: Option<HashMap<String, DependencyVersion>>,
    pub conflicts: Option<HashMap<String, DependencyVersion>>,

    //Metadata
    pub name: Option<String>,
    pub description: Option<String>,
    pub contact: Option<ContactObject>,
    pub authors: Option<Vec<Author>>,
    //The data contained within "contributors" has the same layout as "authors"
    pub contributors: Option<Vec<Author>>,
    pub license: Option<License>,
    pub icon: Option<String>,

    //Things inside the "custom" field will be parsed to the best of Serde's abilities
    pub custom: Option<HashMap<String, Value>>,
}

impl FabricManifest {
    pub fn from_buffer(buf: &str) -> LibResult<Self> {
        match serde_json::from_str(buf) {
            Ok(metadata) => Ok(metadata),
            Err(err) => Err(err.into()),
        }
    }

    pub fn from_file(file: &mut File) -> LibResult<Self> {
        let modloader = get_modloader(file)?;

        if modloader == ModLoader::Fabric || modloader == ModLoader::Both {
            let reader = BufReader::new(file);

            let mut archive = zip::ZipArchive::new(reader)?;

            let file = archive.by_name(FABRIC_META_PATH);

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

//* Mod loading
#[derive(Deserialize, Clone)]
pub enum Environment {
    #[serde(rename = "*")]
    Both,
    #[serde(rename = "client")]
    Client,
    #[serde(rename = "server")]
    Server,
}

#[derive(Deserialize, Clone)]
pub struct Entrypoints {
    #[serde(default)]
    pub main: Vec<Entrypoint>,
    #[serde(default)]
    pub client: Vec<Entrypoint>,
    #[serde(default)]
    pub server: Vec<Entrypoint>,
    #[serde(default)]
    #[serde(rename = "preLaunch")]
    pub prelaunch: Vec<Entrypoint>,

    //A catch-all for custom entrypoints added by other mods
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Entrypoint {
    JavaEntrypoint(String),
    AdapterEntrypoint(AdapterEntrypoint),
}

#[derive(Deserialize, Clone)]
pub struct AdapterEntrypoint {
    pub adapter: String,
    pub value: String,
}

#[derive(Deserialize, Clone)]
pub struct JarFilePath {
    pub file: String,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Mixin {
    Path(String),
    MixinObject(MixinObject),
}

#[derive(Deserialize, Clone)]
pub struct MixinObject {
    pub config: String,
    pub environment: Environment,
}

//* Dependency resolution
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum DependencyVersion {
    Single(String),
    Multiple(Vec<String>),
}

//* Metadata
#[derive(Deserialize, Clone)]
pub struct ContactObject {
    pub email: Option<String>,
    pub irc: Option<String>,
    pub homepage: Option<String>,
    pub issues: Option<String>,
    pub sources: Option<String>,

    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Author {
    Name(String),
    AuthorObject(AuthorObject),
}

#[derive(Deserialize, Clone)]
pub struct AuthorObject {
    pub name: String,
    pub contact: String,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum License {
    Single(String),
    Multiple(Vec<String>),
}
