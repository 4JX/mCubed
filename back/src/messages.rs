use std::path::PathBuf;

use daedalus::minecraft::VersionManifest;

use crate::{
    error,
    mod_file::{ModFile, ModLoader},
};

pub enum ToBackend {
    Startup,

    Shutdown,

    ScanFolder,

    UpdateBackendList {
        mod_list: Vec<ModFile>,
    },

    CheckForUpdates {
        game_version: String,
    },

    GetVersionMetadata,

    AddMod {
        modrinth_id: String,
        game_version: String,
        modloader: ModLoader,
    },

    UpdateAll,

    UpdateMod {
        mod_file: Box<ModFile>,
    },

    DeleteMod {
        path: PathBuf,
    },
}

pub enum ToFrontend {
    SetVersionMetadata { manifest: VersionManifest },

    UpdateModList { mod_list: Vec<ModFile> },

    BackendError { error: BackendError },
}

#[derive(Debug)]
pub struct BackendError {
    /// A short description of the error
    pub message: String,

    /// The related error
    pub error: error::Error,
}

impl BackendError {
    pub fn new(message: impl Into<String>, error: impl Into<error::Error>) -> Self {
        Self {
            message: message.into(),
            error: error.into(),
        }
    }
}
