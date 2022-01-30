use std::{collections::HashMap, sync::mpsc::Sender, thread};

use ferinth::Ferinth;
use tokio::task::JoinHandle;

use crate::{
    message::{FetchingModContext, Message},
    mod_entry::{FileState, ModEntry, Source},
};

pub mod message;
pub mod mod_entry;

pub fn start_backend(
    tx: Sender<Message>,
    mod_list: Vec<ModEntry>,
    mod_hash_cache: HashMap<String, String>,
) {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            start_backend_a(tx, mod_list, mod_hash_cache).await.unwrap();
        });
    });
}

pub fn start_backend_a(
    tx: Sender<Message>,
    mut mod_list: Vec<ModEntry>,
    mut mod_hash_cache: HashMap<String, String>,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        struct ModManager {
            modrinth: Ferinth,
        }

        impl ModManager {
            async fn fetch_mod_data(
                &self,
                mod_list: &mut Vec<ModEntry>,
                tx: Sender<Message>,
                mod_hash_cache: &mut HashMap<String, String>,
            ) {
                let list_length = mod_list.len();

                for (position, entry) in mod_list.iter_mut().enumerate() {
                    tx.send(Message::FetchingMod {
                        context: FetchingModContext {
                            name: entry.display_name.clone(),
                            position,
                            total: list_length,
                        },
                    })
                    .unwrap();

                    entry.modrinth_id = if let Some(id) = mod_hash_cache.get(&entry.hashes.sha1) {
                        Some(id.to_owned())
                    } else {
                        if let Some(modrinth_id) =
                            self.get_modrinth_id(entry.hashes.sha1.as_str()).await
                        {
                            mod_hash_cache
                                .insert(entry.hashes.sha1.to_owned(), modrinth_id.to_owned());
                            Some(modrinth_id)
                        } else {
                            None
                        }
                    };

                    if let Some(modrinth_id) = &entry.modrinth_id {
                        match self.modrinth.list_versions(modrinth_id.as_str()).await {
                            Ok(version_data) => {
                                entry.sourced_from = Source::Modrinth;
                                // Assume its outdated unless proven otherwise
                                entry.state = FileState::Outdated;

                                'outer: for file in &version_data[0].files {
                                    if let Some(hash) = &file.hashes.sha1 {
                                        if hash == &entry.hashes.sha1 {
                                            entry.state = FileState::Current;
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                dbg!(err);
                                entry.state = FileState::Local
                            }
                        };
                    }
                }
            }

            async fn get_modrinth_id(&self, mod_hash: &str) -> Option<String> {
                match self.modrinth.get_version_from_file_hash(mod_hash).await {
                    Ok(result) => Some(result.mod_id),
                    Err(_err) => None,
                }
            }
        }

        let modrinth = ModManager {
            modrinth: Ferinth::new("Test app"),
        };

        let tx_clone = tx.clone();
        modrinth
            .fetch_mod_data(&mut mod_list, tx_clone, &mut mod_hash_cache)
            .await;

        tx.send(Message::UpdateModList {
            mod_list,
            mod_hash_cache,
        })
        .unwrap();
    })
}
