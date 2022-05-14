use back::settings::{SettingsBuilder, CONF};
use eframe::egui::Ui;

use crate::ui::IMAGES;

use super::SettingsSection;

pub(super) struct GeneralSettings;

impl SettingsSection for GeneralSettings {
    const ID: &'static str = "general";

    fn show(ui: &mut Ui) {
        Self::settings_section(ui, &IMAGES.lock().settings, "General", |ui| {
            ui.label("Mods folder path");

            if ui
                .button(CONF.lock().mod_folder_path.display().to_string())
                .clicked()
            {
                let folder = rfd::FileDialog::new()
                    .set_title("Choose the mods path")
                    .pick_folder();

                if let Some(folder) = folder {
                    SettingsBuilder::from_current()
                        .mod_folder_path(folder)
                        .apply();
                }
            }
        })
    }
}
