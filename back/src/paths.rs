use std::path::PathBuf;

use once_cell::sync::Lazy;
use tracing::instrument;

static BASE_DIRS: Lazy<directories::BaseDirs> =
    Lazy::new(|| directories::BaseDirs::new().expect("Could not get base dirs"));
static HOME_DIR: Lazy<PathBuf> = Lazy::new(|| BASE_DIRS.home_dir().to_owned());
pub static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| BASE_DIRS.config_dir().join("mCubed"));

#[cfg(target_os = "windows")]
#[instrument(level = "trace")]
pub fn default_mod_dir() -> PathBuf { HOME_DIR.join("AppData").join("Roaming").join(".minecraft").join("mods") }

#[cfg(target_os = "linux")]
#[instrument(level = "trace")]
pub fn default_mod_dir() -> PathBuf { HOME_DIR.join(".minecraft").join("mods") }

#[cfg(target_os = "macos")]
#[instrument(level = "trace")]
pub fn default_mod_dir() -> PathBuf {
    HOME_DIR
        .join("Library")
        .join("ApplicationSupport")
        .join("minecraft")
        .join("mods")
}
