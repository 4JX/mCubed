use std::collections::HashMap;

use crate::mod_entry::ModEntry;

pub enum ToFrontend {
    // Emitted when starting and ending a list update
    UpdateModList {
        mod_list: Vec<ModEntry>,
        mod_hash_cache: HashMap<String, String>,
    },

    // Gives information about the mod whose information is being fetched
    FetchingMod {
        context: FetchingModContext,
    },
}

pub enum ToBackend {
    // Emitted when starting and ending a list update
    UpdateModList {
        mod_list: Vec<ModEntry>,
        mod_hash_cache: HashMap<String, String>,
    },

    UpdateMod {
        version_id: String,
        modloader: mc_mod_meta::ModLoader,
    },
}

pub struct FetchingModContext {
    // The name of the mod
    pub name: String,

    // What position is it in
    pub position: usize,

    // The total amount of mods being fetched
    pub total: usize,
}
