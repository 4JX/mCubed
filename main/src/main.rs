use app_theme::AppTheme;
use back::{
    messages::{CheckProgress, ToBackend, ToFrontend},
    mod_entry::{FileState, ModEntry, ModLoader, Source},
    Back, GameVersion,
};
use image_utils::ImageTextures;

use std::{
    collections::HashMap,
    fs,
    sync::mpsc::{Receiver, Sender},
    thread,
};

use eframe::{
    egui::{
        self, style::DebugOptions, Context, ProgressBar, RichText, Rounding, Style, Vec2, Widget,
    },
    epi,
};

mod app_theme;
mod image_utils;

mod text_utils;
#[derive(Default)]
struct UiApp {
    theme: AppTheme,
    mod_list: Vec<ModEntry>,
    game_version_list: Vec<GameVersion>,
    search_buf: String,
    selected_version: Option<GameVersion>,
    images: ImageTextures,
    mod_hash_cache: HashMap<String, String>,
    front_tx: Option<Sender<ToBackend>>,
    back_rx: Option<Receiver<ToFrontend>>,
    backend_context: BackendContext,
}

#[derive(Default)]
struct BackendContext {
    check_for_update_progress: Option<CheckProgress>,
}

impl epi::App for UiApp {
    fn name(&self) -> &str {
        "An App"
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        let mod_hash_cache = serde_json::to_string(&self.mod_hash_cache).unwrap();

        storage.set_string("hash_storage", mod_hash_cache);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        frame: &epi::Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            if let Some(hash_json) = storage.get_string("hash_storage") {
                if let Ok(cache) = serde_json::from_str(hash_json.as_str()) {
                    self.mod_hash_cache = cache;
                };
            }
        }

        self.configure_style(ctx);
        self.images.load_images(ctx);

        let (front_tx, front_rx) = std::sync::mpsc::channel::<ToBackend>();
        let (back_tx, back_rx) = std::sync::mpsc::channel::<ToFrontend>();

        let dir = fs::canonicalize("./mods/").unwrap();

        let frame_clone = frame.clone();
        thread::spawn(move || {
            Back::new(dir, back_tx, front_rx, Some(frame_clone)).init();
        });

        self.front_tx = Some(front_tx);
        self.back_rx = Some(back_rx);

