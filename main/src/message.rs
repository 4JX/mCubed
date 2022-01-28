use crate::mod_entry::ModEntry;

pub enum Message {
    UpdatedModList { list: Vec<ModEntry> },
    FetchingMod { context: FetchingModContext },
}

pub struct FetchingModContext {
    // The name of the mod
    pub name: String,

    // What position is it in
    pub position: usize,

    // The total amount of mods being fetched
    pub total: usize,
}
