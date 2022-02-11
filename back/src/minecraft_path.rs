use std::path::PathBuf;

lazy_static::lazy_static! {
    pub static ref HOME_DIR: std::path::PathBuf = directories::BaseDirs::new().unwrap().home_dir().to_owned();
}

#[cfg(target_os = "windows")]
pub fn default_mod_dir() -> PathBuf {
    HOME_DIR
        .join("AppData")
        .join("Roaming")
        .join(".minecraft")
        .join("mods")
}

#[cfg(target_os = "linux")]
pub fn default_mod_dir() -> PathBuf {
    HOME_DIR.join(".minecraft").join("mods")
}

#[cfg(target_os = "mac_os")]
pub fn default_mod_dir() -> PathBuf {
    HOME_DIR
        .join("Library")
        .join("ApplicationSupport")
        .join("minecraft")
        .join("mods")
}
