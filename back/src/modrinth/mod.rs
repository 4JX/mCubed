use bytes::Bytes;
use ferinth::{
    structures::version_structs::{ListVersionsParams, Version, VersionType},
    Ferinth,
};

use crate::{
    error::{self, LibResult},
    hash,
    mod_entry::{FileState, ModEntry, ModLoader, ModrinthData, Source},
};

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
    /// Ensures you are working with IDs and not slugs
    pub(crate) async fn normalize_modrinth_id(&self, slug_or_id: &str) -> Option<String> {
        match self.ferinth.get_project(slug_or_id).await {
            Ok(project) => Some(project.id),
            Err(_err) => None,
        }
    }

    pub(crate) async fn get_modrinth_id_from_hash(&self, mod_hash: &str) -> Option<String> {
        match self.ferinth.get_version_from_file_hash(mod_hash).await {
            Ok(result) => Some(result.project_id),
            Err(_err) => None,
        }
    }

    pub(crate) async fn check_for_updates(&self, mod_entry: &mut ModEntry, game_version: &str) {
        if mod_entry.sourced_from == Source::Modrinth || mod_entry.sourced_from == Source::Local {
            // Get and set the modrinth ID, without one the operation cannot proceed
            if mod_entry.modrinth_data.is_none() {
                let modrinth_id = self.get_modrinth_id_from_hash(&mod_entry.hashes.sha1).await;
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
                // The version list can now be fetched
                let version_list = self
                    .list_versions(&modrinth_data.id, mod_entry.modloader, game_version)
                    .await
                    .unwrap();

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
                                && version
                                    .loaders
                                    .contains(&mod_entry.modloader.to_string().to_lowercase())
                                && !version.files.is_empty()
                        })
                        .collect();

                    if !filtered_list.is_empty() {
                        {
                            // If the version being checked contains a file with the hash of our local copy, it means it is already on the latest possible version
                            if !filtered_list[0]
                                .files
                                .iter()
                                .any(|file| file.hashes.sha1 == Some(mod_entry.hashes.sha1.clone()))
                            {
                                modrinth_data.latest_valid_version =
                                    Some(filtered_list[0].files[0].clone());
                                mod_entry.state = FileState::Outdated;
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) async fn update_mod(&self, mod_entry: &ModEntry) -> LibResult<Bytes> {
        if let Some(data) = &mod_entry.modrinth_data {
            if let Some(version_file) = &data.latest_valid_version {
                Ok(self.ferinth.download_version_file(version_file).await?)
            } else {
                Err(error::Error::InvalidLatestVersionError)
            }
        } else {
            Err(error::Error::NoModrinthDataError)
        }
    }

    pub(crate) async fn create_mod_entry(
        &self,
        modrinth_id: String,
        game_version: String,
        modloader: ModLoader,
    ) -> LibResult<ModEntry> {
        let project = self.ferinth.get_project(modrinth_id.as_str()).await?;

        let modrinth_data = ModrinthData {
            id: modrinth_id,
            latest_valid_version: None,
        };

        let mut mod_entry = ModEntry {
            id: project.slug,
            version: "0.0.0".to_string(),
            display_name: project.title,
            modloader,
            hashes: hash::Hashes::dummy(),
            modrinth_data: Some(modrinth_data),
            state: FileState::Current,
            sourced_from: Source::Modrinth,
        };

        self.check_for_updates(&mut mod_entry, &game_version).await;

        Ok(mod_entry)
    }

    async fn list_versions(
        &self,
        modrinth_id: &str,
        modloader: ModLoader,
        game_version: &str,
    ) -> LibResult<Vec<ferinth::structures::version_structs::Version>> {
        let query_params = ListVersionsParams {
            loaders: Some(modloader.into()),
            game_versions: Some(vec![game_version].iter().map(ToString::to_string).collect()),
            featured: None,
        };

        Ok(self
            .ferinth
            .list_versions(modrinth_id, Some(query_params))
            .await?)
    }
}
