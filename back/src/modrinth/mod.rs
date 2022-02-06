use bytes::Bytes;
use ferinth::{
    structures::version_structs::{ListVersionsParams, Version, VersionType},
    Ferinth,
};

use crate::mod_entry::{FileState, ModEntry, ModrinthData, Source};

pub struct Modrinth {
    ferinth: Ferinth,
}

impl Default for Modrinth {
    fn default() -> Self {
        Self {
            ferinth: Ferinth::new("Very much still a test app"),
        }
    }
}

impl Modrinth {
    async fn get_modrinth_id(&self, mod_hash: &str) -> Option<String> {
        match self.ferinth.get_version_from_file_hash(mod_hash).await {
            Ok(result) => Some(result.project_id),
            Err(_err) => None,
        }
    }

    pub async fn check_for_updates(&self, game_version: &str, mod_entry: &mut ModEntry) {
        if mod_entry.sourced_from == Source::Modrinth || mod_entry.sourced_from == Source::Local {
            // Get and set the modrinth ID, without one the operation cannot proceed
            if mod_entry.modrinth_data.is_none() {
                let modrinth_id = self.get_modrinth_id(&mod_entry.hashes.sha1).await;
                if let Some(id) = modrinth_id {
                    mod_entry.modrinth_data = Some(ModrinthData {
                        id,
                        latest_valid_version: None,
                    });

                    mod_entry.sourced_from = Source::Modrinth;
                }
            }

            // This will not always give a result, therefore the data needs to be checked again (In case it is "Some", assume its correct)
            if let Some(modrinth_data) = &mut mod_entry.modrinth_data {
                let query_params = ListVersionsParams {
                    loaders: Some(mod_entry.modloader.clone().into()),
                    game_versions: Some(
                        vec![game_version].iter().map(ToString::to_string).collect(),
                    ),
                    featured: None,
                };

                // The version list can now be fetched
                match self
                    .ferinth
                    .list_versions(modrinth_data.id.as_str(), Some(query_params))
                    .await
                {
                    Ok(version_list) => {
                        if version_list.is_empty() {
                            // No versions could be found that match the criteria, therefore the mod is incompatible for this version
                            mod_entry.state = FileState::Invalid;
                        } else {
                            // There are results, consider the state to be up to date unless proven otherwise
                            mod_entry.state = FileState::Current;

                            let filtered_list: Vec<&Version> = version_list
                                .iter()
                                .filter(|version| {
                                    version.version_type == VersionType::Release
                                        && version.loaders.contains(
                                            &mod_entry.modloader.to_string().to_lowercase(),
                                        )
                                        && !version.files.is_empty()
                                })
                                .collect();

                            if !filtered_list.is_empty() {
                                {
                                    // If the version being checked contains a file with the hash of our local copy, it means it is already on the latest possible version
                                    if !filtered_list[0].files.iter().any(|file| {
                                        file.hashes.sha1 == Some(mod_entry.hashes.sha1.clone())
                                    }) {
                                        modrinth_data.latest_valid_version =
                                            Some(filtered_list[0].files[0].clone());
                                        mod_entry.state = FileState::Outdated;
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        dbg!(&err);
                    }
                };
            }
        }
    }

    pub async fn update_mod(&self, mod_entry: &ModEntry) -> Option<Bytes> {
        if let Some(data) = &mod_entry.modrinth_data {
            if let Some(version_file) = &data.latest_valid_version {
                Some(
                    self.ferinth
                        .download_version_file(version_file)
                        .await
                        .unwrap(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }
}
