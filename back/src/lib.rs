use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use ferinth::Ferinth;
use messages::ToBackend;
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
                        _ => unreachable!(),
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

            mod_entry.modrinth_id = if let Some(id) = mod_hash_cache.get(&mod_entry.hashes.sha1) {
                Some(id.to_owned())
            } else {
                if let Some(modrinth_id) =
                    self.get_modrinth_id(mod_entry.hashes.sha1.as_str()).await
                {
                    mod_hash_cache.insert(mod_entry.hashes.sha1.to_owned(), modrinth_id.to_owned());
                    Some(modrinth_id)
                } else {
                    None
                }
            };

            if let Some(modrinth_id) = &mod_entry.modrinth_id {
                match self.modrinth.list_versions(modrinth_id.as_str()).await {
                    Ok(version_data) => {
                        mod_entry.sourced_from = Source::Modrinth;
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
                    }
                    Err(err) => {
                        dbg!(err);
                        mod_entry.state = FileState::Local
                    }
                };
            }
        }

        self.back_tx
            .send(ToFrontend::UpdateModList {
                mod_list,
                mod_hash_cache,
            })
            .unwrap();
    }

    async fn get_modrinth_id(&self, mod_hash: &str) -> Option<String> {
        match self.modrinth.get_version_from_file_hash(mod_hash).await {
            Ok(result) => Some(result.mod_id),
            Err(_err) => None,
        }
    }
}
