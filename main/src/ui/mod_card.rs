use std::collections::HashMap;

use back::{
    messages::ToBackend,
    mod_entry::{CurrentSource, FileState, ModEntry, ModFile, ModLoader},
};
use crossbeam_channel::Sender;
use eframe::{
    egui::{
        collapsing_header, style::Margin, ComboBox, Context, Frame, ImageButton, Layout, Response,
        Sense, Ui,
    },
    emath::Vec2,
    epaint::{ColorImage, Rounding, TextureHandle},
};

use super::{misc, text_utils, ICON_RESIZE_QUALITY, IMAGES, THEME};

pub struct FileCard {
    mod_file: ModFile,
    mod_icons: HashMap<String, TextureHandle>,
}

impl FileCard {
    pub fn new(mut mod_file: ModFile, ctx: &Context) -> Self {
        let mut mod_icons: HashMap<String, TextureHandle> = HashMap::new();
        for entry in &mut mod_file.entries {
            if let Some(image_raw) = &entry.icon {
                let key = format!("{}{}", mod_file.hashes.sha1, entry.id);
                let texture_handle = ctx.load_texture(
                    key.clone(),
                    ColorImage::from_rgba_unmultiplied(
                        [ICON_RESIZE_QUALITY as usize, ICON_RESIZE_QUALITY as usize],
                        image_raw.as_slice(),
                    ),
                );

                mod_icons.insert(key, texture_handle);
            };

            // No need to keep the icon data afterwards
            entry.icon = None;
        }

        Self {
            mod_file,
            mod_icons,
        }
    }

    pub fn mod_file(&self) -> &ModFile {
        &self.mod_file
    }

    pub fn show(
        &mut self,
        current_search: &str,
        ui: &mut Ui,
        front_tx: &Option<Sender<ToBackend>>,
    ) {
        let mod_file = &mut self.mod_file;
        // Skip the entries that are not within the filtered list
        for entry in mod_file.entries.clone() {
            if !entry
                .display_name
                .to_lowercase()
                .contains(current_search.to_lowercase().as_str())
            {
                continue;
            }

            let key = format!("{}{}", mod_file.hashes.sha1, entry.id);
            let mod_icon = self.mod_icons.get(&key);

            ModCard::show(&entry, mod_file, ui, front_tx, mod_icon);
        }
    }
}

pub struct ModCard {}

impl ModCard {
    pub fn show(
        mod_entry: &ModEntry,
        mod_file: &mut ModFile,
        ui: &mut Ui,
        front_tx: &Option<Sender<ToBackend>>,
        mod_icon: Option<&TextureHandle>,
    ) {
        let mut state = collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id("mod_collapsing_header")
                .with(&mod_file.path),
            false,
        );

        let header_res = Self::render_header(mod_entry, mod_file, ui, front_tx, mod_icon);

        if header_res.clicked() {
            state.toggle(ui);
        }

