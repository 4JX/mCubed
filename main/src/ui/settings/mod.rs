use std::hash::Hash;

use eframe::egui::{
    collapsing_header::{self, CollapsingState},
    style::Margin,
    Frame, Id, Sense, Ui,
};

use self::modrinth::ModrinthSettings;

use super::THEME;

mod modrinth;
pub struct SettingsUi;

impl SettingsUi {
    pub fn show(ui: &mut Ui) {
        ModrinthSettings::show(ui);
    }
}

struct SettingsSection {
    id: Id,
}

impl SettingsSection {
    fn with_id(id: impl Hash) -> Self {
        Self {
            id: Id::new("settings_section").with(id),
        }
    }

    fn settings_section(
        &self,
        ui: &mut Ui,
        header: impl FnOnce(&mut Ui, &mut CollapsingState),
        body: impl FnOnce(&mut Ui),
    ) {
        let mut state = collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(self.id),
            false,
        );

        let header_res = Frame {
            fill: THEME.colors.darker_gray,
            inner_margin: Margin::same(6.0),
            rounding: THEME.rounding.big,
            ..Frame::default()
        }
        .show(ui, |ui| header(ui, &mut state));

        let interact = ui.interact(
            header_res.response.rect,
            ui.id().with(self.id).with("interact"),
            Sense::click(),
        );

        if interact.clicked() {
            state.toggle(ui);
        }

        state.show_body_indented(&header_res.response, ui, |ui| body(ui));
    }
}
