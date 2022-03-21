use back::{
    messages::{BackendError, CheckProgress, ToBackend, ToFrontend},
    mod_entry::{CurrentSource, FileState, ModEntry, ModLoader},
    Back, GameVersion,
};
use parking_lot::Once;

use std::thread;

use crossbeam_channel::{Receiver, Sender};

use eframe::{
    egui::{
        self,
        style::{DebugOptions, Margin},
        Align, Context, ImageButton, Label, Layout, ProgressBar, RichText, Rounding, Style, Ui,
        Vec2, Widget,
    },
    epi, CreationContext,
};

use self::{app_theme::AppTheme, image_utils::ImageTextures};

mod app_theme;
mod image_utils;
mod misc;
mod text_utils;

static SET_LEFT_PANEL_BOTTOM_BUTTONS_WIDTH: Once = Once::new();

#[derive(Default)]
pub struct MCubedAppUI {
    // UI
    theme: AppTheme,
    search_buf: String,
    add_mod_buf: String,
    images: ImageTextures,

    // Data
    mod_list: Vec<ModEntry>,
    game_version_list: Vec<GameVersion>,
    game_version_string_list: Vec<String>,
    selected_version: Option<GameVersion>,
    selected_modloader: ModLoader,
    backend_context: BackendContext,

    // Data transferring
    front_tx: Option<Sender<ToBackend>>,
    back_rx: Option<Receiver<ToFrontend>>,

    // Misc sizes to combat immediate mode shenanigans
    left_panel_bottom_buttons_width: f32,
}

#[derive(Default)]
struct BackendContext {
    check_for_update_progress: Option<CheckProgress>,
    backend_errors: Vec<BackendError>,
}

impl MCubedAppUI {
    pub fn new(cc: &CreationContext) -> Self {
        let mut new_app = Self::default();

        new_app.configure_style(&cc.egui_ctx);
        new_app.images.load_images(&cc.egui_ctx);

        let (front_tx, front_rx) = crossbeam_channel::unbounded();
        let (back_tx, back_rx) = crossbeam_channel::unbounded();

        let frame_clone = cc.egui_ctx.clone();
        thread::spawn(move || {
            Back::new(None, back_tx, front_rx, Some(frame_clone)).init();
        });

        new_app.front_tx = Some(front_tx);
        new_app.back_rx = Some(back_rx);

        if let Some(sender) = &new_app.front_tx {
            sender.send(ToBackend::Startup).unwrap();
        }

        new_app
    }
}

impl epi::App for MCubedAppUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        if let Some(rx) = &self.back_rx {
            match rx.try_recv() {
                Ok(message) => match message {
                    ToFrontend::SetVersionMetadata { manifest } => {
                        self.selected_version = Some(manifest.versions[0].clone());

                        // Get both a String and GameVersion vec
                        self.game_version_string_list = manifest
                            .versions
                            .iter()
                            .map(|game_version| game_version.id.clone())
                            .collect();
                        self.game_version_list = manifest.versions;
                    }
                    ToFrontend::UpdateModList { mod_list } => {
                        self.backend_context.check_for_update_progress = None;
                        self.mod_list = mod_list;
                        ctx.request_repaint();
                    }
                    ToFrontend::CheckForUpdatesProgress { progress } => {
                        self.backend_context.check_for_update_progress = Some(progress);
                    }
                    ToFrontend::BackendError { error } => {
                        self.backend_context.backend_errors.push(error);
                    }
                },
                Err(err) => {
                    let _ = err;
                }
            }
        }

        self.render_side_panel(ctx);

        self.render_central_panel(ctx);
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        if let Some(tx) = &self.front_tx {
            tx.send(ToBackend::UpdateBackendList {
                mod_list: self.mod_list.clone(),
            })
            .unwrap();

            tx.send(ToBackend::Shutdown).unwrap();
        }
    }
}

impl MCubedAppUI {
    fn render_side_panel(&mut self, ctx: &Context) {
        egui::SidePanel::left("options_panel")
            .frame(self.theme.default_panel_frame)
            .resizable(false)
            .max_width(240.)
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 8.0);

