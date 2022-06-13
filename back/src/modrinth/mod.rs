use std::collections::HashMap;

use ferinth::{
    structures::version_structs::{Version, VersionType},
    Ferinth,
};
use tracing::instrument;

use crate::{
    error::{self, LibResult},
    mod_file::{CurrentSource, FileState, Hashes, ModFileData, ModLoader, ModrinthData, Sources},
    settings::CONF,
};

#[derive(Debug, Default)]
pub struct Modrinth {
    ferinth: Ferinth,
}

impl Modrinth {
    #[instrument(skip(self))]
    pub(crate) async fn get_modrinth_id_from_hash(&self, mod_hash: &str) -> LibResult<String> {
        let version = self.ferinth.get_version_from_hash(mod_hash).await?;
        Ok(version.project_id)
    }

    #[instrument(skip(self, data))]
    pub(crate) async fn check_for_updates(&self, data: Vec<&mut ModFileData>, game_version: &str) -> LibResult<()> {
        let meets_req: Vec<&mut ModFileData> = data
            .into_iter()
            .filter(|e| e.sourced_from == CurrentSource::Modrinth && e.sources.modrinth.is_some())
            .collect();

        let mut handles = Vec::new();
        for mod_data in meets_req {
            handles.push(async {
                let modrinth_data = mod_data.sources.modrinth.as_mut().unwrap();
                // The version list can now be fetched
                let version_list = self
                    .list_versions(&modrinth_data.project_id, &mod_data.loaders, &[game_version])
                    .await?;

                if version_list.is_empty() {
                    // No versions could be found that match the criteria, therefore the mod is incompatible for this version
                    mod_data.state = FileState::Invalid;
                } else {
                    // There are results, consider the state to be up to date unless proven otherwise
                    mod_data.state = FileState::Current;

                    let accepted_version_types = accepted_versions_vec();
                    let filtered_list: Vec<&Version> = version_list
                        .iter()
                        .filter(|version| {
                            accepted_version_types
                                .iter()
                                .any(|ver_type| ver_type == &version.version_type)
                                && mod_data
                                    .loaders
                                    .iter()
                                    .all(|loader| version.loaders.contains(&loader.to_string().to_lowercase()))
                                && !version.files.is_empty()
                        })
                        .collect();

                    if !filtered_list.is_empty() {
                        {
                            if let Some(version_id) = &modrinth_data.version_id {
                                // If the version being checked has the same ID as our local copy, it means it is already on the latest possible version
                                if &filtered_list[0].id != version_id {
                                    modrinth_data.cdn_file = Some(filtered_list[0].files[0].clone().into());
                                    mod_data.state = FileState::Outdated;
                                }
                            } else {
                                // If an ID isn't provided assume its outdated
                                modrinth_data.cdn_file = Some(filtered_list[0].files[0].clone().into());
                                mod_data.state = FileState::Outdated;
                            }
                        }
                    }
                };

                Ok::<(), error::Error>(())
            });
        }

        futures::future::try_join_all(handles).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) async fn create_mod_data(
        &self,
        modrinth_id: String,
        game_version: String,
        modloader: ModLoader,
    ) -> LibResult<ModFileData> {
        match self.ferinth.get_project(modrinth_id.as_str()).await {
            Ok(project) => {
                let modrinth = ModrinthData {
                    project_id: project.id,
                    version_id: None,
                    cdn_file: None,
                };

                let sources = Sources {
                    curseforge: None,
                    modrinth: Some(modrinth),
                };

                let loaders = vec![modloader];

                // Create an entry from whatever data is available
                let mut mod_file = ModFileData {
                    state: FileState::Current,
                    sourced_from: CurrentSource::Modrinth,
                    loaders,
                    sources,
                };

                self.check_for_updates(vec![&mut mod_file], &game_version).await?;

                Ok(mod_file)
            }
            Err(_err) => Err(error::Error::NotValidModrinthId),
        }
    }

    #[instrument(skip(self))]
    async fn list_versions(
        &self,
        modrinth_id: &str,
        loaders: &Vec<ModLoader>,
        game_versions: &[&str],
    ) -> LibResult<Vec<ferinth::structures::version_structs::Version>> {
        Ok(self
            .ferinth
            .list_versions_filtered(
                modrinth_id,
                Some(
                    loaders
                        .iter()
                        .map(|loader| loader.as_str())
                        .collect::<Vec<&str>>()
                        .as_slice(),
                ),
                Some(game_versions),
                None,
            )
            .await?)
    }

    pub(crate) async fn set_modrinth_data(&self, data: HashMap<&Hashes, &mut ModFileData>) -> LibResult<()> {
        let mut valid_data: HashMap<&Hashes, &mut ModFileData> =
            data.into_iter().filter(|e| e.1.sources.modrinth.is_none()).collect();

        let hashes: Vec<String> = valid_data.iter().map(|e| e.0.sha1.clone()).collect();
        let versions = self.ferinth.get_versions_from_hashes(hashes).await?;

        for (sha1, ver_data) in versions {
            let entries = valid_data.iter_mut().filter(|e| e.0.sha1 == sha1);
            for entry in entries {
                entry.1.sources.modrinth = Some(ModrinthData {
                    project_id: ver_data.project_id.clone(),
                    version_id: Some(ver_data.id.clone()),
                    cdn_file: None,
                });

                if entry.1.sourced_from == CurrentSource::None {
                    entry.1.sourced_from = CurrentSource::Modrinth;
                };
            }
        }

        Ok(())
    }
}

fn accepted_versions_vec() -> Vec<VersionType> {
    let min_ver = CONF.lock().modrinth_version_type;
    let ver_arr = [VersionType::Release, VersionType::Beta, VersionType::Alpha];
    let mut allowed_versions = Vec::new();
    for version in ver_arr {
        allowed_versions.push(version);
    }
    allowed_versions.push(min_ver.into());
    allowed_versions
}
