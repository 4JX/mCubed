use ferinth::{
    structures::version_structs::{ListVersionsParams, VersionType},
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
    async fn get_modrinth_id(&self, mod_hash: &String) -> Option<String> {
        match self
            .ferinth
            .get_version_from_file_hash(mod_hash.as_str())
            .await
        {
            Ok(result) => Some(result.project_id),
            Err(_err) => None,
        }
    }

    pub async fn check_for_updates(&self, mod_entry: &mut ModEntry) {
        if mod_entry.sourced_from == Source::Modrinth || mod_entry.sourced_from == Source::Local {
            // Get and set the modrinth ID, without one the operation cannot proceed
            if let None = &mod_entry.modrinth_data {
                let modrinth_id = self.get_modrinth_id(&mod_entry.hashes.sha1).await;
                if let Some(id) = modrinth_id {
                    mod_entry.modrinth_data = Some(ModrinthData {
                        id,
                        latest_valid_version: None,
                    })
                }
            }

            // This will not always give a result, therefore the data needs to be checked again (In case it is "Some", assume its correct)
            if let Some(modrinth_data) = &mut mod_entry.modrinth_data {
                let query_params = ListVersionsParams {
                    loaders: Some(mod_entry.modloader.clone().into()),
                    game_versions: None,
                    featured: None,
                };

                // The version list can now be fetched
                match self
                    .ferinth
                    .list_versions(modrinth_data.id.as_str(), Some(query_params))
                    .await
                {
                    Ok(version_list) => {
                        if !version_list.is_empty() {
                            // There are results, set the source to Modrinth and consider the state to be up to date unless proven otherwise
                            mod_entry.sourced_from = Source::Modrinth;
                            mod_entry.state = FileState::Current;

                            'find_version_loop: for version in version_list {
                                if version
                                    .loaders
                                    .contains(&mod_entry.modloader.to_string().to_lowercase())
                                    && version.version_type == VersionType::Release
                                {
                                    if !version.files.is_empty() {
                                        modrinth_data.latest_valid_version = Some(version.id);
                                        mod_entry.state = FileState::Outdated;
                                        break 'find_version_loop;
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
}
