//TODO: Add the Quilt manifest whenever that becomes stable, see https://github.com/QuiltMC/rfcs/blob/master/specification/0002-quilt.mod.json.md

use core::fmt;
use std::{fs::File, io::BufReader};

use error::AppResult;
use fabric::FABRIC_META_PATH;
use forge::FORGE_META_PATH;

pub mod common;
pub mod error;
pub mod fabric;
pub mod forge;

pub fn get_modloader(file: &File) -> AppResult<ModLoader> {
    let reader = BufReader::new(file);

    let archive = zip::ZipArchive::new(reader)?;

    let names: Vec<String> = archive.file_names().map(ToString::to_string).collect();

    if names.contains(&FORGE_META_PATH.to_string()) {
        Ok(ModLoader::Forge)
    } else if names.contains(&FABRIC_META_PATH.to_string()) {
        Ok(ModLoader::Fabric)
    } else {
        Err(error::Error::InvalidModFile)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ModLoader {
    Forge,
    Fabric,
}

impl fmt::Display for ModLoader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
