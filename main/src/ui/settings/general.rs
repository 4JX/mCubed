use back::settings::{SettingsBuilder, CONF};
use eframe::egui::Ui;

use crate::ui::image_utils::ImageTextures;

use super::SettingsSection;

pub(super) struct GeneralSettings;

impl SettingsSection for GeneralSettings {
    const ID: &'static str = "general";

    fn show(ui: &mut Ui, images: &ImageTextures) {
        Self::settings_section(ui, &images.settings, "General", |ui| {
            ui.label("Mods folder path").on_hover_text(
                "The path to the current mods folder of your Minecraft installation",
            );

            if ui
                .button(CONF.lock().mod_folder_path.display().to_string())
                .clicked()
            {
                // This intentionally causes the UI to hang while the dialog is open, so that the user must do something before operations resume
                let folder = rfd::FileDialog::new()
                    .set_title("Choose the mods path")
                    .set_directory(&CONF.lock().mod_folder_path)
                    .pick_folder();

                if let Some(folder) = folder {
                    SettingsBuilder::from_current()
                        .mod_folder_path(folder)
                        .apply();
                }
            }
        });
    }
}
