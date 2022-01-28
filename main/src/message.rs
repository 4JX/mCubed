use crate::mod_entry::ModEntry;

pub enum Message {
    UpdatedModList { list: Vec<ModEntry> },
}
