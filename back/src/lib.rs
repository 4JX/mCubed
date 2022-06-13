use std::{collections::HashMap, fmt::Debug, fs, path::Path, sync::Arc};

use crossbeam_channel::{Receiver, Sender};
use futures::future;
use messages::{ToBackend, ToFrontend};
use mod_file::{FileState, Hashes, ModFile, ModFileData, ModLoader};
use modrinth::Modrinth;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, Once};
use persistence::cache::CacheStorage;
use structs::CdnFile;
use tracing::{debug, error, info, instrument};

use crate::{messages::BackendError, settings::CONF};

mod error;
pub mod messages;
pub mod mod_file;
mod modrinth;
mod paths;
mod persistence;
mod structs;

pub use daedalus::minecraft::Version as GameVersion;
pub use persistence::settings;

static LOG_CHANNEL_CLOSED: Once = Once::new();
static MODRINTH: Lazy<Modrinth> = Lazy::new(Modrinth::default);

pub struct Back {
    mod_list: Vec<ModFile>,
    cache: CacheStorage,
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
    egui_context: eframe::egui::Context,
}

impl Debug for Back {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Back")
            .field("mod_list", &self.mod_list)
            .field("cache", &self.cache)
            .field("back_tx", &self.back_tx)
            .field("front_rx", &self.front_rx)
            // .field("egui_context", &self.egui_context)
            .finish()
    }
}

impl Back {
    #[instrument(skip(egui_context), level = "trace")]
    pub fn new(
        back_tx: Sender<ToFrontend>,
        front_rx: Receiver<ToBackend>,
        egui_context: eframe::egui::Context,
    ) -> Self {
        Self {
            mod_list: Vec::default(),
            cache: CacheStorage::default(),
            back_tx,
            front_rx,
            egui_context,
        }
    }

    #[instrument(skip(self))]
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

                                self.send_list();

