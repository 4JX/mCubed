use std::path::PathBuf;

use bytes::Bytes;
use ferinth::{
    structures::version_structs::{ListVersionsParams, Version},
    Ferinth,
};
use tracing::instrument;

use crate::{
    error::{self, LibResult},
    mod_entry::{CurrentSource, FileState, Hashes, ModFile, ModLoader, ModrinthData, Sources},
};

#[derive(Debug)]
pub struct Modrinth {
    ferinth: Ferinth,
}

impl Default for Modrinth {
    fn default() -> Self {
        Self {
            ferinth: Ferinth::new(),
        }
    }
}

impl Modrinth {
    #[instrument(skip(self))]
    pub(crate) async fn get_modrinth_id_from_hash(&self, mod_hash: &str) -> Option<String> {
        match self.ferinth.get_version_from_file_hash(mod_hash).await {
            Ok(result) => Some(result.project_id),
            Err(_err) => None,
        }
    }

    #[instrument(skip(self, mod_file))]
    pub(crate) async fn check_for_updates(
        &self,
        mod_file: &mut ModFile,
        game_version: &str,
    ) -> LibResult<()> {
        // Get and set the modrinth ID, without one the operation cannot proceed
        if mod_file.sources.modrinth.is_none() {
            let modrinth_id = self.get_modrinth_id_from_hash(&mod_file.hashes.sha1).await;

            if let Some(id) = modrinth_id {
                mod_file.sources.modrinth = Some(ModrinthData {
                    id,
                    latest_valid_version: None,
                });

                // If the source has not been set by the user, automatically track Modrinth
                if mod_file.sourced_from == CurrentSource::None {
                    mod_file.sourced_from = CurrentSource::Modrinth;
                }
            }
        }

        if mod_file.sourced_from == CurrentSource::Modrinth {
            // This will not always give a result, therefore the data needs to be checked again (In case it is "Some", assume its correct)
            if let Some(modrinth_data) = &mut mod_file.sources.modrinth {
                // The version list can now be fetched
                let version_list = self
                    .list_versions(&modrinth_data.id, &mod_file.loaders, game_version)
                    .await?;

                if version_list.is_empty() {
                    // No versions could be found that match the criteria, therefore the mod is incompatible for this version
                    mod_file.state = FileState::Invalid;
                } else {
                    // There are results, consider the state to be up to date unless proven otherwise
                    mod_file.state = FileState::Current;

                    let filtered_list: Vec<&Version> = version_list
                        .iter()
                        .filter(|version| {
                            // version.version_type == VersionType::Release
                            // &&
                            mod_file.loaders.iter().all(|loader| {
                                version.loaders.contains(&loader.to_string().to_lowercase())
                            }) && !version.files.is_empty()
                        })
                        .collect();

                    if !filtered_list.is_empty() {
                        {
                            // If the version being checked contains a file with the hash of our local copy, it means it is already on the latest possible version
                            if !filtered_list[0]
                                .files
                                .iter()
                                .any(|file| file.hashes.sha1 == Some(mod_file.hashes.sha1.clone()))
                            {
                                modrinth_data.latest_valid_version =
                                    Some(filtered_list[0].files[0].clone());
                                mod_file.state = FileState::Outdated;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self, mod_entry))]
    pub(crate) async fn update_mod(&self, mod_entry: &ModFile) -> LibResult<Bytes> {
        if let Some(data) = &mod_entry.sources.modrinth {
            if let Some(version_file) = &data.latest_valid_version {
                Ok(self.ferinth.download_version_file(version_file).await?)
            } else {
                Err(error::Error::InvalidLatestVersionError)
            }
        } else {
            Err(error::Error::NoModrinthDataError)
        }
    }

    #[instrument(skip(self))]
    pub(crate) async fn create_mod_file(
        &self,
        modrinth_id: String,
        game_version: String,
        modloader: ModLoader,
    ) -> LibResult<(ModFile, Bytes)> {
        match self.ferinth.get_project(modrinth_id.as_str()).await {
            Ok(project) => {
                let modrinth = ModrinthData {
                    id: project.id,
                    latest_valid_version: None,
                };

                let sources = Sources {
                    curseforge: None,
                    modrinth: Some(modrinth),
                };

                let loaders = vec![modloader];

                // Create an entry from whatever data is available
                let mut mod_entry = ModFile {
                    state: FileState::Current,
                    sourced_from: CurrentSource::Modrinth,
                    entries: Vec::new(),
                    hashes: Hashes::dummy(),
                    path: PathBuf::new(),
                    loaders,
                    sources,
                };

                self.check_for_updates(&mut mod_entry, &game_version)
                    .await?;

                let bytes = self.update_mod(&mod_entry).await?;
                Ok((mod_entry, bytes))
            }
            Err(_err) => Err(error::Error::NotValidModrinthId),
        }
    }

    #[instrument(skip(self))]
    async fn list_versions(
        &self,
        modrinth_id: &str,
        loaders: &Vec<ModLoader>,
        game_version: &str,
    ) -> LibResult<Vec<ferinth::structures::version_structs::Version>> {
        let query_params = ListVersionsParams {
            loaders: Some(loaders.iter().map(|loader| (*loader).into()).collect()),
            game_versions: Some(vec![game_version].iter().map(ToString::to_string).collect()),
            featured: None,
        };

        Ok(self
            .ferinth
            .list_versions(modrinth_id, Some(query_params))
            .await?)
    }
}