        if let Some(sender) = &self.front_tx {
            sender.send(ToBackend::ScanFolder).unwrap();

            if let Some(rx) = &self.back_rx {
                if let Ok(message) = rx.recv() {
                    match message {
                        ToFrontend::UpdateModList { mod_list } => {
                            self.backend_context.check_for_update_progress = None;
                            self.mod_list = mod_list;
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
            }

            sender.send(ToBackend::GetVersionMetadata).unwrap();
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        if let Some(rx) = &self.back_rx {
            match rx.try_recv() {
                Ok(message) => match message {
                    ToFrontend::SetVersionMetadata { version_list } => {
                        self.selected_version = Some(version_list[0].clone());
                        self.game_version_list = version_list;
                    }
                    ToFrontend::UpdateModList { mod_list } => {
                        self.backend_context.check_for_update_progress = None;
                        self.mod_list = mod_list;
                        ctx.request_repaint();
                    }
                    ToFrontend::CheckForUpdatesProgress { progress } => {
                        self.backend_context.check_for_update_progress = Some(progress);
                    }
                },
                Err(err) => {
                    let _ = err;
                }
            }
        }

        egui::SidePanel::left("options_panel")
            .resizable(false)
            .max_width(180.)
            .show(ctx, |ui| {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical_centered_justified(|ui| {
                    let edit = egui::TextEdit::singleline(&mut self.search_buf).hint_text(
                        RichText::new("Search installed mods").color(self.theme.colors.gray),
                    );
                    ui.add(edit);
                });
            });

            ui.add_space(5.);

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

                ui.add_space(5.);
            }

            ui.vertical_centered_justified(|ui| {
                egui::Frame {
                    fill: self.theme.colors.darker_gray,
                    margin: Vec2::new(10., 10.),
                    rounding: Rounding::same(4.),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_height(ui.available_height());

                    let filtered_list: Vec<&ModEntry> = self
                        .mod_list
                        .iter()
                        .filter(|mod_entry| {
                            mod_entry
                                .display_name
                                .to_lowercase()
                                .contains(self.search_buf.as_str().to_lowercase().as_str())
                        })
                        .collect();

                    if self.mod_list.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("There are no mods to display");
                        });
                    } else if filtered_list.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("No mods match your search");
                        });
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing.y = 10.0;

                            for mod_entry in filtered_list {
                                egui::Frame {
                                    fill: self.theme.colors.dark_gray,
                                    ..Default::default()
                                }
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.set_height(36.);

                                        ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                                        let version = mod_entry.normalized_version();
                                        egui::Frame {
                                            margin: Vec2::new(10.0, 0.0),
                                            ..Default::default()
                                        }
                                        .show(ui, |ui| {
                                            ui.set_width(120.);
                                            ui.set_max_width(120.);

                                            ui.centered_and_justified(|ui| {
                                                ui.style_mut().wrap = Some(true);
                                                ui.label(&mod_entry.display_name);
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
                                                    FileState::Current => raw.color(
                                                        self.theme
                                                            .colors
                                                            .mod_card
                                                            .version_status
                                                            .up_to_date,
                                                    ),
                                                    FileState::Outdated => raw.color(
                                                        self.theme
                                                            .colors
                                                            .mod_card
                                                            .version_status
                                                            .outdated,
                                                    ),
                                                    FileState::Local => {
                                                        raw.color(self.theme.colors.white)
                                                    }
                                                    FileState::Invalid => raw.color(
                                                        self.theme
                                                            .colors
                                                            .mod_card
                                                            .version_status
                                                            .invalid,
                                                    ),
                                                };

                                                ui.label(text);
                                            });
                                        });

                                        ui.add_space(10.);

                                        ui.vertical(|ui| {
                                            ui.set_width(60.);

                                            let image_size = ui.available_height() / 2.0 * 0.5;

                                            ui.horizontal(|ui| {
                                                let raw_text = text_utils::mod_card_data_text(
                                                    mod_entry.modloader.to_string(),
                                                );

                                                let text = match mod_entry.modloader {
                                                    ModLoader::Forge => {
                                                        ui.image(
                                                            self.images.forge.as_mut().unwrap(),
                                                            Vec2::splat(image_size),
                                                        );

                                                        raw_text.color(
                                                            self.theme
                                                                .colors
                                                                .mod_card
                                                                .modloader
                                                                .forge,
                                                        )
                                                    }
                                                    ModLoader::Fabric => {
                                                        ui.image(
                                                            self.images.fabric.as_mut().unwrap(),
                                                            Vec2::splat(image_size),
                                                        );

                                                        raw_text.color(
                                                            self.theme
                                                                .colors
                                                                .mod_card
                                                                .modloader
                                                                .fabric,
                                                        )
                                                    }
                                                    ModLoader::Both => {
                                                        ui.image(
                                                            self.images
                                                                .forge_and_fabric
                                                                .as_mut()
                                                                .unwrap(),
                                                            Vec2::splat(image_size),
                                                        );

                                                        raw_text.color(
                                                            self.theme
                                                                .colors
                                                                .mod_card
                                                                .modloader
                                                                .forge_and_fabric,
                                                        )
                                                    }
                                                };

                                                ui.add_space(5.);

                                                ui.label(text);
                                            });

                                            ui.horizontal(|ui| {
                                                let raw_text = text_utils::mod_card_data_text(
                                                    mod_entry.sourced_from.to_string(),
                                                );

                                                let text = match mod_entry.sourced_from {
                                                    Source::Local | Source::ExplicitLocal => {
                                                        ui.image(
                                                            self.images.local.as_mut().unwrap(),
                                                            Vec2::splat(image_size),
                                                        );

                                                        raw_text.color(
                                                            self.theme.colors.mod_card.source.local,
                                                        )
                                                    }
                                                    Source::Modrinth => {
                                                        ui.image(
                                                            self.images.modrinth.as_mut().unwrap(),
                                                            Vec2::splat(image_size),
                                                        );

                                                        raw_text.color(
                                                            self.theme
                                                                .colors
                                                                .mod_card
                                                                .source
                                                                .modrinth,
                                                        )
                                                    }
                                                    Source::CurseForge => {
                                                        ui.image(
                                                            self.images
                                                                .curseforge
                                                                .as_mut()
                                                                .unwrap(),
                                                            Vec2::splat(image_size),
                                                        );

                                                        raw_text.color(
                                                            self.theme
                                                                .colors
                                                                .mod_card
                                                                .source
                                                                .curseforge,
                                                        )
                                                    }
                                                };

                                                ui.add_space(5.);

                                                ui.label(text);
                                            });
                                        });

                                        ui.centered_and_justified(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                                ui.add_space(10.);
                                                ui.image(
                                                    self.images.bin.as_mut().unwrap(),
                                                    Vec2::splat(12.),
                                                );

                                                if mod_entry.state == FileState::Outdated {
                                                    ui.add_space(5.0);
                                                    if ui
                                                        .button(text_utils::update_button_text(
                                                            "Update",
                                                        ))
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
                                });
                            }
                        });
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
                show_resize: true,
            },
            ..Default::default()
        };

        ctx.set_fonts(text_utils::get_font_def());
        ctx.set_style(style);
    }
}

fn main() {
    let app = UiApp::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(970., 300.)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
