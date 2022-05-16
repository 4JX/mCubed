use eframe::{
    egui::{collapsing_header, style::Margin, Frame, Id, Layout, Sense, Ui},
    epaint::TextureHandle,
};
use once_cell::sync::Lazy;

use self::{general::GeneralSettings, modrinth::ModrinthSettings};

use super::{image_utils::ImageTextures, misc, THEME};

mod general;
mod modrinth;

pub struct SettingsUi;

impl SettingsUi {
    pub fn show(ui: &mut Ui, images: &ImageTextures) {
        ui.spacing_mut().item_spacing = THEME.spacing.widget_spacing;
        GeneralSettings::show(ui, images);
        ModrinthSettings::show(ui, images);
    }
}

static SECTION_BASE_ID: Lazy<Id> = Lazy::new(|| Id::new("settings_section"));

pub(super) trait SettingsSection {
    const ID: &'static str;

    fn settings_section(
        ui: &mut Ui,
        header_image: &TextureHandle,
        header_text: &str,
        body: impl FnOnce(&mut Ui),
    ) {
        let id = SECTION_BASE_ID.with(Self::ID);

        let mut state = collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(id),
            false,
        );

        let header_res = Frame {
            fill: THEME.colors.darker_gray,
            inner_margin: Margin::same(6.0),
            rounding: THEME.rounding.big,
            ..Frame::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.image(header_image, THEME.image_size.settings_heading);
                ui.heading(header_text);
                ui.with_layout(Layout::right_to_left(), |ui| {
                    state.show_toggle_button(ui, misc::collapsing_state_icon_fn);
                });
            });
        });

        let interact = ui.interact(
            header_res.response.rect,
            ui.id().with(id).with("interact"),
            Sense::click(),
        );

        if interact.clicked() {
            state.toggle(ui);
        }

        state.show_body_indented(&header_res.response, ui, |ui| body(ui));
    }

    fn show(ui: &mut Ui, images: &ImageTextures);
}
