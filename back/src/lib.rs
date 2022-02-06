use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use hash::Hashes;
use messages::{CheckProgress, ToBackend, ToFrontend};
use mod_entry::ModEntry;
use modrinth::Modrinth;
use tokio::runtime::Runtime;
mod modrinth;

mod hash;
pub mod messages;
pub mod mod_entry;

pub use daedalus::minecraft::Version as GameVersion;

pub struct Back {
    mod_list: Vec<ModEntry>,
    folder_path: PathBuf,
    rt: Runtime,
    modrinth: Modrinth,
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
}

impl Back {
    pub fn new(
        folder_path: PathBuf,
        back_tx: Sender<ToFrontend>,
        front_rx: Receiver<ToBackend>,
    ) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();

        Self {
            mod_list: Default::default(),
            folder_path,
            rt,
            modrinth: Default::default(),
            back_tx,
            front_rx,
        }
    }

    pub fn init(&mut self) {
        self.rt.block_on(async {
            loop {
                match self.front_rx.recv() {
                    Ok(message) => match message {
                        ToBackend::CheckForUpdates { game_version } => {
                            let total_len = self.mod_list.len();
                            for (position, mod_entry) in self.mod_list.iter_mut().enumerate() {
                                // Update the frontend on whats happening
                                self.back_tx
                                    .send(ToFrontend::CheckForUpdatesProgress {
                                        progress: CheckProgress {
                                            name: mod_entry.display_name.clone(),
                                            position,
                                            total_len,
                                        },
                                    })
                                    .unwrap();

                                self.modrinth
                                    .check_for_updates(&game_version, mod_entry)
                                    .await;
                            }

                            self.back_tx
                                .send(ToFrontend::UpdateModList {
                                    mod_list: self.mod_list.clone(),
                                })
                                .unwrap();
                        }

                        ToBackend::ScanFolder => {
                            self.mod_list.clear();

                            let read_dir = fs::read_dir(&self.folder_path).unwrap();

                            for file_entry in read_dir {
                                let path = file_entry.unwrap().path();

                                // Minecraft does not really care about mods within folders, therefore skip anything that is not a file
                                if path.is_file() {
                                    let mut file = fs::File::open(&path).unwrap();

                                    self.mod_list.append(&mut ModEntry::from_file(&mut file));
                                }
                            }

                            self.back_tx
                                .send(ToFrontend::UpdateModList {
                                    mod_list: self.mod_list.clone(),
                                })
                                .unwrap();
                        }

                        ToBackend::GetVersionMetadata => {
                            let manifest = daedalus::minecraft::fetch_version_manifest(None)
                                .await
                                .unwrap();

                            self.back_tx
                                .send(ToFrontend::SetVersionMetadata {
                                    version_list: manifest.versions,
                                })
                                .unwrap();
                        }

                        ToBackend::UpdateMod { mod_entry } => {
                            if let Some(bytes) = self.modrinth.update_mod(&mod_entry).await {
                                let read_dir = fs::read_dir(&self.folder_path).unwrap();

                                'file_loop: for file_entry in read_dir {
                                    let path = file_entry.unwrap().path();

                                    if path.is_file() {
                                        let mut file = fs::File::open(&path).unwrap();

                                        let hashes = Hashes::get_hashes_from_file(&mut file);

                                        // We found the file the mod_entry belongs to
                                        if mod_entry.hashes.sha1 == hashes.sha1 {
                                            std::fs::remove_file(path).unwrap();

                                            // The data is guaranteed to exist, unwrapping here is fine
                                            let path = self.folder_path.join(format!(
                                                "{}-{}.jar",
                                                mod_entry.id,
                                                mod_entry
                                                    .modrinth_data
                                                    .as_ref()
                                                    .unwrap()
                                                    .latest_valid_version
                                                    .as_ref()
                                                    .unwrap()
                                                    .filename
                                            ));

                                            // Essentially fs::File::create(path) but with read access as well
                                            let mut new_mod_file = OpenOptions::new()
                                                .read(true)
                                                .write(true)
                                                .create(true)
                                                .truncate(true)
                                                .open(path)
                                                .unwrap();

                                            new_mod_file.write_all(&bytes).unwrap();

                                            let mut new_entries =
                                                ModEntry::from_file(&mut new_mod_file);

                                            for new_mod_entry in new_entries.iter_mut() {
                                                // Ensure the data for the entry is kept
                                                new_mod_entry.modrinth_data =
                                                    mod_entry.modrinth_data.clone();
                                                new_mod_entry.sourced_from =
                                                    mod_entry.sourced_from.clone();

                                                for list_entry in self.mod_list.iter_mut() {
                                                    if list_entry.hashes.sha1
                                                        == mod_entry.hashes.sha1
                                                        && list_entry.id == new_mod_entry.id.clone()
                                                    {
                                                        *list_entry = new_mod_entry.clone();
                                                    }
                                                }
                                            }

                                            self.back_tx
                                                .send(ToFrontend::UpdateModList {
                                                    mod_list: self.mod_list.clone(),
                                                })
                                                .unwrap();

                                            break 'file_loop;
                                        }
                                    }
                                }
                            };
                        }
                    },
                    Err(err) => {
                        let _ = err;
                    }
                };
            }
        });
    }
}
