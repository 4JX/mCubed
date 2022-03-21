use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
};

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use hash::Hashes;
use messages::{CheckProgress, ToBackend, ToFrontend};
use mod_entry::{ModEntry, ModLoader};
use modrinth::Modrinth;
mod minecraft_path;
mod modrinth;

mod error;
mod hash;
pub mod messages;
pub mod mod_entry;
mod persistence;

pub use daedalus::minecraft::Version as GameVersion;
use parking_lot::Once;
use persistence::cache::CacheStorage;
use tracing::{debug, error, info};

use crate::messages::BackendError;

static LOG_CHANNEL_CLOSED: Once = Once::new();

pub struct Back {
    mod_list: Vec<ModEntry>,
    cache: CacheStorage,
    folder_path: PathBuf,
    modrinth: Modrinth,
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
    egui_context: Option<eframe::egui::Context>,
}

impl Back {
    pub fn new(
        mod_folder_path: Option<PathBuf>,
        back_tx: Sender<ToFrontend>,
        front_rx: Receiver<ToBackend>,
        egui_epi_frame: Option<eframe::egui::Context>,
    ) -> Self {
        let folder_path = mod_folder_path.unwrap_or_else(minecraft_path::default_mod_dir);

        Self {
            mod_list: Vec::default(),
            cache: CacheStorage::new(&folder_path),
            folder_path,
            modrinth: Modrinth::default(),
            back_tx,
            front_rx,
            egui_context: egui_epi_frame,
        }
    }

    pub fn init(&mut self) {
        info!("Initializing backend");

        let rt = tokio::runtime::Runtime::new().unwrap();
        debug!("Runtime created");

        rt.block_on(async {
            loop {
                match self.front_rx.recv() {
                    Ok(message) => {
                        match message {
                            ToBackend::Startup => {
                                self.load_list_cache();

                                self.scan_folder();

                                self.transfer_list_data_to_current(&self.cache.get_cache().clone());

                                self.sort_and_send_list();

                                self.get_version_metadata().await;
                            }

                            ToBackend::Shutdown => {
                                self.save_list_cache();
                                process::exit(0);
                            }

                            ToBackend::ScanFolder => {
                                self.scan_folder();

                                self.sort_and_send_list();
                            }

                            ToBackend::UpdateBackendList { mod_list } => {
                                self.mod_list = mod_list;
                            },


                            ToBackend::CheckForUpdates { game_version } => {
                                self.scan_folder();

                                self.check_for_updates(game_version).await;

                                self.sort_and_send_list();
                            }

                            ToBackend::GetVersionMetadata => {
                               self.get_version_metadata().await;
                            }

                            ToBackend::AddMod {
                                modrinth_id,
                                game_version,
                                modloader,
                            } => {
                                self.add_mod(modrinth_id, game_version, modloader).await;
                            }
                            ToBackend::UpdateMod { mod_entry } => {
                                self.update_mod(*mod_entry).await;
                            }

                            ToBackend::DeleteMod { path } => {
                                self.delete_mod(&path);
                            },
                        }

                        if let Some(context) = &self.egui_context {
                            context.request_repaint();
                        }
                    }
                    Err(error) => {
                        // As the only reason this will error out is if the channel is closed (sender is dropped) a one time log of the error is enough
                       LOG_CHANNEL_CLOSED.call_once(|| {
                           error!(%error, "There was an error when receiving a message from the frontend:");
                       });
                    }
                };
            }
        });
    }

    fn load_list_cache(&mut self) {
        if let Err(error) = self.cache.load_list_cache() {
            error!(%error, "Could not load cache");

            self.back_tx
                .send(ToFrontend::BackendError {
                    error: BackendError::new(format!("Could not load cache: {}", error), error),
                })
                .unwrap();
        }
    }

