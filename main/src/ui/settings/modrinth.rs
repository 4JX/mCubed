use back::{
    settings::{SettingsBuilder, CONF},
    VersionType,
};
use eframe::egui::{ComboBox, Layout, Ui};

use crate::ui::{misc, IMAGES, THEME};

use super::SettingsSection;

pub(super) struct ModrinthSettings;

impl ModrinthSettings {
    pub(super) fn show(ui: &mut Ui) {
        SettingsSection::with_id("modrinth").settings_section(
            ui,
            |ui, col_state| {
                ui.horizontal(|ui| {
                    ui.image(
                        IMAGES.lock().modrinth.as_ref().unwrap(),
                        THEME.image_size.settings_heading,
                    );
                    ui.heading("Modrinth");
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        col_state.show_toggle_button(ui, misc::collapsing_state_icon_fn);
                    });
                });
            },
            |ui| {
                ui.label("Base release type");

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
            },
        )
    }
}
