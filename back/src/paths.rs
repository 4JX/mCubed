use std::path::PathBuf;
use tracing::instrument;

lazy_static::lazy_static! {
    static ref BASE_DIRS: directories::BaseDirs = directories::BaseDirs::new().expect("Could not get base dirs");
    static ref HOME_DIR: std::path::PathBuf = BASE_DIRS.home_dir().to_owned();
    pub static ref CONFIG_DIR: std::path::PathBuf = BASE_DIRS.config_dir().join("mCubed");
}

#[cfg(target_os = "windows")]
#[instrument(level = "trace")]
pub fn default_mod_dir() -> PathBuf {
    HOME_DIR
        .join("AppData")
        .join("Roaming")
        .join(".minecraft")
        .join("mods")
}

#[cfg(target_os = "linux")]
#[instrument(level = "trace")]
pub fn default_mod_dir() -> PathBuf {
    HOME_DIR.join(".minecraft").join("mods")
}

#[cfg(target_os = "macos")]
#[instrument(level = "trace")]
pub fn default_mod_dir() -> PathBuf {
    HOME_DIR
        .join("Library")
        .join("ApplicationSupport")
        .join("minecraft")
        .join("mods")
}
