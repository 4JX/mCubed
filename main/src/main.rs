use app_theme::AppTheme;
use back::{
    messages::{FetchingModContext, ToBackend, ToFrontend},
    mod_entry::{FileState, ModEntry, Source},
    Back,
};
use image_utils::ImageTextures;

use std::{
    collections::HashMap,
    fs,
    sync::mpsc::{Receiver, Sender},
    thread,
};

use eframe::{
    egui::{self, style::DebugOptions, Context, ProgressBar, RichText, Style, Vec2, Widget},
    epi,
};

mod app_theme;
mod image_utils;

mod text_utils;
#[derive(Default)]
struct UiApp {
    theme: AppTheme,
    mod_list: Vec<ModEntry>,
    search_buf: String,
    receiver: Option<Receiver<ToFrontend>>,
    sender: Option<Sender<ToBackend>>,
    fetching_info: Option<FetchingModContext>,
    images: ImageTextures,
    mod_hash_cache: HashMap<String, String>,
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
        _frame: &epi::Frame,
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

        self.scan_mod_folder();

        let (backend_sender, frontend_reciever) = std::sync::mpsc::channel::<ToFrontend>();
        let (frontend_sender, backend_reciever) = std::sync::mpsc::channel::<ToBackend>();

        let backend = Back::new(backend_sender, backend_reciever);

        thread::spawn(move || {
            backend.init();
        });

        self.sender = Some(frontend_sender);
        self.receiver = Some(frontend_reciever);

        if let Some(front_tx) = &self.sender {
            front_tx
                .send(ToBackend::UpdateModList {
                    mod_list: self.mod_list.clone(),
                    mod_hash_cache: self.mod_hash_cache.clone(),
                })
                .unwrap();
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(message) => {
                    //A
                    match message {
                        ToFrontend::UpdateModList {
                            mod_list,
                            mod_hash_cache,
                        } => {
                            self.fetching_info = None;
                            self.mod_list = mod_list;
                            self.mod_hash_cache = mod_hash_cache;
                        }
                        ToFrontend::FetchingMod { context: mod_name } => {
                            self.fetching_info = Some(mod_name);
                        }
                    }
                }
                Err(err) => {
                    match err {
                        std::sync::mpsc::TryRecvError::Empty => {
                            //Non issue
                        }
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            // Eventually handle
                        }
                    }
                }
            };
        }

        egui::SidePanel::left("options_panel")
            .resizable(false)
            .max_width(180.)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Placeholder");
                });
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

            if let Some(context) = &self.fetching_info {
                let count = context.position as f32;
                let total = context.total as f32;

                ui.vertical_centered(|ui| {
                    ui.style_mut().spacing.interact_size.y = 20.;

                    ProgressBar::new(count / total)
                        .text(format!("Fetching info for mod \"{}\"", context.name))
                        .ui(ui);
                });

                ui.add_space(5.);
                ctx.request_repaint();
            }

            ui.vertical_centered_justified(|ui| {
                egui::Frame {
                    fill: self.theme.colors.darker_gray,
                    margin: Vec2::new(10., 10.),
                    corner_radius: 4.,
                    ..Default::default()
                }
                .show(ui, |ui| {
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
                        ui.label("There are no mods to display");
                    } else if filtered_list.is_empty() {
                        ui.label("No mods match your search");
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
                                                    mc_mod_meta::ModLoader::Forge => {
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
                                                    mc_mod_meta::ModLoader::Fabric => {
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
                                                };

                                                ui.add_space(5.);

                                                ui.label(text);
                                            });

                                            ui.horizontal(|ui| {
                                                let raw_text = text_utils::mod_card_data_text(
                                                    mod_entry.sourced_from.to_string(),
                                                );

                                                let text = match mod_entry.sourced_from {
                                                    Source::Local => {
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
                                                        self.sender
                                                            .as_ref()
                                                            .unwrap()
                                                            .send(ToBackend::UpdateMod {
                                                                version_id: mod_entry
                                                                    .modrinth_data
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .lastest_valid_version
                                                                    .clone(),
                                                                modloader: mod_entry.modloader,
                                                            })
                                                            .unwrap();
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
    fn scan_mod_folder(&mut self) {
        self.mod_list.clear();

        let dir = fs::canonicalize("./mods/").unwrap();

        let read_dir = fs::read_dir(dir).unwrap();

        for path in read_dir {
            let path = path.unwrap().path();

            let file = fs::File::open(&path).unwrap();

            self.mod_list.append(&mut ModEntry::from_file(file));
        }
    }

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
