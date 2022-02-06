use daedalus::minecraft::Version;

use crate::mod_entry::ModEntry;

pub enum ToBackend {
    ScanFolder,

    CheckForUpdates { game_version: String },

    GetVersionMetadata,

    UpdateMod { mod_entry: ModEntry },
}

pub enum ToFrontend {
    SetVersionMetadata { version_list: Vec<Version> },

    UpdateModList { mod_list: Vec<ModEntry> },

    CheckForUpdatesProgress { progress: CheckProgress },
}

pub struct CheckProgress {
    // The name of the project
    pub name: String,

    // What position is it in
    pub position: usize,

    // The total amount of projects being fetched
    pub total_len: usize,
}
