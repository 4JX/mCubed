use back::{
    settings::{SettingsBuilder, CONF},
    VersionType,
};
use eframe::egui::{ComboBox, Ui};

use crate::ui::{misc, IMAGES};

use super::SettingsSection;

pub(super) struct ModrinthSettings;

impl SettingsSection for ModrinthSettings {
    const ID: &'static str = "modrinth";

    fn show(ui: &mut Ui) {
        Self::settings_section(ui, &IMAGES.lock().modrinth, "Modrinth", |ui| {
            ui.label("Base release type").on_hover_text("This indicates the minimum level of stability a version should be marked with to appear when update-checking");

            let current = CONF.lock().modrinth_version_type;

            ComboBox::from_id_source(ui.id().with("version-type"))
                .icon(misc::combobox_icon_fn)
                .selected_text(format!("{:?}", current))
                .show_ui(ui, |ui| {
                    let release_res =
                        ui.selectable_label(current == VersionType::Release, "Release");
                    let beta_res = ui.selectable_label(current == VersionType::Beta, "Beta");
                    let alpha_res = ui.selectable_label(current == VersionType::Alpha, "Alpha");
                    let builder = SettingsBuilder::from_current();
                    if release_res.clicked() {
                        builder.modrinth_version_type(VersionType::Release).apply()
                    } else if beta_res.clicked() {
                        builder.modrinth_version_type(VersionType::Beta).apply()
                    } else if alpha_res.clicked() {
                        builder.modrinth_version_type(VersionType::Alpha).apply()
                    }
                });
        })
    }
}
