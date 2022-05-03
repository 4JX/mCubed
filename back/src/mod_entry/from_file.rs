use std::{
    fs,
    io::{BufReader, Read},
    path::PathBuf,
};

use mc_mod_meta::{fabric::FabricManifest, forge::ForgeManifest};
use tracing::instrument;

use crate::{error::LibResult, settings::CONF};

use super::{Hashes, ModEntry, ModLoader};

impl ModEntry {
    #[instrument(level = "debug")]
    pub fn from_path(path: PathBuf) -> LibResult<Vec<Self>> {
        let mut file = fs::File::open(&path)?;

        let hashes = Hashes::get_hashes_from_file(&mut file)?;

        let mut mod_vec = Vec::new();

        let modloader = mc_mod_meta::get_modloader(&file)?;
        match modloader {
            mc_mod_meta::ModLoader::Forge => {
                let forge_meta = ForgeManifest::from_file(&mut file)?;
                for forge_mod_entry in forge_meta.mods {
                    let icon_path = forge_mod_entry.logo_file.clone();
                    let mod_entry = ModEntry::from_forge_manifest(
                        forge_mod_entry,
                        hashes.clone(),
                        None,
                        path.clone(),
                    );

                    add_to_mod_vec(&mut mod_vec, &file, mod_entry, icon_path)?;
                }
            }
            mc_mod_meta::ModLoader::Fabric => {
                let fabric_manifest = FabricManifest::from_file(&mut file)?;
                let icon_path = fabric_manifest.icon.clone();

                let mod_entry = ModEntry::from_fabric_manifest(fabric_manifest, hashes, None, path);
                add_to_mod_vec(&mut mod_vec, &file, mod_entry, icon_path)?;
            }

            mc_mod_meta::ModLoader::Both => {
                // Given the mod has entries for both forge and fabric, simplify things by just displaying one entry with the fabric data
                let fabric_manifest = FabricManifest::from_file(&mut file)?;
                let icon_path = fabric_manifest.icon.clone();

                let mut mod_entry =
                    ModEntry::from_fabric_manifest(fabric_manifest, hashes, None, path);

                // However, the modloader is replaced with the "Both" type
                mod_entry.modloader = ModLoader::Both;

                add_to_mod_vec(&mut mod_vec, &file, mod_entry, icon_path)?
            }
        };

        Ok(mod_vec)
    }
}

fn add_to_mod_vec(
    mod_vec: &mut Vec<ModEntry>,
    file: &fs::File,
    mut mod_entry: ModEntry,
    icon_path: Option<String>,
) -> LibResult<()> {
    if let Some(icon_path) = icon_path {
        if let Ok(icon) = load_icon(file, &icon_path) {
            mod_entry.icon = Some(icon);
        }
    }

    mod_vec.push(mod_entry);
    Ok(())
}

fn load_icon(zip_file: &fs::File, icon_path: &str) -> LibResult<Vec<u8>> {
    let reader = BufReader::new(zip_file);
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut file = archive.by_name(icon_path)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    let icon_size = CONF.lock().icon_resize_size;
    let image = image::load_from_memory(&buf)
        .unwrap()
        .resize(icon_size, icon_size, image::imageops::FilterType::Triangle)
        .to_rgba8()
        .to_vec();
    Ok(image)
}
