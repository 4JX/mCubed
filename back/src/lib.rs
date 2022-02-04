use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use messages::{CheckProgress, ToBackend, ToFrontend};
use mod_entry::ModEntry;
use modrinth::Modrinth;
use tokio::runtime::Runtime;
mod modrinth;

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

                            for entry in read_dir {
                                let path = entry.unwrap().path();

                                // Minecraft does not really care about mods within folders, therefore skip anything that is not a file
                                if path.is_file() {
                                    let file = fs::File::open(&path).unwrap();

                                    self.mod_list.append(&mut ModEntry::from_file(file));
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
                    },
                    Err(err) => {
                        let _ = err;
                    }
                };
            }
        });
    }
}
