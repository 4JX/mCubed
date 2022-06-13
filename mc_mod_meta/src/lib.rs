// TODO: Add the Quilt manifest whenever that becomes stable, see https://github.com/QuiltMC/rfcs/blob/master/specification/0002-quilt.mod.json.md

use core::fmt;
use std::{fs::File, io::BufReader};

use error::LibResult;
use fabric::FABRIC_META_PATH;
use forge::FORGE_META_PATH;

pub mod error;
pub mod fabric;
pub mod forge;

pub fn get_modloaders(file: &File) -> LibResult<Vec<ModLoader>> {
    let reader = BufReader::new(file);

    let archive = zip::ZipArchive::new(reader)?;

    let names: Vec<String> = archive.file_names().map(ToString::to_string).collect();

    let mut modloader_vec = Vec::new();

    if names.contains(&FORGE_META_PATH.to_string()) {
        modloader_vec.push(ModLoader::Forge);
    }

    if names.contains(&FABRIC_META_PATH.to_string()) {
        modloader_vec.push(ModLoader::Fabric);
    }

    Ok(modloader_vec)
}

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub enum ModLoader {
    Forge,
    Fabric,
}

impl fmt::Display for ModLoader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(self, f) }
}
