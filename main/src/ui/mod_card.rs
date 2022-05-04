use back::{
    messages::ToBackend,
    mod_entry::{CurrentSource, FileState, ModEntry, ModLoader},
};
use crossbeam_channel::Sender;
use eframe::{
    egui::{
        collapsing_header::{self, CollapserPosition},
        style::Margin,
        ComboBox, Context, Frame, Id, ImageButton, Layout, Response, Sense, Ui,
    },
    emath::Vec2,
    epaint::{ColorImage, Rounding, TextureHandle},
};

use super::{
    app_theme::AppTheme, image_utils::ImageTextures, misc, text_utils, ICON_RESIZE_QUALITY,
};

pub struct ModCard {
    mod_entry: ModEntry,
    mod_icon: Option<TextureHandle>,
    extra_details_open: bool,
}

impl ModCard {
    pub fn new(mod_entry: ModEntry, ctx: &Context) -> Self {
        let mod_icon = if let Some(image_raw) = mod_entry.icon.clone() {
            let texture_handle = ctx.load_texture(
                mod_entry.hashes.sha1.clone(),
                ColorImage::from_rgba_unmultiplied(
                    [ICON_RESIZE_QUALITY as usize, ICON_RESIZE_QUALITY as usize],
                    image_raw.as_slice(),
                ),
            );

            Some(texture_handle)
        } else {
            None
        };

        Self {
            mod_entry,
            mod_icon,
            extra_details_open: false,
        }
    }

    pub fn entry(&self) -> &ModEntry {
        &self.mod_entry
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        images: &mut ImageTextures,
        front_tx: &Option<Sender<ToBackend>>,
    ) {
        let mut state = collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id("mod_collapsing_header")
                .with(&self.mod_entry.path),
            false,
        )
        .icon(misc::collapsing_state_icon_fn)
        .collapser_position(CollapserPosition::Invisible);

        state.set_open(self.extra_details_open);

        let state_res = state
            .show_header(ui, |ui| self.render_header(ui, theme, images, front_tx))
            .body(|ui| {
                ui.spacing_mut().item_spacing.y = theme.spacing.small;
                mod_info_text(
                    "Description:",
                    self.mod_entry
                        .description
                        .as_ref()
                        .unwrap_or(&"None".to_string()),
                    ui,
                    theme,
                );

                mod_info_text(
                    "Authors:",
                    self.mod_entry
                        .authors
                        .as_ref()
                        .unwrap_or(&"None".to_string()),
                    ui,
                    theme,
                );

                mod_info_text(
                    "Mod path:",
                    self.mod_entry.path.display().to_string(),
                    ui,
                    theme,
                );
            });

