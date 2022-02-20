use back::{
    messages::{BackendError, CheckProgress, ToBackend, ToFrontend},
    mod_entry::{FileState, ModEntry, ModLoader, Source},
    Back, GameVersion,
};

use std::thread;

use crossbeam_channel::{Receiver, Sender};

use eframe::{
    egui::{
        self,
        style::{DebugOptions, Margin},
        Align, Context, ImageButton, Layout, ProgressBar, RichText, Rounding, Style, Ui, Vec2,
        Widget,
    },
    epi,
};

use self::{app_theme::AppTheme, image_utils::ImageTextures};

mod app_theme;
mod image_utils;
mod text_utils;

#[derive(Default)]
pub struct UiApp {
    theme: AppTheme,
    mod_list: Vec<ModEntry>,
    game_version_list: Vec<GameVersion>,
    search_buf: String,
    add_mod_buf: String,
    selected_version: Option<GameVersion>,
    selected_modloader: ModLoader,
    images: ImageTextures,

    front_tx: Option<Sender<ToBackend>>,
    back_rx: Option<Receiver<ToFrontend>>,
    backend_context: BackendContext,
    backend_errors: Vec<BackendError>,
}

#[derive(Default)]
struct BackendContext {
    check_for_update_progress: Option<CheckProgress>,
}