    fn save_list_cache(&mut self) {
        let mut mod_list_clone = self.mod_list.clone();

        // Transfer the data for existing entries
        Self::transfer_list_data(&mod_list_clone, self.cache.get_cache_mut(), false);

        // Append the mods that did not exist before
        mod_list_clone.retain(|mod_entry| {
            // Check for unique entries by hash
            !self
                .cache
                .get_cache()
                .iter()
                .any(|cache_entry| cache_entry.hashes.sha1 == mod_entry.hashes.sha1)
        });

        self.cache.get_cache_mut().append(&mut mod_list_clone);

        self.cache.set_cache(self.mod_list.clone());

        if let Err(error) = self.cache.save_list_cache() {
            error!(%error, "Could not save cache");

            self.back_tx
                .send(ToFrontend::BackendError {
                    error: BackendError::new(format!("Could not save cache: {}", error), error),
                })
                .unwrap();
        }
    }

    fn sort_and_send_list(&mut self) {
        info!(length = self.mod_list.len(), "Sending the mods list");
        self.mod_list
            .sort_by(|entry_1, entry_2| entry_1.display_name.cmp(&entry_2.display_name));

        self.back_tx
            .send(ToFrontend::UpdateModList {
                mod_list: self.mod_list.clone(),
            })
            .unwrap();
    }

    fn scan_folder(&mut self) {
        info!(folder_path = %self.folder_path.display(), "Scanning the mods folder");

        let old_list = self.mod_list.clone();

        self.mod_list.clear();

        let read_dir = fs::read_dir(&self.folder_path).unwrap();

        'file_loop: for file_entry in read_dir {
            let path = file_entry.unwrap().path();

            if self.is_relevant_file(&path) {
                debug!(?path, "Parsing file");
                let mut file = fs::File::open(&path).unwrap();

                match ModEntry::from_file(&mut file, Some(path.clone())) {
                    Ok(mut entry) => self.mod_list.append(&mut entry),
                    Err(error) => {
                        // In the case of an error the mod list will be cleared
                        self.mod_list.clear();

                        error!(path = %path.display(), "Could not parse mod");

                        self.back_tx
                            .send(ToFrontend::BackendError {
                                error: BackendError::new(
                                    format!("Could not parse: {}", path.display()),
                                    error,
                                ),
                            })
                            .unwrap();
                        break 'file_loop;
                    }
                }
            }
        }

