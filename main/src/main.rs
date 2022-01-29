use app_theme::AppTheme;
use ferinth::Ferinth;
use image_utils::ImageTextures;
use message::{FetchingModContext, Message};
use mod_entry::ModEntry;
use std::{
    fs,
    sync::mpsc::{Receiver, Sender},
};

use eframe::{
    egui::{self, style::DebugOptions, Context, ProgressBar, RichText, Style, Vec2, Widget},
    epi,
};

use crate::mod_entry::{FileState, Source};

mod app_theme;
mod image_utils;
mod message;
mod mod_entry;
mod text_utils;
#[derive(Default)]
struct UiApp {
    theme: AppTheme,
    mod_list: Vec<ModEntry>,
    search_buf: String,
    app_rx: Option<Receiver<Message>>,
    fetching_info: Option<FetchingModContext>,
    images: ImageTextures,
}

impl epi::App for UiApp {
    fn name(&self) -> &str {
        "An App"
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.configure_style(ctx);
        self.images.load_images(ctx);

        self.scan_mod_folder();

        let mut mod_list = self.mod_list.clone();

        let (tx, rx) = std::sync::mpsc::channel::<Message>();

        self.app_rx = Some(rx);

        tokio::task::spawn(async move {
            struct ModrinthManager {
                client: Ferinth,
            }

            impl ModrinthManager {
                async fn fetch_mod_data(&self, mod_list: &mut Vec<ModEntry>, tx: Sender<Message>) {
                    let list_length = mod_list.len();
                    for (position, entry) in mod_list.iter_mut().enumerate() {
                        tx.send(Message::FetchingMod {
                            context: FetchingModContext {
                                name: entry.display_name.clone(),
                                position,
                                total: list_length,
                            },
                        })
                        .unwrap();

                        let modrinth_id = self.get_modrinth_id(entry.hashes.sha1.as_str()).await;

                        entry.modrinth_id = Some(modrinth_id.clone());

                        match self.client.list_versions(modrinth_id.as_str()).await {
                            Ok(version_data) => {
                                entry.sourced_from = Source::Modrinth;
                                // Assume its outdated unless proven otherwise
                                entry.state = FileState::Outdated;

                                'outer: for file in &version_data[0].files {
                                    if let Some(hash) = &file.hashes.sha1 {
                                        if hash == &entry.hashes.sha1 {
                                            entry.state = FileState::Current;
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                            Err(_err) => entry.state = FileState::Local,
                        };
                    }
                }

                async fn get_modrinth_id(&self, mod_hash: &str) -> String {
                    match self.client.get_version_from_file_hash(mod_hash).await {
                        Ok(result) => result.mod_id,
                        Err(_err) => "No".into(),
                    }
                }
            }

            let modrinth = ModrinthManager {
                client: Ferinth::new("Test app"),
            };

            let tx_clone = tx.clone();
            modrinth.fetch_mod_data(&mut mod_list, tx_clone).await;

            tx.send(Message::UpdatedModList { list: mod_list }).unwrap();
        });
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        if let Some(rx) = &self.app_rx {
            match rx.try_recv() {
                Ok(message) => {
                    //A
                    match message {
                        Message::UpdatedModList { list } => {
                            self.fetching_info = None;
                            self.mod_list = list;
                        }
                        Message::FetchingMod { context: mod_name } => {
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
                                                            "Le update button",
                                                        ))
                                                        .clicked()
                                                    {
                                                        //Boo
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

#[tokio::main(worker_threads = 4)]
async fn main() {
    let app = UiApp::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(970., 300.)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