impl epi::App for UiApp {
    fn name(&self) -> &str {
        "mCubed"
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.configure_style(ctx);
        self.images.load_images(ctx);

        let (front_tx, front_rx) = crossbeam_channel::unbounded();
        let (back_tx, back_rx) = crossbeam_channel::unbounded();

        let frame_clone = frame.clone();
        thread::spawn(move || {
            Back::new(None, back_tx, front_rx, Some(frame_clone)).init();
        });

        self.front_tx = Some(front_tx);
        self.back_rx = Some(back_rx);

        if let Some(sender) = &self.front_tx {
            sender.send(ToBackend::Startup).unwrap();
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        if let Some(rx) = &self.back_rx {
            match rx.try_recv() {
                Ok(message) => match message {
                    ToFrontend::SetVersionMetadata { manifest } => {
                        self.selected_version = Some(manifest.versions[0].clone());
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
                        self.backend_errors.push(error);
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

    fn on_exit(&mut self) {
        if let Some(tx) = &self.front_tx {
            tx.send(ToBackend::Shutdown).unwrap();
        }
    }
}

impl UiApp {
    fn render_side_panel(&mut self, ctx: &Context) {
        egui::SidePanel::left("options_panel")
            .frame(self.theme.default_panel_frame)
            .resizable(false)
            .max_width(180.)
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 8.0);

                egui::Frame {
                    fill: self.theme.colors.light_gray,
                    margin: Margin::same(10.0),
                    rounding: Rounding::same(4.),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    egui::Frame {
                        fill: self.theme.colors.dark_gray,
                        margin: Margin::same(4.0),
                        rounding: Rounding::same(4.),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| {
                            ui.label("Game Version");
                        });
                    });
                    ui.vertical_centered_justified(|ui| {
                        egui::ComboBox::from_id_source("version-combo")
                            .selected_text(format!("{:?}", {
                                if let Some(selected_value) = self.selected_version.as_ref() {
                                    selected_value.id.as_str()
                                } else if self.game_version_list.is_empty() {
                                    "Fetching version list..."
                                } else {
                                    self.selected_version = Some(self.game_version_list[0].clone());
                                    self.selected_version.as_ref().unwrap().id.as_str()
                                }
                            }))
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
                    ui.horizontal(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            let edit = egui::TextEdit::singleline(&mut self.add_mod_buf).hint_text(
                                RichText::new("Modrinth ID or Slug").color(self.theme.colors.gray),
                            );
                            ui.add(edit);
                        });

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
                    ui.horizontal(|ui| {
                        if ui.button("Re-scan Folder").clicked() {
                            if let Some(tx) = &self.front_tx {
                                tx.send(ToBackend::ScanFolder).unwrap();
                            }
                        }

                        if ui.button("Refresh").clicked() {
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

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.backend_errors.retain(|error| {
                        let mut retain = true;

                        egui::Frame {
                            fill: self.theme.colors.error_message,
                            margin: Margin::same(6.0),
                            rounding: Rounding::same(4.),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(&error.message)
                                    .on_hover_text(error.error.to_string());
                                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                    if ui.button("Close").clicked() {
                                        retain = false
                                    }
                                });
                            });
                        });

                        retain
                    });
                });

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
                            let filtered_list: Vec<ModEntry> =
                                self.mod_list
                                    .iter()
                                    .filter(|mod_entry| {
                                        mod_entry.display_name.to_lowercase().contains(
                                            self.search_buf.as_str().to_lowercase().as_str(),
                                        )
                                    })
                                    .cloned()
                                    .collect();

                            if filtered_list.is_empty() {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No mods match your search");
                                });
                            } else {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.style_mut().spacing.item_spacing.y = 10.0;

                                    for mod_entry in filtered_list {
                                        self.render_mod_card(ui, &mod_entry);
                                    }
                                });
                            }
                        }
                    });
                });
            });
    }

    fn render_mod_card(&mut self, ui: &mut Ui, mod_entry: &ModEntry) {
        egui::Frame {
            fill: self.theme.colors.dark_gray,
            rounding: Rounding::same(2.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(36.);

                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                let version = mod_entry.normalized_version();
                egui::Frame {
                    margin: Margin::symmetric(10.0, 0.0),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_width(120.);
                    ui.set_max_width(120.);

                    ui.centered_and_justified(|ui| {
                        ui.style_mut().wrap = Some(true);
                        ui.label(&mod_entry.display_name).on_hover_text(
                            text_utils::mod_card_data_text(
                                mod_entry.path.as_ref().unwrap().display().to_string(),
                            ),
                        );
                    });
                });

                egui::Frame {
                    fill: self.theme.colors.gray,
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_width(45.);
                    ui.centered_and_justified(|ui| {
                        let raw = text_utils::mod_version_text(version);

                        let text = match mod_entry.state {
                            FileState::Current => {
                                raw.color(self.theme.mod_card_status().up_to_date)
                            }
                            FileState::Outdated => raw.color(self.theme.mod_card_status().outdated),
                            FileState::Local => raw.color(self.theme.colors.white),
                            FileState::Invalid => raw.color(self.theme.mod_card_status().invalid),
                        };

                        ui.label(text);
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
                            Source::Local | Source::ExplicitLocal => {
                                ui.image(
                                    self.images.local.as_mut().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(self.theme.mod_card_source().local)
                            }
                            Source::Modrinth => {
                                ui.image(
                                    self.images.modrinth.as_mut().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(self.theme.mod_card_source().modrinth)
                            }
                            Source::CurseForge => {
                                ui.image(
                                    self.images.curseforge.as_mut().unwrap(),
                                    Vec2::splat(image_size),
                                );

                                raw_text.color(self.theme.mod_card_source().curseforge)
                            }
                        };

                        ui.add_space(5.);

                        ui.label(text);
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.add_space(10.);

                    let button =
                        ImageButton::new(self.images.bin.as_mut().unwrap().id(), Vec2::splat(12.));

                    if ui.add(button).clicked() {
                        if let Some(tx) = &self.front_tx {
                            tx.send(ToBackend::DeleteMod {
                                path: mod_entry.path.as_ref().unwrap().clone(),
                            })
                            .unwrap();
                        }
                    };

                    ui.add_space(5.0);

                    if mod_entry.state == FileState::Outdated {
                        if ui
                            .button(text_utils::update_button_text("Update"))
                            .clicked()
                        {
                            if let Some(tx) = &self.front_tx {
                                tx.send(ToBackend::UpdateMod {
                                    mod_entry: mod_entry.clone(),
                                })
                                .unwrap();
                            }
                        }
                    }
                });
            });
        });
    }
}
impl UiApp {
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