        if state_res.1.inner.clicked() {
            self.extra_details_open = !self.extra_details_open
        }
    }

    pub fn render_header(
        &mut self,
        ui: &mut Ui,
        theme: &AppTheme,
        images: &mut ImageTextures,
        front_tx: &Option<Sender<ToBackend>>,
    ) -> Response {
        let frame_res = Frame {
            fill: theme.colors.dark_gray,
            rounding: Rounding::same(2.0),
            ..Frame::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(36.);

                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                Frame {
                    inner_margin: Margin::symmetric(6.0, 0.0),
                    fill: theme.colors.mod_card.mod_status_icon_background,
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    let image_size = Vec2::splat(12.0);
                    match self.mod_entry.state {
                        FileState::Current => {
                            ui.image(images.mod_status_ok.as_mut().unwrap().id(), image_size)
                        }
                        FileState::Outdated => ui.image(
                            images.mod_status_outdated.as_mut().unwrap().id(),
                            image_size,
                        ),
                        FileState::Invalid => {
                            ui.image(images.mod_status_invalid.as_mut().unwrap().id(), image_size)
                        }
                        FileState::Local => ui.image(
                            // There's not much that can be done here, assume its all good
                            images.mod_status_ok.as_mut().unwrap().id(),
                            image_size,
                        ),
                    };
                });

                ui.add_space(theme.spacing.medium);

                let icon_size = 26.0;

                Frame {
                    rounding: Rounding::same(5.0),
                    fill: theme.colors.mod_card.mod_status_icon_background,
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    ui.set_width(icon_size);
                    ui.set_height(icon_size);
                    if let Some(texture) = &self.mod_icon {
                        ui.image(texture.id(), Vec2::splat(icon_size));
                    }
                });

                Frame {
                    inner_margin: Margin::symmetric(10.0, 0.0),
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    ui.set_width(120.);
                    ui.set_max_width(120.);

                    ui.centered_and_justified(|ui| {
                        ui.style_mut().wrap = Some(true);
                        ui.label(text_utils::mod_name_job(ui, &self.mod_entry.display_name));
                    });
                });

                ui.add_space(theme.spacing.large);

                ui.vertical(|ui| {
                    ui.set_width(60.);

                    let image_size = ui.available_height() / 2.0 * 0.5;

                    ui.horizontal(|ui| {
                        let raw_text =
                            text_utils::mod_card_data_header(self.mod_entry.modloader.to_string());

                        let text = match self.mod_entry.modloader {
                            ModLoader::Forge => {
                                ui.image(images.forge.as_mut().unwrap(), Vec2::splat(image_size));

                                raw_text.color(theme.mod_card_modloader().forge)
                            }
                            ModLoader::Fabric => {
                                ui.image(images.fabric.as_mut().unwrap(), Vec2::splat(image_size));

                                raw_text.color(theme.mod_card_modloader().fabric)
                            }
                            ModLoader::Both => {
                                ui.image(
                                    images.forge_and_fabric.as_mut().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(theme.mod_card_modloader().forge_and_fabric)
                            }
                        };

                        ui.add_space(theme.spacing.medium);

                        ui.label(text);
                    });

                    ui.horizontal(|ui| {
                        let raw_text = text_utils::mod_card_data_header(
                            self.mod_entry.sourced_from.to_string(),
                        );

                        let text = match self.mod_entry.sourced_from {
                            CurrentSource::None => {
                                ui.image(images.none.as_mut().unwrap(), Vec2::splat(image_size));

                                raw_text.color(theme.mod_card_source().none)
                            }
                            CurrentSource::Local => {
                                ui.image(images.local.as_mut().unwrap(), Vec2::splat(image_size));

                                raw_text.color(theme.mod_card_source().local)
                            }
                            CurrentSource::Modrinth => {
                                ui.image(
                                    images.modrinth.as_mut().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(theme.mod_card_source().modrinth)
                            }
                            CurrentSource::CurseForge => {
                                ui.image(
                                    images.curseforge.as_mut().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(theme.mod_card_source().curseforge)
                            }
                        };

                        ui.add_space(theme.spacing.small);

                        ui.spacing_mut().button_padding = Vec2::new(3.0, 0.0);

                        ComboBox::from_id_source(&self.mod_entry.path)
                            .selected_text(text)
                            .width(75.0)
                            .icon(misc::combobox_icon_fn)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.mod_entry.sourced_from,
                                    CurrentSource::Local,
                                    text_utils::mod_card_data_header(
                                        &CurrentSource::Local.to_string(),
                                    ),
                                );

                                if self.mod_entry.sources.curseforge.is_some() {
                                    ui.selectable_value(
                                        &mut self.mod_entry.sourced_from,
                                        CurrentSource::CurseForge,
                                        text_utils::mod_card_data_header(
                                            &CurrentSource::CurseForge.to_string(),
                                        ),
                                    );
                                }

                                if self.mod_entry.sources.modrinth.is_some() {
                                    ui.selectable_value(
                                        &mut self.mod_entry.sourced_from,
                                        CurrentSource::Modrinth,
                                        text_utils::mod_card_data_header(
                                            &CurrentSource::Modrinth.to_string(),
                                        ),
                                    );
                                }
                            });
                    });
                });

                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.add_space(theme.spacing.large);

                    let button =
                        ImageButton::new(images.bin.as_mut().unwrap().id(), Vec2::splat(12.));

                    if ui.add(button).clicked() {
                        if let Some(tx) = &front_tx {
                            tx.send(ToBackend::DeleteMod {
                                path: self.mod_entry.path.clone(),
                            })
                            .unwrap();
                        }
                    };

                    ui.add_space(theme.spacing.medium);

                    if self.mod_entry.state == FileState::Outdated
                        && ui
                            .button(text_utils::update_button_text("Update"))
                            .clicked()
                    {
                        if let Some(tx) = &front_tx {
                            tx.send(ToBackend::UpdateMod {
                                mod_entry: Box::new(self.mod_entry.clone()),
                            })
                            .unwrap();
                        }
                    }
                });
            });
        });

        ui.interact(
            frame_res.response.rect,
            Id::new(ui.id()).with(&self.mod_entry.path),
            Sense::click(),
        )
    }
}

fn mod_info_text(
    header: impl Into<String>,
    body: impl Into<String>,
    ui: &mut Ui,
    theme: &AppTheme,
) {
    ui.horizontal(|ui| {
        ui.label(text_utils::mod_card_data_header(header).color(theme.colors.lighter_gray));

        ui.label(text_utils::mod_card_data_text(body));
    });
}
