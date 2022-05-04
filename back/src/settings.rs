use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
    pub static ref CONF: Arc<Mutex<SettingsBuilder>> =
        Arc::new(Mutex::new(SettingsBuilder::default()));
}

pub struct SettingsBuilder {
    /// The size of the images the icon of a mod will be resized to
    pub icon_resize_size: u32,
}

impl SettingsBuilder {
    /// Create a new [SettingsBuilder](SettingsBuilder) off of the default struct values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the icon resize size
    #[must_use]
    pub const fn icon_resize_size(mut self, size: u32) -> Self {
        self.icon_resize_size = size;
        self
    }

    /// Apply the configuration
    pub fn apply(self) {
        let mut changer = CONF.lock();
        *changer = self;
    }
}

impl Default for SettingsBuilder {
    fn default() -> Self {
        Self {
            icon_resize_size: 128,
        }
    }
}
