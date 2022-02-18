use daedalus::minecraft::VersionManifest;

use crate::{
    error,
    mod_entry::{ModEntry, ModLoader},
};

pub enum ToBackend {
    ScanFolder,

    CheckForUpdates {
        game_version: String,
    },

    GetVersionMetadata,

    UpdateMod {
        mod_entry: ModEntry,
    },

    AddMod {
        modrinth_id: String,
        game_version: String,
        modloader: ModLoader,
    },
}

pub enum ToFrontend {
    SetVersionMetadata { manifest: VersionManifest },

    UpdateModList { mod_list: Vec<ModEntry> },

    CheckForUpdatesProgress { progress: CheckProgress },

    BackendError { error: BackendError },
}

pub struct CheckProgress {
    /// The name of the project
    pub name: String,

    /// What position is it in
    pub position: usize,

    /// The total amount of projects being fetched
    pub total_len: usize,
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