        self.transfer_list_data_to_current(&old_list);
    }

    fn transfer_list_data_to_current(&mut self, from_list: &[ModEntry]) {
        Self::transfer_list_data(from_list, &mut self.mod_list, true);
    }

    fn transfer_list_data(from_list: &[ModEntry], to_list: &mut Vec<ModEntry>, keep_state: bool) {
        // Ensures the important bits are kept
        for mod_entry in to_list {
            let filtered_old: Vec<&ModEntry> = from_list
                .iter()
                .filter(|filter_entry| filter_entry.id == mod_entry.id)
                .collect();

            if !filtered_old.is_empty() {
                mod_entry.sourced_from = filtered_old[0].sourced_from;
                mod_entry.sources.modrinth = filtered_old[0].sources.modrinth.clone();

                // If the file has not changed, the state can also be kept
                if keep_state && mod_entry.hashes.sha1 == filtered_old[0].hashes.sha1 {
                    mod_entry.state = filtered_old[0].state;
                }
            }
        }
    }

    async fn check_for_updates(&mut self, game_version: String) {
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

            if let Some(context) = &self.egui_context {
                context.request_repaint();
            }

            if let Err(error) = self
                .modrinth
                .check_for_updates(mod_entry, &game_version)
                .await
            {
                error!("Failed to check for updates");

                self.back_tx
                    .send(ToFrontend::BackendError {
                        error: BackendError::new("Failed to check for updates", error),
                    })
                    .unwrap();
            };
        }
    }

    async fn update_mod(&mut self, mod_entry: ModEntry) {
        info!(
            entry_name = %mod_entry.display_name,
            path = ?mod_entry.path,
            sha1 = %mod_entry.hashes.sha1,
            "Updating mod"
        );

        if let Ok(bytes) = self.modrinth.update_mod(&mod_entry).await {
            debug!("Update downloaded");
            let read_dir = fs::read_dir(&self.folder_path).unwrap();

            'file_loop: for file_entry in read_dir {
                let path = file_entry.unwrap().path();

                if self.is_relevant_file(&path) {
                    let mut file = fs::File::open(&path).unwrap();

                    let hashes = Hashes::get_hashes_from_file(&mut file).unwrap();

                    // We found the file the mod_entry belongs to
                    if mod_entry.hashes.sha1 == hashes.sha1 {
                        std::fs::remove_file(path).unwrap();

                        self.create_mod_file(&mod_entry, &bytes);
                        break 'file_loop;
                    }
                }
            }
        };
    }

    async fn add_mod(&mut self, modrinth_id: String, game_version: String, modloader: ModLoader) {
        match self
            .modrinth
            .create_mod_entry(modrinth_id.clone(), game_version, modloader)
            .await
        {
            Ok((mod_entry, bytes)) => {
                self.create_mod_file(&mod_entry, &bytes);
            }
            Err(error) => {
                error!(%modrinth_id, "Could not add mod");

                self.back_tx
                    .send(ToFrontend::BackendError {
                        error: BackendError::new(
                            format!("Could not add mod: {}", modrinth_id),
                            error,
                        ),
                    })
                    .unwrap();
            }
        };
    }

    fn create_mod_file(&mut self, mod_entry: &ModEntry, bytes: &Bytes) {
        info!(
            entry_name = %mod_entry.display_name,
            "Creating a new mod file"
        );
        // The data is guaranteed to exist, unwrapping here is fine
        let path = self.folder_path.join(
            &mod_entry
                .sources
                .modrinth
                .as_ref()
                .unwrap()
                .latest_valid_version
                .as_ref()
                .unwrap()
                .filename,
        );

        // Essentially fs::File::create(path) but with read access as well
        let mut new_mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();

        new_mod_file.write_all(bytes).unwrap();

        let mut new_entries = ModEntry::from_file(&mut new_mod_file, Some(path)).unwrap();

        for new_mod_entry in &mut new_entries {
            // Ensure the data for the entry is kept
            new_mod_entry.sources.modrinth = mod_entry.sources.modrinth.clone();
            new_mod_entry.sourced_from = mod_entry.sourced_from;

            for list_entry in &mut self.mod_list {
                // The hash has to be compared to the old entry, the slug/id can be compared to the new one
                if list_entry.hashes.sha1 == mod_entry.hashes.sha1
                    && list_entry.id == new_mod_entry.id.clone()
                {
                    *list_entry = new_mod_entry.clone();
                }
            }
        }

        self.scan_folder();

        self.sort_and_send_list();
    }

    fn delete_mod(&mut self, path: &Path) {
        info!(
            file_path = %path.display(),
            "Deleting file"
        );

        if let Err(error) = trash::delete(&path) {
            error!(
                file_path = %path.display(),
                "Could not delete file"
            );

            self.back_tx
                .send(ToFrontend::BackendError {
                    error: BackendError::new("Failed to delete the file", error),
                })
                .unwrap();
        } else {
            self.mod_list
                .retain(|mod_entry| mod_entry.path.as_ref() != Some(&path.to_path_buf()));

            debug!("File deleted successfully");

            self.sort_and_send_list();
        };
    }

    async fn get_version_metadata(&self) {
        match daedalus::minecraft::fetch_version_manifest(None).await {
            Ok(manifest) => self
                .back_tx
                .send(ToFrontend::SetVersionMetadata { manifest })
                .unwrap(),
            Err(error) => {
                error!("There was an error getting the version metadata");
                self.back_tx
                    .send(ToFrontend::BackendError {
                        error: BackendError::new(
                            "There was an error getting the version metadata",
                            error,
                        ),
                    })
                    .unwrap();
            }
        };
    }

    fn is_relevant_file(&self, path: &Path) -> bool {
        // Minecraft does not really care about mods within folders, therefore skip anything that is not a file
        path.is_file()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .rsplit('.')
                .next()
                .map(|ext| ext.eq_ignore_ascii_case("jar"))
                == Some(true)
    }
}