                ui.horizontal(|ui| {
                    ui.label("Game Version");

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        egui::ComboBox::from_id_source("version-combo")
                            .icon(misc::combobox_icon_fn)
                            .selected_text(
                                if let Some(selected_value) = self.selected_version.as_ref() {
                                    selected_value.id.as_str()
                                } else if self.game_version_list.is_empty() {
                                    "Loading..."
                                } else {
                                    self.selected_version = Some(self.game_version_list[0].clone());
                                    self.selected_version.as_ref().unwrap().id.as_str()
                                },
                            )
                            .show_ui(ui, |ui| {
                                for version in &self.game_version_list {
                                    ui.selectable_value(
                                        &mut self.selected_version,
                                        Some(version.clone()),
                                        &version.id,
                                    );
                                }
                            });
                    });
                });

                egui::Frame {
                    fill: self.theme.colors.light_gray,
                    margin: Margin::same(10.0),
                    rounding: Rounding::same(4.),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    // Fill the side panel
                    ui.set_width(ui.available_width());

                    ui.horizontal(|ui| {
                        let edit = egui::TextEdit::singleline(&mut self.add_mod_buf).hint_text(
                            RichText::new("Modrinth ID or Slug").color(self.theme.colors.gray),
                        );

                        ui.add_sized(Vec2::new(130.0, ui.available_height()), edit);

                        if ui.button("Fetch Mod").clicked() {
                            if let Some(tx) = &self.front_tx {
                                if let Some(version) = &self.selected_version {
                                    tx.send(ToBackend::AddMod {
                                        modrinth_id: self.add_mod_buf.clone(),
                                        game_version: version.id.clone(),
                                        modloader: self.selected_modloader,
                                    })
                                    .unwrap();
                                } else {
                                    tx.send(ToBackend::AddMod {
                                        modrinth_id: self.add_mod_buf.clone(),
                                        game_version: self.game_version_list[0].id.clone(),
                                        modloader: self.selected_modloader,
                                    })
                                    .unwrap();
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.selected_modloader, ModLoader::Forge, "Forge");
                        ui.radio_value(&mut self.selected_modloader, ModLoader::Fabric, "Fabric");
                    });
                });

                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    ui.set_max_width(self.left_panel_bottom_buttons_width);
                    let horizontal_res = ui.horizontal(|ui| {
                        let rescan_folder_button_res = ui.button("Re-scan Folder");

                        if rescan_folder_button_res.clicked() {
                            if let Some(tx) = &self.front_tx {
                                tx.send(ToBackend::UpdateBackendList {
                                    mod_list: self.mod_list.clone(),
                                })
                                .unwrap();
                                tx.send(ToBackend::ScanFolder).unwrap();
                            }
                        };

                        let refresh_button_res = ui.button("Refresh");
                        if refresh_button_res.clicked() {
                            if let Some(tx) = &self.front_tx {
                                if let Some(version) = &self.selected_version {
                                    tx.send(ToBackend::CheckForUpdates {
                                        game_version: version.id.clone(),
                                    })
                                    .unwrap();
                                } else {
                                    tx.send(ToBackend::CheckForUpdates {
                                        game_version: self.game_version_list[0].id.clone(),
                                    })
                                    .unwrap();
                                }
                            }
                        }
                    });

                    SET_LEFT_PANEL_BOTTOM_BUTTONS_WIDTH.call_once(|| {
                        self.left_panel_bottom_buttons_width = horizontal_res.response.rect.width();
                    });
                });
            });
    }

    fn render_central_panel(&mut self, ctx: &Context) {
        egui::CentralPanel::default()
            .frame(self.theme.default_panel_frame)
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 8.0);
                ui.horizontal(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        let edit = egui::TextEdit::singleline(&mut self.search_buf).hint_text(
                            RichText::new("Search installed mods").color(self.theme.colors.gray),
                        );
                        ui.add(edit);
                    });
                });

                if !self.backend_context.backend_errors.is_empty() {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.backend_context.backend_errors.retain(|error| {
                            let mut retain = true;

                            egui::Frame {
                                fill: self.theme.colors.error_message,
                                margin: Margin::same(6.0),
                                rounding: Rounding::same(4.),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(Label::new(&error.message).wrap(true))
                                        .on_hover_text(error.error.to_string());
                                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                        if ui.button("Close").clicked() {
                                            retain = false;
                                        }
                                    });
                                });
                            });

                            retain
                        });
                    });
                }

                if let Some(progress) = &self.backend_context.check_for_update_progress {
                    let count = progress.position as f32 + 1.0;
                    let total = progress.total_len as f32;

                    ui.vertical_centered(|ui| {
                        ui.style_mut().spacing.interact_size.y = 20.;

                        ProgressBar::new(count / total)
                            .text(format!(
                                "({}/{}) Fetching info for mod \"{}\"",
                                count, total, progress.name,
                            ))
                            .ui(ui);
                    });
                }

                ui.vertical_centered_justified(|ui| {
                    egui::Frame {
                        fill: self.theme.colors.darker_gray,
                        margin: Margin::same(10.0),
                        rounding: Rounding::same(4.),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.set_height(ui.available_height());

                        if self.mod_list.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("There are no mods to display");
                            });
                        } else {
                            let search_results_exist = self.mod_list.iter().any(|mod_entry| {
                                mod_entry
                                    .display_name
                                    .to_lowercase()
                                    .contains(self.search_buf.to_lowercase().as_str())
                            });

                            if !search_results_exist && !self.search_buf.is_empty() {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No mods match your search");
                                });
                            } else {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.style_mut().spacing.item_spacing.y = 10.0;
                                    self.render_mod_cards(ui);
                                });
                            }
                        }
                    });
                });
            });
    }

    fn render_mod_cards(&mut self, ui: &mut Ui) {
        for mod_entry in &mut self.mod_list {
            // Skip the entries that are not within the filtered list
            if !mod_entry
                .display_name
                .to_lowercase()
                .contains(self.search_buf.to_lowercase().as_str())
            {
                continue;
            }

            egui::Frame {
                fill: self.theme.colors.dark_gray,
                rounding: Rounding::same(2.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_height(36.);

                    ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                    egui::Frame {
                        margin: Margin::symmetric(6.0, 0.0),
                        fill: self.theme.colors.mod_card.mod_status_icon_background,
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        let image_size = Vec2::splat(12.0);
                        match mod_entry.state {
                            FileState::Current => ui.image(
                                self.images.mod_status_ok.as_mut().unwrap().id(),
                                image_size,
                            ),
                            FileState::Outdated => ui.image(
                                self.images.mod_status_outdated.as_mut().unwrap().id(),
                                image_size,
                            ),
                            FileState::Invalid => ui.image(
                                self.images.mod_status_invalid.as_mut().unwrap().id(),
                                image_size,
                            ),
                            FileState::Local => ui.image(
                                self.images.mod_status_ok.as_mut().unwrap().id(),
                                image_size,
                            ),
                        };
                    });

                    egui::Frame {
                        margin: Margin::symmetric(10.0, 0.0),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.set_width(120.);
                        ui.set_max_width(120.);

                        ui.centered_and_justified(|ui| {
                            ui.style_mut().wrap = Some(true);
                            ui.label(text_utils::mod_name_job(ui, mod_entry.display_name.clone()))
                                .on_hover_text(text_utils::mod_card_data_text(
                                    mod_entry.path.as_ref().unwrap().display().to_string(),
                                ));
                        });
                    });

                    ui.add_space(10.);

                    ui.vertical(|ui| {
                        ui.set_width(60.);

                        let image_size = ui.available_height() / 2.0 * 0.5;

                        ui.horizontal(|ui| {
                            let raw_text =
                                text_utils::mod_card_data_text(mod_entry.modloader.to_string());

                            let text = match mod_entry.modloader {
                                ModLoader::Forge => {
                                    ui.image(
                                        self.images.forge.as_mut().unwrap(),
                                        Vec2::splat(image_size),
                                    );

                                    raw_text.color(self.theme.mod_card_modloader().forge)
                                }
                                ModLoader::Fabric => {
                                    ui.image(
                                        self.images.fabric.as_mut().unwrap(),
                                        Vec2::splat(image_size),
                                    );

                                    raw_text.color(self.theme.mod_card_modloader().fabric)
                                }
                                ModLoader::Both => {
                                    ui.image(
                                        self.images.forge_and_fabric.as_mut().unwrap(),
                                        Vec2::splat(image_size),
                                    );

                                    raw_text.color(self.theme.mod_card_modloader().forge_and_fabric)
                                }
                            };

                            ui.add_space(5.);

                            ui.label(text);
                        });

                        ui.horizontal(|ui| {
                            let raw_text =
                                text_utils::mod_card_data_text(mod_entry.sourced_from.to_string());

                            let text = match mod_entry.sourced_from {
                                CurrentSource::Local | CurrentSource::ExplicitLocal => {
                                    ui.image(
                                        self.images.local.as_mut().unwrap(),
                                        Vec2::splat(image_size),
                                    );

                                    raw_text.color(self.theme.mod_card_source().local)
                                }
                                CurrentSource::Modrinth => {
                                    ui.image(
                                        self.images.modrinth.as_mut().unwrap(),
                                        Vec2::splat(image_size),
                                    );

                                    raw_text.color(self.theme.mod_card_source().modrinth)
                                }
                                CurrentSource::CurseForge => {
                                    ui.image(
                                        self.images.curseforge.as_mut().unwrap(),
                                        Vec2::splat(image_size),
                                    );

                                    raw_text.color(self.theme.mod_card_source().curseforge)
                                }
                            };

                            ui.add_space(2.0);

                            ui.spacing_mut().button_padding = Vec2::new(3.0, 0.0);

                            egui::ComboBox::from_id_source(&mod_entry.path)
                                .selected_text(text)
                                .width(75.0)
                                .icon(misc::combobox_icon_fn)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut mod_entry.sourced_from,
                                        CurrentSource::ExplicitLocal,
                                        text_utils::mod_card_data_text(
                                            &CurrentSource::ExplicitLocal.to_string(),
                                        ),
                                    );

                                    if mod_entry.sources.curseforge.is_some() {
                                        ui.selectable_value(
                                            &mut mod_entry.sourced_from,
                                            CurrentSource::CurseForge,
                                            text_utils::mod_card_data_text(
                                                &CurrentSource::CurseForge.to_string(),
                                            ),
                                        );
                                    }

                                    if mod_entry.sources.modrinth.is_some() {
                                        ui.selectable_value(
                                            &mut mod_entry.sourced_from,
                                            CurrentSource::Modrinth,
                                            text_utils::mod_card_data_text(
                                                &CurrentSource::Modrinth.to_string(),
                                            ),
                                        );
                                    }
                                });
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                        ui.add_space(10.);

                        let button = ImageButton::new(
                            self.images.bin.as_mut().unwrap().id(),
                            Vec2::splat(12.),
                        );

                        if ui.add(button).clicked() {
                            if let Some(tx) = &self.front_tx {
                                tx.send(ToBackend::DeleteMod {
                                    path: mod_entry.path.as_ref().unwrap().clone(),
                                })
                                .unwrap();
                            }
                        };

                        ui.add_space(5.0);

                        if mod_entry.state == FileState::Outdated
                            && ui
                                .button(text_utils::update_button_text("Update"))
                                .clicked()
                        {
                            if let Some(tx) = &self.front_tx {
                                tx.send(ToBackend::UpdateMod {
                                    mod_entry: Box::new(mod_entry.clone()),
                                })
                                .unwrap();
                            }
                        }
                    });
                });
            });
        }
    }
}

impl MCubedAppUI {
    fn configure_style(&self, ctx: &Context) {
        let style = Style {
            text_styles: text_utils::default_text_styles(),
            visuals: self.theme.visuals.clone(),
            debug: DebugOptions {
                debug_on_hover: false,
                show_expand_width: false,
                show_expand_height: false,
                show_resize: false,
            },
            ..Default::default()
        };

        ctx.set_fonts(text_utils::get_font_def());
        ctx.set_style(style);
    }
}