                                self.get_version_metadata().await;
                            }

                            ToBackend::Shutdown => {
                                if let Err(error) = CONF.lock().save_config() {
                                    error!(%error, "Could not save config");
                                }

                                self.save_list_cache();
                                break;
                            }

                            ToBackend::ScanFolder => {
                                self.scan_folder();

                                self.send_list();
                            }

                            ToBackend::UpdateBackendList { mod_list } => {
                                self.mod_list = mod_list;
                            }

                            ToBackend::CheckForUpdates { game_version } => {
                                self.scan_folder();

                                self.update_file_data().await;

                                self.check_for_updates(game_version).await;

                                self.send_list();
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

                                self.scan_folder();

                                self.send_list();
                            }

                            ToBackend::UpdateAll => {
                                self.update_all_mods().await;

                                self.scan_folder();

                                self.send_list();
                            }

                            ToBackend::UpdateMod { mod_file } => {
                                update_mod(&*mod_file).await;

                                self.scan_folder();

                                self.send_list();
                            }

                            ToBackend::DeleteMod { path } => {
                                self.delete_mod(&path);
                            }
                        }
                        self.egui_context.request_repaint();
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

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    fn save_list_cache(&mut self) {
        let mut mod_list_clone = self.mod_list.clone();

        // Transfer the data for existing entries
        Self::transfer_list_data(&mod_list_clone, self.cache.get_cache_mut(), false);

        let current_cache = self.cache.get_cache();

        // Append the mods that did not exist before
        mod_list_clone.retain(|mod_file| {
            // Check for unique entries by hash
            !current_cache
                .iter()
                .any(|cache_entry| cache_entry.hashes.sha1 == mod_file.hashes.sha1)
        });

        self.cache.get_cache_mut().append(&mut mod_list_clone);

        // self.cache.set_cache(self.mod_list.clone());

        if let Err(error) = self.cache.save_list_cache() {
            error!(%error, "Could not save cache");

            self.back_tx
                .send(ToFrontend::BackendError {
                    error: BackendError::new(format!("Could not save cache: {}", error), error),
                })
                .unwrap();
        }
    }

    #[instrument(skip(self))]
    fn send_list(&mut self) {
        info!(length = self.mod_list.len(), "Sending the mods list");

        self.back_tx
            .send(ToFrontend::UpdateModList {
                mod_list: self.mod_list.clone(),
            })
            .unwrap();
    }

    #[instrument(skip(self))]
    fn scan_folder(&mut self) {
        let mod_folder_path = CONF.lock().mod_folder_path.clone();
        info!(folder_path = %mod_folder_path.display(), "Scanning the mods folder");

        let old_list = self.mod_list.clone();
        self.mod_list.clear();

        let read_dir = fs::read_dir(&mod_folder_path).unwrap();

        'file_loop: for file_entry in read_dir {
            let path = file_entry.unwrap().path();

            if is_relevant_file(&path) {
                debug!(?path, "Parsing file");

                match ModFile::from_path(path.clone()) {
                    Ok(entry) => self.mod_list.push(entry),
                    Err(error) => {
                        // In the case of an error the mod list will be cleared
                        self.mod_list.clear();

                        error!(path = %path.display(), "Could not parse mod");

                        self.back_tx
                            .send(ToFrontend::BackendError {
                                error: BackendError::new(format!("Could not parse: {}", path.display()), error),
                            })
                            .unwrap();
                        break 'file_loop;
                    }
                }
            }
        }

        self.transfer_list_data_to_current(&old_list);
    }

    #[instrument(skip(self))]
    async fn check_for_updates(&mut self, game_version: String) {
        let back_tx = &self.back_tx;
        let data_mut: Vec<&mut ModFileData> = self.mod_list.iter_mut().map(|file| &mut file.data).collect();

        if let Err(error) = MODRINTH.check_for_updates(data_mut, &game_version).await {
            error!("Failed to check for updates");

            back_tx
                .send(ToFrontend::BackendError {
                    error: BackendError::new("Failed to check for updates", error),
                })
                .unwrap();
        };
    }

    #[instrument(skip(self))]
    async fn add_mod(&mut self, modrinth_id: String, game_version: String, modloader: ModLoader) {
        match MODRINTH
            .create_mod_data(modrinth_id.clone(), game_version, modloader)
            .await
        {
            Ok(mod_data) => {
                // create_mod_file(&mod_data, &bytes);
                dbg!(&mod_data.sources.modrinth);
            }
            Err(error) => {
                error!(%modrinth_id, "Could not add mod");

                self.back_tx
                    .send(ToFrontend::BackendError {
                        error: BackendError::new(format!("Could not add mod: {}", modrinth_id), error),
                    })
                    .unwrap();
            }
        };
    }

    #[instrument(skip(self))]
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
            self.mod_list.retain(|mod_file| mod_file.path != path);

            debug!("File deleted successfully");

            self.send_list();
        };
    }

    #[instrument(skip(self))]
    async fn get_version_metadata(&self) {
        match daedalus::minecraft::fetch_version_manifest(None).await {
            Ok(manifest) => self.back_tx.send(ToFrontend::SetVersionMetadata { manifest }).unwrap(),
            Err(error) => {
                error!("There was an error getting the version metadata");
                self.back_tx
                    .send(ToFrontend::BackendError {
                        error: BackendError::new("There was an error getting the version metadata", error),
                    })
                    .unwrap();
            }
        };
    }

    #[instrument(skip(self, from_list), fields(length_from = from_list.len(), length_to = self.mod_list.len()))]
    fn transfer_list_data_to_current(&mut self, from_list: &[ModFile]) {
        Self::transfer_list_data(from_list, &mut self.mod_list, true);
    }

    #[instrument(skip(from_list, to_list), fields(length_from = from_list.len(), length_to = to_list.len()))]
    fn transfer_list_data(from_list: &[ModFile], to_list: &mut Vec<ModFile>, keep_state: bool) {
        // Ensures the important bits are kept
        for mod_file in to_list {
            let filtered_old: Vec<&ModFile> = from_list
                .iter()
                .filter(|m_file| m_file.hashes.sha1 == mod_file.hashes.sha1)
                .collect();

            if !filtered_old.is_empty() {
                mod_file.data.sourced_from = filtered_old[0].data.sourced_from;
                mod_file.data.sources = filtered_old[0].data.sources.clone();

                if keep_state {
                    mod_file.data.state = filtered_old[0].data.state;
                } else {
                    mod_file.data.state = FileState::Current;
                }
            }
        }
    }

    async fn update_all_mods(&mut self) {
        let mod_list_m = self.mod_list.iter_mut().map(|file| Arc::new(Mutex::new(file)));

        let mut handles = Vec::new();

        for file in mod_list_m {
            handles.push(async move {
                let mut file = file.lock();
                let new_file = update_mod(&file).await;
                if let Some(new) = new_file {
                    // There's probably a better way to do this
                    file.entries = new.entries;
                    file.data = new.data;
                    file.hashes = new.hashes;
                    file.path = new.path;
                }
            })
        }

        future::join_all(handles).await;
    }

    async fn update_file_data(&mut self) {
        let map: HashMap<&Hashes, &mut ModFileData> = self
            .mod_list
            .iter_mut()
            .map(|file| (&file.hashes, &mut file.data))
            .collect();

        if let Err(error) = MODRINTH.set_modrinth_data(map).await {
            error!("Failed to check for set Modrinth data");

            self.back_tx
                .send(ToFrontend::BackendError {
                    error: BackendError::new("Failed to set Modrinth data", error),
                })
                .unwrap();
        };
    }
}

