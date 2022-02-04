use crate::mod_entry::ModEntry;

pub enum ToBackend {
    ScanFolder,
    CheckForUpdates { mod_list: Vec<ModEntry> },
}

pub enum ToFrontend {
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