        state.show_body_indented(&header_res, ui, |ui| {
            ui.spacing_mut().item_spacing.y = THEME.spacing.small;

            mod_info_text("Version:", &mod_entry.version, ui);

            mod_info_text(
                "Description:",
                mod_entry.description.as_deref().unwrap_or("None"),
                ui,
            );

            mod_info_text(
                "Authors:",
                mod_entry.authors.as_deref().unwrap_or("None"),
                ui,
            );

            mod_info_text("Mod path:", mod_file.path.display().to_string(), ui);
        });
    }

    pub fn render_header(
        mod_entry: &ModEntry,
        mod_file: &mut ModFile,
        ui: &mut Ui,
        front_tx: &Option<Sender<ToBackend>>,
        mod_icon: Option<&TextureHandle>,
    ) -> Response {
        let images = IMAGES.lock();

        let frame_res = Frame {
            fill: THEME.colors.dark_gray,
            rounding: Rounding::same(2.0),
            ..Frame::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(36.);

                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                Frame {
                    inner_margin: Margin::symmetric(6.0, 0.0),
                    fill: THEME.colors.mod_card.mod_status_icon_background,
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    let image_size = Vec2::splat(12.0);
                    match mod_file.state {
                        FileState::Current => {
                            ui.image(images.mod_status_ok.as_ref().unwrap().id(), image_size)
                        }
                        FileState::Outdated => ui.image(
                            images.mod_status_outdated.as_ref().unwrap().id(),
                            image_size,
                        ),
                        FileState::Invalid => {
                            ui.image(images.mod_status_invalid.as_ref().unwrap().id(), image_size)
                        }
                        FileState::Local => ui.image(
                            // There's not much that can be done here, assume its all good
                            images.mod_status_ok.as_ref().unwrap().id(),
                            image_size,
                        ),
                    };
                });

                ui.add_space(THEME.spacing.medium);

                let icon_size = 26.0;

                Frame {
                    rounding: Rounding::same(5.0),
                    fill: THEME.colors.mod_card.mod_status_icon_background,
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    ui.set_width(icon_size);
                    ui.set_height(icon_size);
                    if let Some(texture) = &mod_icon {
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
                        ui.label(text_utils::mod_name_job(ui, &mod_entry.display_name));
                    });
                });

                ui.add_space(THEME.spacing.large);

                ui.vertical(|ui| {
                    ui.set_width(60.);

                    let image_size = ui.available_height() / 2.0 * 0.5;

                    ui.horizontal(|ui| {
                        let raw_text =
                            text_utils::mod_card_data_header(mod_entry.modloader.to_string());

                        let text = match mod_entry.modloader {
                            ModLoader::Forge => {
                                ui.image(images.forge.as_ref().unwrap(), Vec2::splat(image_size));

                                raw_text.color(THEME.mod_card_modloader().forge)
                            }
                            ModLoader::Fabric => {
                                ui.image(images.fabric.as_ref().unwrap(), Vec2::splat(image_size));

                                raw_text.color(THEME.mod_card_modloader().fabric)
                            }
                            ModLoader::Both => {
                                ui.image(
                                    images.forge_and_fabric.as_ref().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(THEME.mod_card_modloader().forge_and_fabric)
                            }
                        };

                        ui.add_space(THEME.spacing.medium);

                        ui.label(text);
                    });

                    ui.horizontal(|ui| {
                        let raw_text =
                            text_utils::mod_card_data_header(mod_file.sourced_from.to_string());

                        let text = match mod_file.sourced_from {
                            CurrentSource::None => {
                                ui.image(images.none.as_ref().unwrap(), Vec2::splat(image_size));

                                raw_text.color(THEME.mod_card_source().none)
                            }
                            CurrentSource::Local => {
                                ui.image(images.local.as_ref().unwrap(), Vec2::splat(image_size));

                                raw_text.color(THEME.mod_card_source().local)
                            }
                            CurrentSource::Modrinth => {
                                ui.image(
                                    images.modrinth.as_ref().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(THEME.mod_card_source().modrinth)
                            }
                            CurrentSource::CurseForge => {
                                ui.image(
                                    images.curseforge.as_ref().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(THEME.mod_card_source().curseforge)
                            }
                        };

                        ui.add_space(THEME.spacing.small);

                        ui.spacing_mut().button_padding = Vec2::new(3.0, 0.0);

                        ComboBox::from_id_source(&mod_file.path)
                            .selected_text(text)
                            .width(75.0)
                            .icon(misc::combobox_icon_fn)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut mod_file.sourced_from,
                                    CurrentSource::Local,
                                    text_utils::mod_card_data_header(
                                        &CurrentSource::Local.to_string(),
                                    ),
                                );

                                if mod_file.sources.curseforge.is_some() {
                                    ui.selectable_value(
                                        &mut mod_file.sourced_from,
                                        CurrentSource::CurseForge,
                                        text_utils::mod_card_data_header(
                                            &CurrentSource::CurseForge.to_string(),
                                        ),
                                    );
                                }

                                if mod_file.sources.modrinth.is_some() {
                                    ui.selectable_value(
                                        &mut mod_file.sourced_from,
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
                    ui.add_space(THEME.spacing.large);

                    let button =
                        ImageButton::new(images.bin.as_ref().unwrap().id(), Vec2::splat(12.));

                    if ui.add(button).clicked() {
                        if let Some(tx) = &front_tx {
                            tx.send(ToBackend::DeleteMod {
                                path: mod_file.path.clone(),
                            })
                            .unwrap();
                        }
                    };

                    ui.add_space(THEME.spacing.medium);

                    if mod_file.state == FileState::Outdated
                        && ui
                            .button(text_utils::update_button_text("Update"))
                            .clicked()
                    {
                        if let Some(tx) = &front_tx {
                            tx.send(ToBackend::UpdateMod {
                                mod_entry: Box::new(mod_file.clone()),
                            })
                            .unwrap();
                        }
                    }
                });
            });
        });

        ui.interact(
            frame_res.response.rect,
            ui.make_persistent_id(&mod_file.path),
            Sense::click(),
        )
    }
}

fn mod_info_text(header: impl Into<String>, body: impl Into<String>, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(text_utils::mod_card_data_header(header).color(THEME.colors.lighter_gray));

        ui.label(text_utils::mod_card_data_text(body));
    });
}
