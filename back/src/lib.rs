use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use bytes::Bytes;
use ferinth::Ferinth;
use messages::ToBackend;
use mod_entry::ModrinthData;
use tokio::runtime::Runtime;

use crate::{
    messages::{FetchingModContext, ToFrontend},
    mod_entry::{FileState, ModEntry, Source},
};

pub mod messages;
pub mod mod_entry;

pub struct Back {
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
    rt: Runtime,
    modrinth: Ferinth,
}

impl Back {
    pub fn new(back_tx: Sender<ToFrontend>, front_rx: Receiver<ToBackend>) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let modrinth = Ferinth::new("Still a test app");

        Self {
            back_tx,
            front_rx,
            rt,
            modrinth,
        }
    }
    pub fn init(&self) {
        self.rt.block_on(async {
            loop {
                match self.front_rx.recv() {
                    Ok(message) => match message {
                        ToBackend::UpdateModList {
                            mod_list,
                            mod_hash_cache,
                        } => {
                            self.update_mod_list(mod_list, mod_hash_cache).await;
                        }

                        ToBackend::UpdateMod {
                            version_id,
                            modloader,
                        } => {
                            let version = self
                                .modrinth
                                .get_version(version_id.as_str())
                                .await
                                .unwrap();

                            let mut file_contents: Option<Bytes> = None;

                            if version.files.len() > 1 {
                                'outer: for file in version.files {
                                    let filename = file.filename.clone();

                                    if filename
                                        .to_lowercase()
                                        .contains(modloader.to_string().to_lowercase().as_str())
                                    {
                                        file_contents = Some(
                                            self.modrinth
                                                .download_version_file(&file)
                                                .await
                                                .unwrap(),
                                        );
                                        break 'outer;
                                    }
                                }
                            } else if version.files.len() == 1 {
                                file_contents = Some(
                                    self.modrinth
                                        .download_version_file(&version.files[0])
                                        .await
                                        .unwrap(),
                                );
                            }

                            let _a = file_contents;
                        }
                    },
                    Err(_err) => {
                        //TODO: Handle
                    }
                }
            }
        });
    }

    async fn update_mod_list(
        &self,
        mut mod_list: Vec<ModEntry>,
        mut mod_hash_cache: HashMap<String, String>,
    ) {
        let list_length = mod_list.len();

        for (position, mod_entry) in mod_list.iter_mut().enumerate() {
            self.back_tx
                .send(ToFrontend::FetchingMod {
                    context: FetchingModContext {
                        name: mod_entry.display_name.clone(),
                        position,
                        total: list_length,
                    },
                })
                .unwrap();

            mod_entry.modrinth_data = if let Some(id) = self
                .get_modrinth_id(&mod_entry.hashes.sha1, &mut mod_hash_cache)
                .await
            {
                mod_entry.sourced_from = Source::Modrinth;

                match self.modrinth.list_versions(id.as_str()).await {
                    Ok(version_data) => {
                        // Assume its outdated unless proven otherwise
                        mod_entry.state = FileState::Outdated;

                        'outer: for file in &version_data[0].files {
                            if let Some(hash) = &file.hashes.sha1 {
                                if hash == &mod_entry.hashes.sha1 {
                                    mod_entry.state = FileState::Current;
                                    break 'outer;
                                }
                            }
                        }

                        Some(ModrinthData {
                            id,
                            lastest_valid_version: version_data[0].id.clone(),
                        })
                    }
                    Err(_err) => {
                        mod_entry.state = FileState::Local;
                        None
                    }
                }
            } else {
                None
            };
        }

        self.back_tx
            .send(ToFrontend::UpdateModList {
                mod_list,
                mod_hash_cache,
            })
            .unwrap();
    }

    async fn get_modrinth_id(
        &self,
        mod_hash: &String,
        mod_hash_cache: &mut HashMap<String, String>,
    ) -> Option<String> {
        if let Some(id) = mod_hash_cache.get(mod_hash) {
            Some(id.clone())
        } else {
            match self
                .modrinth
                .get_version_from_file_hash(mod_hash.as_str())
                .await
            {
                Ok(result) => {
                    mod_hash_cache.insert(mod_hash.clone(), result.mod_id.clone());
                    Some(result.mod_id)
                }
                Err(_err) => None,
            }
        }
    }
}