#[instrument(skip(mod_file))]
async fn update_mod(mod_file: &ModFile) -> Option<ModFile> {
    info!(
        path = ?mod_file.path,
        sha1 = %mod_file.hashes.sha1,
        "Updating mod"
    );

    let read_dir = fs::read_dir(&CONF.lock().mod_folder_path).unwrap();

    for file_entry in read_dir {
        let path = file_entry.unwrap().path();

        if is_relevant_file(&path) {
            let mut file = fs::File::open(&path).unwrap();

            let hashes = Hashes::get_hashes_from_file(&mut file).unwrap();

            // We found the file the mod_file belongs to
            if mod_file.hashes.sha1 == hashes.sha1 {
                std::fs::remove_file(path).unwrap();

                return create_mod_file(&mod_file.data).await;
            }
        }
    }

    None
}

#[instrument(level = "trace")]
fn is_relevant_file(path: &Path) -> bool {
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

#[instrument(skip(file_data))]
async fn create_mod_file(file_data: &ModFileData) -> Option<ModFile> {
    // The data is guaranteed to exist, unwrapping here is fine
    let cdn_file = get_cdn_file(file_data);

    if let Some(cdn_file) = cdn_file {
        let folder = CONF.lock().mod_folder_path.clone();
        let full_path = folder.join(&cdn_file.filename);

        info!("Creating file {}", cdn_file.filename);

        cdn_file.download(folder).await;

        let mut new_file = ModFile::from_path(full_path).unwrap();

        // Ensure the data for the entry is kept
        new_file.data.sources = file_data.sources.clone();
        new_file.data.sourced_from = file_data.sourced_from;

        return Some(new_file);
    };

    None
}

fn get_cdn_file(file_data: &ModFileData) -> Option<CdnFile> {
    match file_data.sourced_from {
        mod_file::CurrentSource::Modrinth => {
            if let Some(modrinth_data) = &file_data.sources.modrinth {
                if let Some(ref file) = modrinth_data.cdn_file {
                    return Some(file.clone());
                }
            }
        }
        _ => unreachable!("Attempted to get CDN file via invalid route {}", file_data.sourced_from),
    }

    None
}
