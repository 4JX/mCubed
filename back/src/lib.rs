use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use hash::Hashes;
use messages::{CheckProgress, ToBackend, ToFrontend};
use mod_entry::{ModEntry, ModrinthData, Source};
use modrinth::Modrinth;
mod modrinth;

mod error;
mod hash;
pub mod messages;
pub mod mod_entry;

pub use daedalus::minecraft::Version as GameVersion;

pub struct Back {
    mod_list: Vec<ModEntry>,
    folder_path: PathBuf,
    modrinth: Modrinth,
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
    egui_epi_frame: Option<epi::Frame>,
}

impl Back {
    pub fn new(
        folder_path: PathBuf,
        back_tx: Sender<ToFrontend>,
        front_rx: Receiver<ToBackend>,
        egui_epi_frame: Option<epi::Frame>,
    ) -> Self {
        Self {
            mod_list: Default::default(),
            folder_path,
            modrinth: Default::default(),
            back_tx,
            front_rx,
            egui_epi_frame,
        }
    }

    fn scan_folder(&mut self) {
        self.mod_list.clear();

        let read_dir = fs::read_dir(&self.folder_path).unwrap();

        'file_loop: for file_entry in read_dir {
            let path = file_entry.unwrap().path();

            // Minecraft does not really care about mods within folders, therefore skip anything that is not a file
            if path.is_file() {
                let mut file = fs::File::open(&path).unwrap();

                match ModEntry::from_file(&mut file) {
                    Ok(mut entry) => self.mod_list.append(&mut entry),
                    Err(error) => {
                        // In the case of an error the mod list will be cleared
                        self.mod_list.clear();
                        self.back_tx
                            .send(ToFrontend::BackendError { error })
                            .unwrap();
                        break 'file_loop;
                    }
                }
            }
        }

        self.back_tx
            .send(ToFrontend::UpdateModList {
                mod_list: self.mod_list.clone(),
            })
            .unwrap();
    }

    pub fn init(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            loop {
                match self.front_rx.recv() {
                    Ok(message) => {
                        match message {
                            ToBackend::ScanFolder => {
                                self.scan_folder();
                            }

                            ToBackend::CheckForUpdates { game_version } => {
                                self.scan_folder();

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

                                    if let Some(frame) = &self.egui_epi_frame {
                                        frame.request_repaint();
                                    }

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

                            ToBackend::GetVersionMetadata => {
                                match daedalus::minecraft::fetch_version_manifest(None).await {
                                    Ok(manifest) => self
                                        .back_tx
                                        .send(ToFrontend::SetVersionMetadata { manifest })
                                        .unwrap(),
                                    Err(err) => self
                                        .back_tx
                                        .send(ToFrontend::BackendError { error: err.into() })
                                        .unwrap(),
                                };
                            }

                            ToBackend::UpdateMod { mod_entry } => {
                                if let Ok(bytes) = self.modrinth.update_mod(&mod_entry).await {
                                    let read_dir = fs::read_dir(&self.folder_path).unwrap();

                                    'file_loop: for file_entry in read_dir {
                                        let path = file_entry.unwrap().path();

                                        if path.is_file() {
                                            let mut file = fs::File::open(&path).unwrap();

                                            let hashes =
                                                Hashes::get_hashes_from_file(&mut file).unwrap();

                                            // We found the file the mod_entry belongs to
                                            if mod_entry.hashes.sha1 == hashes.sha1 {
                                                std::fs::remove_file(path).unwrap();

                                                // The data is guaranteed to exist, unwrapping here is fine
                                                let path = self.folder_path.join(format!(
                                                    "{}-{}",
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
                                                    ModEntry::from_file(&mut new_mod_file).unwrap();

                                                for new_mod_entry in new_entries.iter_mut() {
                                                    // Ensure the data for the entry is kept
                                                    new_mod_entry.modrinth_data =
                                                        mod_entry.modrinth_data.clone();
                                                    new_mod_entry.sourced_from =
                                                        mod_entry.sourced_from;

                                                    for list_entry in self.mod_list.iter_mut() {
                                                        if list_entry.hashes.sha1
                                                            == mod_entry.hashes.sha1
                                                            && list_entry.id
                                                                == new_mod_entry.id.clone()
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

                            ToBackend::AddMod {
                                modrinth_id,
                                game_version,
                                modloader,
                            } => {
                                match self
                                    .modrinth
                                    .normalize_modrinth_id(modrinth_id.as_str())
                                    .await
                                {
                                    Some(modrinth_id) => {
                                        // Ensure the entry does not already exist
                                        let mut entry_exists = false;

                                        for mod_entry in &self.mod_list {
                                            if let Some(modrinth_data) = &mod_entry.modrinth_data {
                                                if modrinth_data.id == modrinth_id {
                                                    entry_exists = true
                                                };
                                            } else if let Some(fetched_id) = self
                                                .modrinth
                                                .get_modrinth_id_from_hash(
                                                    mod_entry.hashes.sha1.as_str(),
                                                )
                                                .await
                                            {
                                                if fetched_id == modrinth_id {
                                                    entry_exists = true;
                                                }
                                            };
                                        }

                                        if entry_exists {
                                            // A collision happened, the mod should be updated through standard procedures
                                            self.back_tx
                                                .send(ToFrontend::BackendError {
                                                    error: error::Error::EntryAlreadyInList,
                                                })
                                                .unwrap();
                                        } else {
                                            match self
                                                .modrinth
                                                .get_mod_bytes(
                                                    modrinth_id.clone(),
                                                    game_version.clone(),
                                                    modloader,
                                                )
                                                .await
                                            {
                                                Ok((bytes, filename_details)) => {
                                                    let path = self.folder_path.join(format!(
                                                        "{}-{}",
                                                        &filename_details.project_id,
                                                        &filename_details.file_name
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
                                                        ModEntry::from_file(&mut new_mod_file)
                                                            .unwrap();

                                                    for new_mod_entry in new_entries.iter_mut() {
                                                        // Update the entry information
                                                        let modrinth_data = ModrinthData {
                                                            id: modrinth_id.clone(),
                                                            latest_valid_version: None,
                                                        };
                                                        new_mod_entry.modrinth_data =
                                                            Some(modrinth_data);
                                                        new_mod_entry.sourced_from =
                                                            Source::Modrinth;

                                                        for list_entry in self.mod_list.iter_mut() {
                                                            if list_entry.id
                                                                == new_mod_entry.id.clone()
                                                            {
                                                                *list_entry = new_mod_entry.clone();
                                                            }
                                                        }

                                                        self.mod_list.push(new_mod_entry.clone());
                                                    }

                                                    self.back_tx
                                                        .send(ToFrontend::UpdateModList {
                                                            mod_list: self.mod_list.clone(),
                                                        })
                                                        .unwrap();
                                                }

                                                Err(error) => {
                                                    self.back_tx
                                                        .send(ToFrontend::BackendError { error })
                                                        .unwrap();
                                                }
                                            };
                                        };
                                    }
                                    None => {
                                        self.back_tx
                                            .send(ToFrontend::BackendError {
                                                error: error::Error::NotValidModrinthId,
                                            })
                                            .unwrap();
                                    }
                                };
                            }
                        }

                        if let Some(frame) = &self.egui_epi_frame {
                            frame.request_repaint();
                        }
                    }
                    Err(err) => {
                        let _ = err;
                    }
                };
            }
        });
    }
}
